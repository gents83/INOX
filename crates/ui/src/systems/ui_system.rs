use std::{
    any::TypeId,
    collections::{hash_map::Entry, HashMap},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use egui::{
    ClippedMesh, CtxRef, Event, Modifiers, Output, PointerButton, RawInput, Rect,
    TextureId as eguiTextureId,
};
use image::RgbaImage;
use sabi_core::{JobHandlerRw, System};
use sabi_graphics::{
    Material, Mesh, MeshCategoryId, MeshData, Pipeline, RenderPass, RenderTarget, Texture,
    TextureId, TextureType, VertexData,
};

use sabi_math::Vector4;
use sabi_messenger::{read_messages, MessageChannel, MessengerRw};
use sabi_platform::{
    InputState, KeyEvent, KeyTextEvent, MouseButton, MouseEvent, MouseState, WindowEvent,
    DEFAULT_DPI,
};
use sabi_profiler::debug_log;
use sabi_resources::{
    ConfigBase, DataTypeResource, Handle, Resource, SerializableResource, SharedData, SharedDataRc,
};
use sabi_serialize::{generate_random_uid, read_from_file};

use crate::UIWidget;

use super::config::Config;

const UI_MESH_CATEGORY_IDENTIFIER: &str = "ui_mesh";

pub struct UISystem {
    shared_data: SharedDataRc,
    job_handler: JobHandlerRw,
    global_messenger: MessengerRw,
    message_channel: MessageChannel,
    ui_context: CtxRef,
    ui_texture_version: u64,
    ui_texture: Handle<Texture>,
    ui_input: RawInput,
    ui_input_modifiers: Modifiers,
    ui_clipboard: Option<String>,
    ui_pipeline: Handle<Pipeline>,
    ui_materials: HashMap<TextureId, Resource<Material>>,
    ui_meshes: Vec<Resource<Mesh>>,
    ui_scale: f32,
}

impl UISystem {
    pub fn new(
        shared_data: &SharedDataRc,
        global_messenger: &MessengerRw,
        job_handler: &JobHandlerRw,
    ) -> Self {
        let message_channel = MessageChannel::default();

        crate::register_resource_types(shared_data);

        Self {
            ui_pipeline: None,
            shared_data: shared_data.clone(),
            job_handler: job_handler.clone(),
            global_messenger: global_messenger.clone(),
            message_channel,
            ui_context: CtxRef::default(),
            ui_texture_version: 0,
            ui_texture: None,
            ui_input: RawInput::default(),
            ui_input_modifiers: Modifiers::default(),
            ui_clipboard: None,
            ui_materials: HashMap::new(),
            ui_meshes: Vec::new(),
            ui_scale: 2.,
        }
    }

    fn get_ui_material(&mut self, texture: Resource<Texture>) -> Resource<Material> {
        sabi_profiler::scoped_profile!("ui_system::get_ui_material");

        match self.ui_materials.entry(*texture.id()) {
            Entry::Occupied(e) => e.get().clone(),
            Entry::Vacant(e) => {
                if let Some(render_pass) =
                    SharedData::match_resource(&self.shared_data, |r: &RenderPass| {
                        r.data().render_target == RenderTarget::Screen
                    })
                {
                    let shared_data = self.shared_data.clone();
                    render_pass
                        .get_mut()
                        .add_category_to_draw(MeshCategoryId::new(UI_MESH_CATEGORY_IDENTIFIER));
                    if let Some(pipeline) = &self.ui_pipeline {
                        let material = Material::duplicate_from_pipeline(&shared_data, pipeline);
                        material
                            .get_mut()
                            .set_texture(TextureType::BaseColor, &texture);
                        e.insert(material.clone());
                        return material;
                    } else {
                        panic!("UI pipeline not set - maybe you forgot to read ui.cfg file");
                    };
                }
                panic!("No Pass with Screen as RenderTarget has been loaded");
            }
        }
    }

    fn update_egui_texture(&mut self) -> &mut Self {
        sabi_profiler::scoped_profile!("ui_system::update_egui_texture");
        if self.ui_texture_version != self.ui_context.texture().version {
            let mut pixels: Vec<u8> =
                Vec::with_capacity(self.ui_context.texture().pixels.len() * 4);
            for srgba in self.ui_context.texture().srgba_pixels(1.) {
                pixels.push(srgba.r());
                pixels.push(srgba.g());
                pixels.push(srgba.b());
                pixels.push(srgba.a());
            }
            let image_data = RgbaImage::from_vec(
                self.ui_context.texture().width as _,
                self.ui_context.texture().height as _,
                pixels,
            );
            if let Some(texture) = &self.ui_texture {
                if let Some(material) = self.ui_materials.remove(texture.id()) {
                    material.get_mut().remove_texture(TextureType::BaseColor);
                }
            }
            let texture = Texture::new_resource(
                &self.shared_data,
                &self.global_messenger,
                generate_random_uid(),
                image_data.unwrap(),
            );
            self.ui_texture = Some(texture);
            self.ui_texture_version = self.ui_context.texture().version;
        }
        self
    }

    fn compute_mesh_data(&mut self, clipped_meshes: Vec<ClippedMesh>) {
        sabi_profiler::scoped_profile!("ui_system::compute_mesh_data");
        let shared_data = self.shared_data.clone();
        let global_messenger = self.global_messenger.clone();
        self.ui_meshes.resize_with(clipped_meshes.len(), || {
            Mesh::new_resource(
                &shared_data,
                &global_messenger,
                generate_random_uid(),
                MeshData::new(UI_MESH_CATEGORY_IDENTIFIER),
            )
        });

        for (i, clipped_mesh) in clipped_meshes.into_iter().enumerate() {
            let ClippedMesh(clip_rect, mesh) = clipped_mesh;
            let draw_index = i as u32;
            self.ui_meshes[i].get_mut().set_draw_index(draw_index);
            if mesh.vertices.is_empty() || mesh.indices.is_empty() {
                continue;
            }
            let texture = match mesh.texture_id {
                eguiTextureId::Egui => self.ui_texture.as_ref().unwrap().clone(),
                eguiTextureId::User(index) => {
                    SharedData::get_resource_at_index(&self.shared_data, index as _).unwrap()
                }
            };
            let material = self.get_ui_material(texture);
            let mesh_instance = self.ui_meshes[i].clone();
            let ui_scale = self.ui_scale;
            let screen_rect = self.ui_input.screen_rect;
            let job_name = format!("ui_system::compute_mesh_data[{}]", i);
            self.job_handler.write().unwrap().add_job(
                &UISystem::id(),
                job_name.as_str(),
                move || {
                    let mut mesh_data = MeshData::new(UI_MESH_CATEGORY_IDENTIFIER);
                    let mut vertices: Vec<VertexData> = Vec::new();
                    vertices.resize(mesh.vertices.len(), VertexData::default());
                    for (i, v) in mesh.vertices.iter().enumerate() {
                        vertices[i].pos = [v.pos.x * ui_scale, v.pos.y * ui_scale, 0.].into();
                        vertices[i].tex_coord.iter_mut().for_each(|t| {
                            *t = [v.uv.x, v.uv.y].into();
                        });
                        vertices[i].color = [
                            v.color.r() as _,
                            v.color.g() as _,
                            v.color.b() as _,
                            v.color.a() as _,
                        ]
                        .into();
                    }
                    mesh_data.append_mesh(vertices.as_slice(), mesh.indices.as_slice());

                    let mut clip_rect = Vector4::new(
                        clip_rect.min.x * ui_scale,
                        clip_rect.min.y * ui_scale,
                        clip_rect.max.x * ui_scale,
                        clip_rect.max.y * ui_scale,
                    );

                    if let Some(screen_rect) = &screen_rect {
                        clip_rect.x = clip_rect.x.clamp(0.0, screen_rect.width());
                        clip_rect.y = clip_rect.y.clamp(0.0, screen_rect.height());
                        clip_rect.z = clip_rect.x.clamp(clip_rect.x, screen_rect.width());
                        clip_rect.w = clip_rect.y.clamp(clip_rect.y, screen_rect.height());
                    }

                    mesh_instance
                        .get_mut()
                        .set_material(material.clone())
                        .set_mesh_data(mesh_data.clone())
                        .set_draw_area(clip_rect);
                },
            );
        }
    }

    fn update_events(&mut self) -> &mut Self {
        self.ui_input.events.clear();
        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<MouseEvent>() {
                let event = msg.as_any().downcast_ref::<MouseEvent>().unwrap();
                if event.state == MouseState::Move {
                    self.ui_input.events.push(Event::PointerMoved(
                        [
                            event.x as f32 / self.ui_scale,
                            event.y as f32 / self.ui_scale,
                        ]
                        .into(),
                    ));
                } else if event.state == MouseState::Down || event.state == MouseState::Up {
                    self.ui_input.events.push(Event::PointerButton {
                        pos: [
                            event.x as f32 / self.ui_scale,
                            event.y as f32 / self.ui_scale,
                        ]
                        .into(),
                        button: match event.button {
                            MouseButton::Right => PointerButton::Secondary,
                            MouseButton::Middle => PointerButton::Middle,
                            _ => PointerButton::Primary,
                        },
                        pressed: event.state == MouseState::Down,
                        modifiers: self.ui_input_modifiers,
                    });
                }
            } else if msg.type_id() == TypeId::of::<WindowEvent>() {
                let event = msg.as_any().downcast_ref::<WindowEvent>().unwrap();
                match *event {
                    WindowEvent::SizeChanged(width, height) => {
                        self.ui_input.screen_rect = Some(Rect::from_min_size(
                            Default::default(),
                            [width as f32 / self.ui_scale, height as f32 / self.ui_scale].into(),
                        ));
                    }
                    WindowEvent::DpiChanged(x, _y) => {
                        self.ui_input.pixels_per_point = Some(x / DEFAULT_DPI);
                    }
                    _ => {}
                }
            } else if msg.type_id() == TypeId::of::<KeyEvent>() {
                let event = msg.as_any().downcast_ref::<KeyEvent>().unwrap();
                let just_pressed = event.state == InputState::JustPressed;
                let pressed = just_pressed || event.state == InputState::Pressed;

                if let Some(key) = convert_key(event.code) {
                    self.ui_input.events.push(Event::Key {
                        key,
                        pressed,
                        modifiers: self.ui_input_modifiers,
                    });
                }

                if event.code == sabi_platform::Key::Shift {
                    self.ui_input_modifiers.shift = pressed;
                } else if event.code == sabi_platform::Key::Control {
                    self.ui_input_modifiers.ctrl = pressed;
                    self.ui_input_modifiers.command = pressed;
                } else if event.code == sabi_platform::Key::Alt {
                    self.ui_input_modifiers.alt = pressed;
                } else if event.code == sabi_platform::Key::Meta {
                    self.ui_input_modifiers.command = pressed;
                    self.ui_input_modifiers.mac_cmd = pressed;
                }

                if just_pressed
                    && self.ui_input_modifiers.ctrl
                    && event.code == sabi_platform::input::Key::C
                {
                    self.ui_input.events.push(Event::Copy);
                } else if just_pressed
                    && self.ui_input_modifiers.ctrl
                    && event.code == sabi_platform::input::Key::X
                {
                    self.ui_input.events.push(Event::Cut);
                } else if just_pressed
                    && self.ui_input_modifiers.ctrl
                    && event.code == sabi_platform::input::Key::V
                {
                    if let Some(content) = &self.ui_clipboard {
                        self.ui_input.events.push(Event::Text(content.clone()));
                    }
                }
            } else if msg.type_id() == TypeId::of::<KeyTextEvent>() {
                let event = msg.as_any().downcast_ref::<KeyTextEvent>().unwrap();
                if event.char.is_ascii_control() {
                    return;
                }
                self.ui_input
                    .events
                    .push(Event::Text(event.char.to_string()));
            }
        });
        self
    }

    fn show_ui(
        shared_data: SharedDataRc,
        job_handler: JobHandlerRw,
        context: CtxRef,
        use_multithreading: bool,
    ) {
        sabi_profiler::scoped_profile!("ui_system::show_ui");
        let wait_count = Arc::new(AtomicUsize::new(0));
        shared_data.for_each_resource_mut(|widget_handle, widget: &mut UIWidget| {
            if use_multithreading {
                let context = context.clone();
                let widget_handle = widget_handle.clone();
                let job_name = format!("ui_system::show_ui[{:?}]", widget_handle.id());
                let wait_count = wait_count.clone();
                wait_count.fetch_add(1, Ordering::SeqCst);
                job_handler.write().unwrap().add_job(
                    &UISystem::id(),
                    job_name.as_str(),
                    move || {
                        widget_handle.get_mut().execute(&context);
                        wait_count.fetch_sub(1, Ordering::SeqCst);
                    },
                );
            } else {
                widget.execute(&context);
            }
        });
        while wait_count.load(Ordering::SeqCst) > 0 {
            thread::yield_now();
        }
    }

    fn handle_output(&mut self, output: Output) -> &mut Self {
        if let Some(open) = output.open_url {
            debug_log(format!("Trying to open url: {:?}", open.url).as_str());
        }

        if !output.copied_text.is_empty() {
            self.ui_clipboard = Some(output.copied_text);
        }

        self
    }
}

impl Drop for UISystem {
    fn drop(&mut self) {
        crate::unregister_resource_types(&self.shared_data);
    }
}

impl System for UISystem {
    fn read_config(&mut self, plugin_name: &str) {
        let mut config = Config::default();
        config = read_from_file(config.get_filepath(plugin_name).as_path());

        self.ui_scale = config.ui_scale;
        self.ui_pipeline = Some(Pipeline::request_load(
            &self.shared_data,
            &self.global_messenger,
            config.ui_pipeline.as_path(),
            None,
        ));
    }
    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {
        self.global_messenger
            .write()
            .unwrap()
            .register_messagebox::<WindowEvent>(self.message_channel.get_messagebox())
            .register_messagebox::<KeyEvent>(self.message_channel.get_messagebox())
            .register_messagebox::<KeyTextEvent>(self.message_channel.get_messagebox())
            .register_messagebox::<MouseEvent>(self.message_channel.get_messagebox());
    }

    fn run(&mut self) -> bool {
        self.update_events();

        let (output, shapes) = {
            sabi_profiler::scoped_profile!("ui_context::run");
            let shared_data = self.shared_data.clone();
            let job_handler = self.job_handler.clone();
            let ui_context = self.ui_context.clone();
            self.ui_context.run(self.ui_input.take(), move |_| {
                Self::show_ui(shared_data, job_handler, ui_context, true);
            })
        };
        let clipped_meshes = {
            sabi_profiler::scoped_profile!("ui_context::tessellate");
            self.ui_context.tessellate(shapes)
        };
        self.handle_output(output)
            .update_egui_texture()
            .compute_mesh_data(clipped_meshes);

        true
    }

    fn uninit(&mut self) {
        self.global_messenger
            .write()
            .unwrap()
            .unregister_messagebox::<MouseEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<KeyTextEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<KeyEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<WindowEvent>(self.message_channel.get_messagebox());
    }
}

fn convert_key(key: sabi_platform::input::Key) -> Option<egui::Key> {
    match key {
        sabi_platform::Key::ArrowDown => Some(egui::Key::ArrowDown),
        sabi_platform::Key::ArrowLeft => Some(egui::Key::ArrowLeft),
        sabi_platform::Key::ArrowRight => Some(egui::Key::ArrowRight),
        sabi_platform::Key::ArrowUp => Some(egui::Key::ArrowUp),
        sabi_platform::Key::Escape => Some(egui::Key::Escape),
        sabi_platform::Key::Tab => Some(egui::Key::Tab),
        sabi_platform::Key::Backspace => Some(egui::Key::Backspace),
        sabi_platform::Key::Enter => Some(egui::Key::Enter),
        sabi_platform::Key::Space => Some(egui::Key::Space),
        sabi_platform::Key::Insert => Some(egui::Key::Insert),
        sabi_platform::Key::Delete => Some(egui::Key::Delete),
        sabi_platform::Key::Home => Some(egui::Key::Home),
        sabi_platform::Key::End => Some(egui::Key::End),
        sabi_platform::Key::PageUp => Some(egui::Key::PageUp),
        sabi_platform::Key::PageDown => Some(egui::Key::PageDown),
        sabi_platform::Key::Numpad0 | sabi_platform::Key::Key0 => Some(egui::Key::Num0),
        sabi_platform::Key::Numpad1 | sabi_platform::Key::Key1 => Some(egui::Key::Num1),
        sabi_platform::Key::Numpad2 | sabi_platform::Key::Key2 => Some(egui::Key::Num2),
        sabi_platform::Key::Numpad3 | sabi_platform::Key::Key3 => Some(egui::Key::Num3),
        sabi_platform::Key::Numpad4 | sabi_platform::Key::Key4 => Some(egui::Key::Num4),
        sabi_platform::Key::Numpad5 | sabi_platform::Key::Key5 => Some(egui::Key::Num5),
        sabi_platform::Key::Numpad6 | sabi_platform::Key::Key6 => Some(egui::Key::Num6),
        sabi_platform::Key::Numpad7 | sabi_platform::Key::Key7 => Some(egui::Key::Num7),
        sabi_platform::Key::Numpad8 | sabi_platform::Key::Key8 => Some(egui::Key::Num8),
        sabi_platform::Key::Numpad9 | sabi_platform::Key::Key9 => Some(egui::Key::Num9),
        sabi_platform::Key::A => Some(egui::Key::A),
        sabi_platform::Key::B => Some(egui::Key::B),
        sabi_platform::Key::C => Some(egui::Key::C),
        sabi_platform::Key::D => Some(egui::Key::D),
        sabi_platform::Key::E => Some(egui::Key::E),
        sabi_platform::Key::F => Some(egui::Key::F),
        sabi_platform::Key::G => Some(egui::Key::G),
        sabi_platform::Key::H => Some(egui::Key::H),
        sabi_platform::Key::I => Some(egui::Key::I),
        sabi_platform::Key::J => Some(egui::Key::J),
        sabi_platform::Key::K => Some(egui::Key::K),
        sabi_platform::Key::L => Some(egui::Key::L),
        sabi_platform::Key::M => Some(egui::Key::M),
        sabi_platform::Key::N => Some(egui::Key::N),
        sabi_platform::Key::O => Some(egui::Key::O),
        sabi_platform::Key::P => Some(egui::Key::P),
        sabi_platform::Key::Q => Some(egui::Key::Q),
        sabi_platform::Key::R => Some(egui::Key::R),
        sabi_platform::Key::S => Some(egui::Key::S),
        sabi_platform::Key::T => Some(egui::Key::T),
        sabi_platform::Key::U => Some(egui::Key::U),
        sabi_platform::Key::V => Some(egui::Key::V),
        sabi_platform::Key::W => Some(egui::Key::W),
        sabi_platform::Key::X => Some(egui::Key::X),
        sabi_platform::Key::Y => Some(egui::Key::Y),
        sabi_platform::Key::Z => Some(egui::Key::Z),
        _ => None,
    }
}

use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use egui::{
    ClippedMesh, Context, Event, Modifiers, Output, PointerButton, RawInput, Rect,
    TextureId as eguiTextureId,
};
use image::RgbaImage;
use inox_core::{JobHandlerRw, System};
use inox_graphics::{
    Material, Mesh, MeshData, Pipeline, Texture, TextureId, TextureType, VertexData,
};

use inox_math::Vector4;
use inox_messenger::{Listener, MessageHubRc};
use inox_platform::{
    InputState, KeyEvent, KeyTextEvent, MouseButton, MouseEvent, MouseState, WindowEvent,
    DEFAULT_DPI,
};
use inox_profiler::debug_log;
use inox_resources::{
    ConfigBase, DataTypeResource, Handle, Resource, SerializableResource, SharedDataRc,
};
use inox_serialize::{generate_random_uid, read_from_file};

use crate::UIWidget;

use super::config::Config;

const UI_MESH_CATEGORY_IDENTIFIER: &str = "ui_mesh";

pub struct UISystem {
    shared_data: SharedDataRc,
    job_handler: JobHandlerRw,
    message_hub: MessageHubRc,
    listener: Listener,
    ui_context: Context,
    ui_textures: HashMap<eguiTextureId, Resource<Texture>>,
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
        message_hub: &MessageHubRc,
        job_handler: &JobHandlerRw,
    ) -> Self {
        let listener = Listener::new(message_hub);

        crate::register_resource_types(shared_data);

        Self {
            ui_pipeline: None,
            shared_data: shared_data.clone(),
            job_handler: job_handler.clone(),
            message_hub: message_hub.clone(),
            listener,
            ui_context: Context::default(),
            ui_textures: HashMap::new(),
            ui_input: RawInput::default(),
            ui_input_modifiers: Modifiers::default(),
            ui_clipboard: None,
            ui_materials: HashMap::new(),
            ui_meshes: Vec::new(),
            ui_scale: 2.,
        }
    }

    fn get_ui_material(&mut self, texture: Resource<Texture>) -> Resource<Material> {
        inox_profiler::scoped_profile!("ui_system::get_ui_material");

        match self.ui_materials.entry(*texture.id()) {
            Entry::Occupied(e) => e.get().clone(),
            Entry::Vacant(e) => {
                let shared_data = self.shared_data.clone();
                let message_hub = self.message_hub.clone();
                if let Some(pipeline) = &self.ui_pipeline {
                    let material =
                        Material::duplicate_from_pipeline(&shared_data, &message_hub, pipeline);
                    material
                        .get_mut()
                        .set_texture(TextureType::BaseColor, &texture);
                    e.insert(material.clone());
                    material
                } else {
                    panic!("UI pipeline not set - maybe you forgot to read ui.cfg file");
                }
            }
        }
    }

    fn compute_mesh_data(&mut self, clipped_meshes: Vec<ClippedMesh>) {
        inox_profiler::scoped_profile!("ui_system::compute_mesh_data");
        let shared_data = self.shared_data.clone();
        let message_hub = self.message_hub.clone();
        self.ui_meshes.resize_with(clipped_meshes.len(), || {
            Mesh::new_resource(
                &shared_data,
                &message_hub,
                generate_random_uid(),
                MeshData::default(),
            )
        });

        for (i, clipped_mesh) in clipped_meshes.into_iter().enumerate() {
            let ClippedMesh(clip_rect, mesh) = clipped_mesh;
            let draw_index = i as u32;
            if mesh.vertices.is_empty() || mesh.indices.is_empty() {
                self.ui_meshes[i].get_mut().set_visible(false);
                continue;
            }
            let texture = match mesh.texture_id {
                eguiTextureId::Managed(_) => self.ui_textures[&mesh.texture_id].clone(),
                eguiTextureId::User(texture_uniform_index) => {
                    if let Some(texture) = self.shared_data.match_resource(|t: &Texture| {
                        t.uniform_index() as u64 == texture_uniform_index
                    }) {
                        texture.clone()
                    } else {
                        self.ui_textures.iter().next().unwrap().1.clone()
                    }
                }
            };
            let material = self.get_ui_material(texture);
            let mesh_instance = &self.ui_meshes[i];
            let ui_scale = self.ui_scale;

            let mut mesh_data = MeshData::default();
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

            let clip_rect = Vector4::new(
                clip_rect.min.x * ui_scale,
                clip_rect.min.y * ui_scale,
                clip_rect.max.x * ui_scale,
                clip_rect.max.y * ui_scale,
            );

            mesh_instance
                .get_mut()
                .set_material(material)
                .set_mesh_data(mesh_data)
                .set_draw_area(clip_rect)
                .set_draw_index(draw_index)
                .set_visible(true);
        }
    }

    fn update_events(&mut self) -> &mut Self {
        self.ui_input.events.clear();

        Listener::process_messages(&self.listener, |event: &MouseEvent| {
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
        });
        Listener::process_messages(&self.listener, |event: &WindowEvent| match *event {
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
        });
        Listener::process_messages(&self.listener, |event: &KeyEvent| {
            let just_pressed = event.state == InputState::JustPressed;
            let pressed = just_pressed || event.state == InputState::Pressed;

            if let Some(key) = convert_key(event.code) {
                self.ui_input.events.push(Event::Key {
                    key,
                    pressed,
                    modifiers: self.ui_input_modifiers,
                });
            }

            if event.code == inox_platform::Key::Shift {
                self.ui_input_modifiers.shift = pressed;
            } else if event.code == inox_platform::Key::Control {
                self.ui_input_modifiers.ctrl = pressed;
                self.ui_input_modifiers.command = pressed;
            } else if event.code == inox_platform::Key::Alt {
                self.ui_input_modifiers.alt = pressed;
            } else if event.code == inox_platform::Key::Meta {
                self.ui_input_modifiers.command = pressed;
                self.ui_input_modifiers.mac_cmd = pressed;
            }

            if just_pressed
                && self.ui_input_modifiers.ctrl
                && event.code == inox_platform::input::Key::C
            {
                self.ui_input.events.push(Event::Copy);
            } else if just_pressed
                && self.ui_input_modifiers.ctrl
                && event.code == inox_platform::input::Key::X
            {
                self.ui_input.events.push(Event::Cut);
            } else if just_pressed
                && self.ui_input_modifiers.ctrl
                && event.code == inox_platform::input::Key::V
            {
                if let Some(content) = &self.ui_clipboard {
                    self.ui_input.events.push(Event::Text(content.clone()));
                }
            }
        });
        Listener::process_messages(&self.listener, |event: &KeyTextEvent| {
            if event.char.is_ascii_control() {
                return;
            }
            self.ui_input
                .events
                .push(Event::Text(event.char.to_string()));
        });

        self
    }

    fn show_ui(
        shared_data: SharedDataRc,
        job_handler: JobHandlerRw,
        context: Context,
        use_multithreading: bool,
    ) {
        inox_profiler::scoped_profile!("ui_system::show_ui");
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

        for (egui_texture_id, image_delta) in output.textures_delta.set {
            let pixels: Vec<u8> = match &image_delta.image {
                egui::ImageData::Color(image) => {
                    assert_eq!(
                        image.width() * image.height(),
                        image.pixels.len(),
                        "Mismatch between texture size and texel count"
                    );
                    image
                        .pixels
                        .iter()
                        .flat_map(|color| color.to_array().to_vec())
                        .collect()
                }
                egui::ImageData::Alpha(image) => {
                    let gamma = 1.0;
                    image
                        .srgba_pixels(gamma)
                        .flat_map(|color| color.to_array().to_vec())
                        .collect()
                }
            };
            let image_data = RgbaImage::from_vec(
                image_delta.image.width() as _,
                image_delta.image.height() as _,
                pixels,
            );
            if let Some(texture) = self.ui_textures.get(&egui_texture_id) {
                if let Some(material) = self.ui_materials.remove(texture.id()) {
                    material.get_mut().remove_texture(TextureType::BaseColor);
                }
            }
            let texture = Texture::new_resource(
                &self.shared_data,
                &self.message_hub,
                generate_random_uid(),
                image_data.unwrap(),
            );
            self.ui_textures.insert(egui_texture_id, texture);
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
            &self.message_hub,
            config.ui_pipeline.as_path(),
            None,
        ));
    }
    fn should_run_when_not_focused(&self) -> bool {
        false
    }
    fn init(&mut self) {
        self.listener
            .register::<WindowEvent>()
            .register::<KeyEvent>()
            .register::<KeyTextEvent>()
            .register::<MouseEvent>();
    }

    fn run(&mut self) -> bool {
        self.update_events();

        let (output, shapes) = {
            inox_profiler::scoped_profile!("ui_context::run");
            let shared_data = self.shared_data.clone();
            let job_handler = self.job_handler.clone();
            let ui_context = self.ui_context.clone();
            self.ui_context.run(self.ui_input.take(), move |_| {
                Self::show_ui(shared_data, job_handler, ui_context, false);
            })
        };
        let clipped_meshes = {
            inox_profiler::scoped_profile!("ui_context::tessellate");
            self.ui_context.tessellate(shapes)
        };
        self.handle_output(output).compute_mesh_data(clipped_meshes);

        true
    }

    fn uninit(&mut self) {
        self.listener
            .unregister::<MouseEvent>()
            .unregister::<KeyTextEvent>()
            .unregister::<KeyEvent>()
            .unregister::<WindowEvent>();
    }
}

fn convert_key(key: inox_platform::input::Key) -> Option<egui::Key> {
    match key {
        inox_platform::Key::ArrowDown => Some(egui::Key::ArrowDown),
        inox_platform::Key::ArrowLeft => Some(egui::Key::ArrowLeft),
        inox_platform::Key::ArrowRight => Some(egui::Key::ArrowRight),
        inox_platform::Key::ArrowUp => Some(egui::Key::ArrowUp),
        inox_platform::Key::Escape => Some(egui::Key::Escape),
        inox_platform::Key::Tab => Some(egui::Key::Tab),
        inox_platform::Key::Backspace => Some(egui::Key::Backspace),
        inox_platform::Key::Enter => Some(egui::Key::Enter),
        inox_platform::Key::Space => Some(egui::Key::Space),
        inox_platform::Key::Insert => Some(egui::Key::Insert),
        inox_platform::Key::Delete => Some(egui::Key::Delete),
        inox_platform::Key::Home => Some(egui::Key::Home),
        inox_platform::Key::End => Some(egui::Key::End),
        inox_platform::Key::PageUp => Some(egui::Key::PageUp),
        inox_platform::Key::PageDown => Some(egui::Key::PageDown),
        inox_platform::Key::Numpad0 | inox_platform::Key::Key0 => Some(egui::Key::Num0),
        inox_platform::Key::Numpad1 | inox_platform::Key::Key1 => Some(egui::Key::Num1),
        inox_platform::Key::Numpad2 | inox_platform::Key::Key2 => Some(egui::Key::Num2),
        inox_platform::Key::Numpad3 | inox_platform::Key::Key3 => Some(egui::Key::Num3),
        inox_platform::Key::Numpad4 | inox_platform::Key::Key4 => Some(egui::Key::Num4),
        inox_platform::Key::Numpad5 | inox_platform::Key::Key5 => Some(egui::Key::Num5),
        inox_platform::Key::Numpad6 | inox_platform::Key::Key6 => Some(egui::Key::Num6),
        inox_platform::Key::Numpad7 | inox_platform::Key::Key7 => Some(egui::Key::Num7),
        inox_platform::Key::Numpad8 | inox_platform::Key::Key8 => Some(egui::Key::Num8),
        inox_platform::Key::Numpad9 | inox_platform::Key::Key9 => Some(egui::Key::Num9),
        inox_platform::Key::A => Some(egui::Key::A),
        inox_platform::Key::B => Some(egui::Key::B),
        inox_platform::Key::C => Some(egui::Key::C),
        inox_platform::Key::D => Some(egui::Key::D),
        inox_platform::Key::E => Some(egui::Key::E),
        inox_platform::Key::F => Some(egui::Key::F),
        inox_platform::Key::G => Some(egui::Key::G),
        inox_platform::Key::H => Some(egui::Key::H),
        inox_platform::Key::I => Some(egui::Key::I),
        inox_platform::Key::J => Some(egui::Key::J),
        inox_platform::Key::K => Some(egui::Key::K),
        inox_platform::Key::L => Some(egui::Key::L),
        inox_platform::Key::M => Some(egui::Key::M),
        inox_platform::Key::N => Some(egui::Key::N),
        inox_platform::Key::O => Some(egui::Key::O),
        inox_platform::Key::P => Some(egui::Key::P),
        inox_platform::Key::Q => Some(egui::Key::Q),
        inox_platform::Key::R => Some(egui::Key::R),
        inox_platform::Key::S => Some(egui::Key::S),
        inox_platform::Key::T => Some(egui::Key::T),
        inox_platform::Key::U => Some(egui::Key::U),
        inox_platform::Key::V => Some(egui::Key::V),
        inox_platform::Key::W => Some(egui::Key::W),
        inox_platform::Key::X => Some(egui::Key::X),
        inox_platform::Key::Y => Some(egui::Key::Y),
        inox_platform::Key::Z => Some(egui::Key::Z),
        _ => None,
    }
}

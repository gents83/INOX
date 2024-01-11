use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use egui::{
    epaint::Primitive, ClippedPrimitive, Context, Event, Modifiers, PlatformOutput, PointerButton,
    RawInput, Rect, TextureId as eguiTextureId, TexturesDelta,
};

use inox_core::{
    implement_unique_system_uid, ContextRc, JobHandlerRw, JobHandlerTrait, JobPriority, System,
    SystemUID,
};

use inox_graphics::{Texture, TextureData, TextureFormat, TextureUsage};

use inox_log::debug_log;
use inox_messenger::{Listener, MessageHubRc};
use inox_platform::{
    InputState, KeyEvent, KeyTextEvent, MouseButton, MouseEvent, MouseState, WindowEvent,
};
use inox_resources::{to_slice, ConfigBase, ConfigEvent, DataTypeResource, Resource, SharedDataRc};
use inox_serialize::read_from_file;
use inox_uid::generate_random_uid;

use crate::{UIEvent, UIInstance, UIVertex, UIWidget};

use super::config::Config;

pub struct UISystem {
    config: Config,
    shared_data: SharedDataRc,
    job_handler: JobHandlerRw,
    message_hub: MessageHubRc,
    listener: Listener,
    ui_context: Context,
    ui_textures: HashMap<eguiTextureId, Resource<Texture>>,
    ui_input: RawInput,
    ui_input_modifiers: Modifiers,
    ui_clipboard: Option<String>,
    ui_scale: f32,
}

impl UISystem {
    pub fn new(context: &ContextRc) -> Self {
        let listener = Listener::new(context.message_hub());

        crate::register_resource_types(context.shared_data(), context.message_hub());

        Self {
            config: Config::default(),
            shared_data: context.shared_data().clone(),
            message_hub: context.message_hub().clone(),
            job_handler: context.job_handler().clone(),
            listener,
            ui_context: Context::default(),
            ui_textures: HashMap::new(),
            ui_input: RawInput::default(),
            ui_input_modifiers: Modifiers::default(),
            ui_clipboard: None,
            ui_scale: 1.,
        }
    }

    fn compute_mesh_data(&mut self, primitives: Vec<ClippedPrimitive>) {
        inox_profiler::scoped_profile!("ui_system::compute_mesh_data");
        let mut vertices: Vec<UIVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut instances: Vec<UIInstance> = Vec::new();

        for primitive in primitives.into_iter() {
            if let Primitive::Mesh(mesh) = primitive.primitive {
                if mesh.vertices.is_empty() || mesh.indices.is_empty() {
                    continue;
                }
                inox_profiler::scoped_profile!("ui_system::create_mesh_data");

                let texture_index = match mesh.texture_id {
                    eguiTextureId::Managed(_) => {
                        self.ui_textures[&mesh.texture_id].get().texture_index()
                    }
                    eguiTextureId::User(texture_uniform_index) => texture_uniform_index as _,
                };

                instances.push(UIInstance {
                    index_start: indices.len() as _,
                    index_count: mesh.indices.len() as _,
                    vertex_start: vertices.len() as _,
                    texture_index: texture_index as _,
                });
                vertices.extend_from_slice(to_slice(mesh.vertices.as_slice()));
                indices.extend_from_slice(&mesh.indices);
            }
        }
        self.message_hub
            .send_event(UIEvent::DrawData(vertices, indices, instances));
    }

    fn update_events(&mut self) -> &mut Self {
        self.ui_input.events.clear();

        self.listener
            .process_messages(|event: &MouseEvent| {
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
            })
            .process_messages(|e: &ConfigEvent<Config>| match e {
                ConfigEvent::Loaded(filename, config) => {
                    if filename == self.config.get_filename() {
                        self.config = config.clone();
                        self.ui_scale = self.config.ui_scale;
                        self.ui_context.set_zoom_factor(self.ui_scale);
                        self.message_hub.send_event(UIEvent::Scale(self.ui_scale));
                    }
                }
            })
            .process_messages(|event: &WindowEvent| match *event {
                WindowEvent::SizeChanged(width, height) => {
                    self.ui_input.screen_rect = Some(Rect::from_min_size(
                        Default::default(),
                        [width as f32, height as f32].into(),
                    ));
                    if width < 1 || height < 1 {
                        self.ui_input.screen_rect = None;
                    }
                }
                WindowEvent::ScaleFactorChanged(v) => {
                    self.ui_scale = v.max(1.) * self.config.ui_scale.max(1.);
                    self.ui_context.set_zoom_factor(self.ui_scale);
                    self.message_hub.send_event(UIEvent::Scale(self.ui_scale));
                }
                _ => {}
            })
            .process_messages(|event: &KeyEvent| {
                let just_pressed = event.state == InputState::JustPressed;
                let is_repeat = event.state == InputState::Pressed;
                let pressed = just_pressed || is_repeat;

                if let Some(key) = convert_key(event.code) {
                    self.ui_input.events.push(Event::Key {
                        key,
                        physical_key: None,
                        pressed,
                        repeat: is_repeat,
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
            })
            .process_messages(|event: &KeyTextEvent| {
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
                job_handler.add_job(
                    &UISystem::system_id(),
                    job_name.as_str(),
                    JobPriority::Medium,
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
            std::thread::yield_now();
        }
    }

    fn handle_output(
        &mut self,
        output: PlatformOutput,
        textures_delta: TexturesDelta,
    ) -> &mut Self {
        if let Some(open) = output.open_url {
            debug_log!("Trying to open url: {:?}", open.url);
        }

        if !output.copied_text.is_empty() {
            self.ui_clipboard = Some(output.copied_text);
        }

        for (egui_texture_id, image_delta) in textures_delta.set {
            let color32 = match &image_delta.image {
                egui::ImageData::Color(image) => {
                    assert_eq!(
                        image_delta.image.width() * image_delta.image.height(),
                        image.pixels.len(),
                        "Mismatch between texture size and texel count"
                    );
                    Cow::Borrowed(&image.pixels)
                }
                egui::ImageData::Font(image) => Cow::Owned(image.srgba_pixels(None).collect()),
            };
            let pixels: &[u8] = to_slice(color32.as_slice());
            if let Some(pos) = image_delta.pos {
                let texture = self.ui_textures.get(&egui_texture_id).unwrap();
                texture.get_mut().update(
                    [pos[0] as u32, pos[1] as u32].into(),
                    [
                        image_delta.image.width() as u32,
                        image_delta.image.height() as u32,
                    ]
                    .into(),
                    pixels,
                );
            } else {
                let texture_data = TextureData {
                    width: image_delta.image.width() as _,
                    height: image_delta.image.height() as _,
                    data: Some(pixels.to_vec()),
                    format: TextureFormat::Rgba8Unorm,
                    usage: TextureUsage::TextureBinding | TextureUsage::CopyDst,
                    sample_count: 1,
                    is_LUT: false,
                };
                let texture = Texture::new_resource(
                    &self.shared_data,
                    &self.message_hub,
                    generate_random_uid(),
                    &texture_data,
                    None,
                );
                self.ui_textures.insert(egui_texture_id, texture);
            }
        }

        self
    }
}

impl Drop for UISystem {
    fn drop(&mut self) {
        crate::unregister_resource_types(&self.shared_data, &self.message_hub);
    }
}

implement_unique_system_uid!(UISystem);

impl System for UISystem {
    fn read_config(&mut self, plugin_name: &str) {
        self.listener.register::<ConfigEvent<Config>>();
        let message_hub = self.message_hub.clone();
        let filename = self.config.get_filename().to_string();
        read_from_file(
            self.config.get_filepath(plugin_name).as_path(),
            self.shared_data.serializable_registry(),
            Box::new(move |data: Config| {
                message_hub.send_event(ConfigEvent::Loaded(filename.clone(), data));
            }),
        );
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

        let output = {
            inox_profiler::scoped_profile!("ui_context::run");
            let shared_data = self.shared_data.clone();
            let job_handler = self.job_handler.clone();
            let ui_context = self.ui_context.clone();
            self.ui_context.run(self.ui_input.take(), move |_| {
                Self::show_ui(shared_data, job_handler, ui_context, false);
            })
        };
        /*
        if !output.needs_repaint {
            return true;
        }*/

        let clipped_meshes = {
            inox_profiler::scoped_profile!("ui_context::tessellate");
            self.ui_context.tessellate(output.shapes, self.ui_scale)
        };
        self.handle_output(output.platform_output, output.textures_delta)
            .compute_mesh_data(clipped_meshes);

        true
    }

    fn uninit(&mut self) {
        self.listener
            .unregister::<MouseEvent>()
            .unregister::<KeyTextEvent>()
            .unregister::<KeyEvent>()
            .unregister::<WindowEvent>()
            .unregister::<ConfigEvent<Config>>();
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

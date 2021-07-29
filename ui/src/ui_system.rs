use std::any::TypeId;

use egui::{
    pos2, vec2, ClippedMesh, CtxRef, Event, Modifiers, Output, PointerButton, RawInput, Rect,
};
use image::{DynamicImage, Pixel};
use nrg_core::{JobHandlerRw, System, SystemId};
use nrg_graphics::{
    MaterialInstance, MaterialRc, MeshData, MeshInstance, PipelineInstance, TextureInstance,
    VertexData,
};

use nrg_messenger::{read_messages, MessageChannel, MessengerRw};
use nrg_platform::{
    InputState, KeyEvent, KeyTextEvent, MouseButton, MouseEvent, MouseState, WindowEvent,
    DEFAULT_DPI,
};
use nrg_resources::{DataTypeResource, SharedData, SharedDataRw};

use crate::UIWidget;

pub struct UISystem {
    id: SystemId,
    shared_data: SharedDataRw,
    job_handler: JobHandlerRw,
    global_messenger: MessengerRw,
    message_channel: MessageChannel,
    ui_context: CtxRef,
    ui_texture_version: u64,
    ui_input: RawInput,
    ui_input_modifiers: Modifiers,
    ui_clipboard: Option<String>,
    ui_default_material: MaterialRc,
    ui_scale: f32,
}

impl UISystem {
    pub fn new(
        shared_data: SharedDataRw,
        global_messenger: MessengerRw,
        job_handler: JobHandlerRw,
    ) -> Self {
        let message_channel = MessageChannel::default();
        Self {
            id: SystemId::new(),
            shared_data,
            job_handler,
            global_messenger,
            message_channel,
            ui_context: CtxRef::default(),
            ui_texture_version: 0,
            ui_input: RawInput::default(),
            ui_input_modifiers: Modifiers::default(),
            ui_clipboard: None,
            ui_default_material: MaterialRc::default(),
            ui_scale: 2.,
        }
    }

    fn create_default_material(&mut self) -> &mut Self {
        if let Some(pipeline) =
            SharedData::match_resource(&self.shared_data, |p: &PipelineInstance| {
                p.data().name == "UI"
            })
        {
            self.ui_default_material =
                MaterialInstance::create_from_pipeline(&self.shared_data, pipeline);
        } else {
            panic!("No pipeline with name UI has been loaded");
        }
        self
    }

    fn update_egui_texture(&mut self) -> &mut Self {
        if self.ui_texture_version != self.ui_context.texture().version {
            let image = DynamicImage::new_rgba8(
                self.ui_context.texture().width as _,
                self.ui_context.texture().height as _,
            );
            let mut image_data = image.to_rgba8();
            let (width, height) = image_data.dimensions();
            for x in 0..width {
                for y in 0..height {
                    let r = self.ui_context.texture().pixels[(x + y * width) as usize];
                    image_data.put_pixel(x, y, Pixel::from_channels(r, r, r, r));
                }
            }
            let ui_texture = TextureInstance::create_from_data(&self.shared_data, image_data);
            self.ui_default_material
                .resource()
                .get_mut()
                .add_texture(ui_texture);
            self.ui_texture_version = self.ui_context.texture().version;
        }
        self
    }

    fn compute_mesh_data(&mut self, clipped_meshes: Vec<ClippedMesh>) -> MeshData {
        let mut mesh_data = MeshData::default();
        let mut max_indices = 0;
        for clipped_mesh in clipped_meshes {
            let ClippedMesh(_, mesh) = clipped_mesh;
            if mesh.vertices.is_empty() || mesh.indices.is_empty() {
                continue;
            }
            let vertices: Vec<VertexData> = mesh
                .vertices
                .iter()
                .map(|v| VertexData {
                    pos: [v.pos.x * self.ui_scale, v.pos.y * self.ui_scale, 0.].into(),
                    tex_coord: [v.uv.x, v.uv.y].into(),
                    color: [
                        v.color.r() as f32 / 255.,
                        v.color.g() as f32 / 255.,
                        v.color.b() as f32 / 255.,
                        v.color.a() as f32 / 255.,
                    ]
                    .into(),
                    ..Default::default()
                })
                .collect();
            let indices: Vec<u32> = mesh.indices.iter().map(|i| i + max_indices).collect();
            max_indices += vertices.len() as u32;
            mesh_data.append_mesh(vertices.as_slice(), indices.as_slice());
        }
        mesh_data
    }

    fn update_mesh_in_ui_material(&mut self, mesh_data: MeshData) -> &mut Self {
        if !self.ui_default_material.resource().get().has_meshes() {
            let mesh = MeshInstance::create_from_data(&self.shared_data, mesh_data);
            self.ui_default_material.resource().get_mut().add_mesh(mesh);
        } else {
            self.ui_default_material
                .resource()
                .get()
                .meshes()
                .first()
                .unwrap()
                .resource()
                .get_mut()
                .set_mesh_data(mesh_data);
        }
        self
    }

    fn update_events(&mut self) -> &mut Self {
        self.ui_input.events.clear();
        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<MouseEvent>() {
                let event = msg.as_any().downcast_ref::<MouseEvent>().unwrap();
                if event.state == MouseState::Move {
                    self.ui_input.events.push(Event::PointerMoved(pos2(
                        event.x as f32 / self.ui_scale,
                        event.y as f32 / self.ui_scale,
                    )));
                } else if event.state == MouseState::Down || event.state == MouseState::Up {
                    self.ui_input.events.push(Event::PointerButton {
                        pos: pos2(
                            event.x as f32 / self.ui_scale,
                            event.y as f32 / self.ui_scale,
                        ),
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
                            vec2(width as f32 / self.ui_scale, height as f32 / self.ui_scale),
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

                if event.code == nrg_platform::Key::Shift {
                    self.ui_input_modifiers.shift = pressed;
                } else if event.code == nrg_platform::Key::Control {
                    self.ui_input_modifiers.ctrl = pressed;
                    self.ui_input_modifiers.command = pressed;
                } else if event.code == nrg_platform::Key::Alt {
                    self.ui_input_modifiers.alt = pressed;
                } else if event.code == nrg_platform::Key::Meta {
                    self.ui_input_modifiers.command = pressed;
                    self.ui_input_modifiers.mac_cmd = pressed;
                }

                if just_pressed
                    && self.ui_input_modifiers.ctrl
                    && event.code == nrg_platform::input::Key::C
                {
                    self.ui_input.events.push(Event::Copy);
                } else if just_pressed
                    && self.ui_input_modifiers.ctrl
                    && event.code == nrg_platform::input::Key::X
                {
                    self.ui_input.events.push(Event::Cut);
                } else if just_pressed
                    && self.ui_input_modifiers.ctrl
                    && event.code == nrg_platform::input::Key::V
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

    fn draw_ui(&mut self) {
        let widgets = SharedData::get_resources_of_type::<UIWidget>(&self.shared_data);
        for widget in widgets {
            widget.resource().get_mut().execute(&self.ui_context);
        }
    }

    fn handle_output(&mut self, output: Output) -> &mut Self {
        if let Some(open) = output.open_url {
            println!("Trying to open url: {:?}", open.url);
        }

        if !output.copied_text.is_empty() {
            self.ui_clipboard = Some(output.copied_text);
            println!("Clipboard content: {:?}", self.ui_clipboard);
        }

        self
    }
}

impl System for UISystem {
    fn id(&self) -> nrg_core::SystemId {
        self.id
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

        self.create_default_material();
    }

    fn run(&mut self) -> bool {
        self.update_events();

        self.ui_context.begin_frame(self.ui_input.take());

        self.draw_ui();

        let (output, shapes) = self.ui_context.end_frame();
        let clipped_meshes = self.ui_context.tessellate(shapes);

        let mesh_data = self.compute_mesh_data(clipped_meshes);
        self.handle_output(output)
            .update_mesh_in_ui_material(mesh_data)
            .update_egui_texture();

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

fn convert_key(key: nrg_platform::input::Key) -> Option<egui::Key> {
    match key {
        nrg_platform::Key::ArrowDown => Some(egui::Key::ArrowDown),
        nrg_platform::Key::ArrowLeft => Some(egui::Key::ArrowLeft),
        nrg_platform::Key::ArrowRight => Some(egui::Key::ArrowRight),
        nrg_platform::Key::ArrowUp => Some(egui::Key::ArrowUp),
        nrg_platform::Key::Escape => Some(egui::Key::Escape),
        nrg_platform::Key::Tab => Some(egui::Key::Tab),
        nrg_platform::Key::Backspace => Some(egui::Key::Backspace),
        nrg_platform::Key::Enter => Some(egui::Key::Enter),
        nrg_platform::Key::Space => Some(egui::Key::Space),
        nrg_platform::Key::Insert => Some(egui::Key::Insert),
        nrg_platform::Key::Delete => Some(egui::Key::Delete),
        nrg_platform::Key::Home => Some(egui::Key::Home),
        nrg_platform::Key::End => Some(egui::Key::End),
        nrg_platform::Key::PageUp => Some(egui::Key::PageUp),
        nrg_platform::Key::PageDown => Some(egui::Key::PageDown),
        nrg_platform::Key::Numpad0 | nrg_platform::Key::Key0 => Some(egui::Key::Num0),
        nrg_platform::Key::Numpad1 | nrg_platform::Key::Key1 => Some(egui::Key::Num1),
        nrg_platform::Key::Numpad2 | nrg_platform::Key::Key2 => Some(egui::Key::Num2),
        nrg_platform::Key::Numpad3 | nrg_platform::Key::Key3 => Some(egui::Key::Num3),
        nrg_platform::Key::Numpad4 | nrg_platform::Key::Key4 => Some(egui::Key::Num4),
        nrg_platform::Key::Numpad5 | nrg_platform::Key::Key5 => Some(egui::Key::Num5),
        nrg_platform::Key::Numpad6 | nrg_platform::Key::Key6 => Some(egui::Key::Num6),
        nrg_platform::Key::Numpad7 | nrg_platform::Key::Key7 => Some(egui::Key::Num7),
        nrg_platform::Key::Numpad8 | nrg_platform::Key::Key8 => Some(egui::Key::Num8),
        nrg_platform::Key::Numpad9 | nrg_platform::Key::Key9 => Some(egui::Key::Num9),
        nrg_platform::Key::A => Some(egui::Key::A),
        nrg_platform::Key::B => Some(egui::Key::B),
        nrg_platform::Key::C => Some(egui::Key::C),
        nrg_platform::Key::D => Some(egui::Key::D),
        nrg_platform::Key::E => Some(egui::Key::E),
        nrg_platform::Key::F => Some(egui::Key::F),
        nrg_platform::Key::G => Some(egui::Key::G),
        nrg_platform::Key::H => Some(egui::Key::H),
        nrg_platform::Key::I => Some(egui::Key::I),
        nrg_platform::Key::J => Some(egui::Key::J),
        nrg_platform::Key::K => Some(egui::Key::K),
        nrg_platform::Key::L => Some(egui::Key::L),
        nrg_platform::Key::M => Some(egui::Key::M),
        nrg_platform::Key::N => Some(egui::Key::N),
        nrg_platform::Key::O => Some(egui::Key::O),
        nrg_platform::Key::P => Some(egui::Key::P),
        nrg_platform::Key::Q => Some(egui::Key::Q),
        nrg_platform::Key::R => Some(egui::Key::R),
        nrg_platform::Key::S => Some(egui::Key::S),
        nrg_platform::Key::T => Some(egui::Key::T),
        nrg_platform::Key::U => Some(egui::Key::U),
        nrg_platform::Key::V => Some(egui::Key::V),
        nrg_platform::Key::W => Some(egui::Key::W),
        nrg_platform::Key::X => Some(egui::Key::X),
        nrg_platform::Key::Y => Some(egui::Key::Y),
        nrg_platform::Key::Z => Some(egui::Key::Z),
        _ => None,
    }
}

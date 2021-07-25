use std::any::TypeId;

use egui::*;
use image::{DynamicImage, Pixel};
use nrg_core::{JobHandlerRw, System, SystemId};
use nrg_graphics::{
    MaterialInstance, MaterialRc, MeshData, MeshInstance, PipelineInstance, TextureInstance,
    VertexData,
};

use nrg_messenger::{read_messages, MessageChannel, MessengerRw};
use nrg_platform::{MouseButton, MouseEvent, MouseState, WindowEvent, DEFAULT_DPI};
use nrg_resources::{DataTypeResource, SharedData, SharedDataRw};

pub struct UISystem {
    id: SystemId,
    shared_data: SharedDataRw,
    job_handler: JobHandlerRw,
    global_messenger: MessengerRw,
    message_channel: MessageChannel,
    ui_context: egui::CtxRef,
    ui_texture_version: u64,
    ui_input: egui::RawInput,
    ui_default_material: MaterialRc,
    ui_theme: u32,
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
            ui_context: egui::CtxRef::default(),
            ui_texture_version: 0,
            ui_input: egui::RawInput::default(),
            ui_default_material: MaterialRc::default(),
            ui_theme: 0,
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
                        modifiers: Default::default(),
                    });
                }
            } else if msg.type_id() == TypeId::of::<WindowEvent>() {
                let event = msg.as_any().downcast_ref::<WindowEvent>().unwrap();
                match *event {
                    WindowEvent::SizeChanged(width, height) => {
                        self.ui_input.screen_rect = Some(Rect::from_min_size(
                            Default::default(),
                            vec2(width as f32, height as f32),
                        ));
                    }
                    WindowEvent::DpiChanged(x, _y) => {
                        self.ui_input.pixels_per_point = Some(x / DEFAULT_DPI);
                    }
                    _ => {}
                }
            }
        });
        self
    }

    fn draw_ui(&mut self) {
        let mut theme = self.ui_theme;
        SidePanel::left("SidePanel").show(&self.ui_context, |ui| {
            ui.heading("Hello");
            ui.label("Ciao GENTS!");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Theme");
                let id = ui.make_persistent_id("theme_combo_box_side");
                ComboBox::from_id_source(id)
                    .selected_text((if theme == 0 { "Dark" } else { "Light" }).to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut theme, 0, "Dark");
                        ui.selectable_value(&mut theme, 1, "Light");
                    });
            });
            ui.separator();
            ui.hyperlink("https://github.com/emilk/egui");
            ui.separator();
        });

        egui::Window::new("Test")
            .scroll(true)
            .title_bar(true)
            .resizable(true)
            .show(&self.ui_context, |ui| {
                ui.heading("My egui Application");
                self.ui_input.ui(ui);
            });

        if self.ui_theme != theme {
            self.ui_theme = theme;
            if self.ui_theme == 0 {
                self.ui_context.set_visuals(Visuals::dark());
            } else {
                self.ui_context.set_visuals(Visuals::light());
            }
        }
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
            .register_messagebox::<MouseEvent>(self.message_channel.get_messagebox());

        self.create_default_material();
    }

    fn run(&mut self) -> bool {
        self.update_events();

        self.ui_context.begin_frame(self.ui_input.take());

        self.draw_ui();

        let (_, shapes) = self.ui_context.end_frame();
        let clipped_meshes = self.ui_context.tessellate(shapes);

        let mesh_data = self.compute_mesh_data(clipped_meshes);
        self.update_mesh_in_ui_material(mesh_data)
            .update_egui_texture();

        true
    }

    fn uninit(&mut self) {
        self.global_messenger
            .write()
            .unwrap()
            .unregister_messagebox::<MouseEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<WindowEvent>(self.message_channel.get_messagebox());
    }
}

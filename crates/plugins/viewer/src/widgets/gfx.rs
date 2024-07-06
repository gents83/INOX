use inox_core::ContextRc;
use inox_render::{
    DrawIndexedCommand, GPUInstance, GPUMesh, GPUMeshlet, GPUVertexIndices, GPUVertexPosition,
    RenderContextRc,
};

use inox_messenger::MessageHubRc;
use inox_resources::{Resource, SharedDataRc};
use inox_ui::{implement_widget_data, UIWidget, Widget, Window};

#[derive(Clone)]
struct GfxData {
    render_context: RenderContextRc,
    vertices_count: usize,
    indices_count: usize,
    meshes_count: usize,
    meshlets_count: usize,
    instance_count: usize,
    commands_instance_count: usize,
    passes: Vec<(String, bool)>,
}
implement_widget_data!(GfxData);

#[derive(Clone)]
pub struct Gfx {
    ui_page: Resource<UIWidget>,
}

impl Gfx {
    pub fn new(context: &ContextRc, render_context: &RenderContextRc) -> Self {
        let data = GfxData {
            vertices_count: 0,
            indices_count: 0,
            meshes_count: 0,
            meshlets_count: 0,
            instance_count: 0,
            commands_instance_count: 0,
            passes: Vec::new(),
            render_context: render_context.clone(),
        };
        Self {
            ui_page: Self::create(context.shared_data(), context.message_hub(), data),
        }
    }

    pub fn update(&mut self) {
        inox_profiler::scoped_profile!("Gfx::update");
        if let Some(data) = self.ui_page.get_mut().data_mut::<GfxData>() {
            {
                data.vertices_count = data
                    .render_context
                    .global_buffers()
                    .buffer::<GPUVertexPosition>()
                    .read()
                    .unwrap()
                    .item_count();
                data.indices_count = data
                    .render_context
                    .global_buffers()
                    .buffer::<GPUVertexIndices>()
                    .read()
                    .unwrap()
                    .item_count();
                data.meshes_count = data
                    .render_context
                    .global_buffers()
                    .buffer::<GPUMesh>()
                    .read()
                    .unwrap()
                    .item_count();
                data.meshlets_count = data
                    .render_context
                    .global_buffers()
                    .buffer::<GPUMeshlet>()
                    .read()
                    .unwrap()
                    .item_count();
                data.instance_count = data
                    .render_context
                    .global_buffers()
                    .vector::<GPUInstance>()
                    .read()
                    .unwrap()
                    .len();
                let commands = data
                    .render_context
                    .global_buffers()
                    .vector::<DrawIndexedCommand>();
                let commands = commands.read().unwrap();
                data.commands_instance_count = 0;
                commands.iter().for_each(|c| {
                    data.commands_instance_count += c.instance_count as usize;
                });
            }

            if data.passes.is_empty() {
                for i in 0..data.render_context.num_passes() {
                    let name = data.render_context.pass_name(i);
                    let is_enabled = data.render_context.is_pass_enabled(i);
                    data.passes.push((name, is_enabled));
                }
            } else {
                data.passes.iter().enumerate().for_each(|(i, (_n, b))| {
                    data.render_context.set_pass_enabled(i, *b);
                });
            }
        }
    }

    fn create(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        data: GfxData,
    ) -> Resource<UIWidget> {
        UIWidget::register(shared_data, message_hub, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any_mut().downcast_mut::<GfxData>() {
                if let Some(response) = Window::new("Graphics")
                    .vscroll(true)
                    .title_bar(true)
                    .resizable(true)
                    .show(ui_context, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Total vertices: ");
                                inox_ui::DragValue::new(&mut data.vertices_count)
                                    .speed(0)
                                    .ui(ui);
                            });
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Total triangles: ");
                                let mut triangles_count = data.indices_count / 3;
                                inox_ui::DragValue::new(&mut triangles_count)
                                    .speed(0)
                                    .ui(ui);
                            });
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Total meshes: ");
                                inox_ui::DragValue::new(&mut data.meshes_count)
                                    .speed(0)
                                    .ui(ui);
                            });
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Total meshlets: ");
                                inox_ui::DragValue::new(&mut data.meshlets_count)
                                    .speed(0)
                                    .ui(ui);
                            });
                        });
                        ui.separator();
                        ui.horizontal_wrapped(|ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Total instances: ");
                                inox_ui::DragValue::new(&mut data.instance_count)
                                    .speed(0)
                                    .ui(ui);
                            });
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Draw instances: ");
                                inox_ui::DragValue::new(&mut data.commands_instance_count)
                                    .speed(0)
                                    .ui(ui);
                            });
                        });
                        data.passes.iter_mut().for_each(|(name, is_enabled)| {
                            ui.checkbox(is_enabled, name.as_str());
                        });
                    })
                {
                    return response.response.is_pointer_button_down_on();
                }
            }
            false
        })
    }
}

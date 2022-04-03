use std::{collections::HashMap, ops::Range};

use inox_core::ContextRc;
use inox_graphics::{GraphicsMesh, Mesh, MeshId, Pipeline, PipelineId, GRAPHIC_MESH_UID};

use inox_messenger::MessageHubRc;
use inox_resources::{Resource, SerializableResource, SharedDataRc};
use inox_ui::{
    epaint::color::Hsva,
    implement_widget_data,
    plot::{Bar, BarChart, Plot},
    Color32, UIWidget, Vec2, Widget, Window,
};

struct MeshesData {
    shared_data: SharedDataRc,
    vertices_count: usize,
    indices_count: usize,
    meshes_names: HashMap<MeshId, (String, Range<usize>, Color32)>,
    pipeline_instances: HashMap<PipelineId, (String, usize, Vec<String>)>,
}
implement_widget_data!(MeshesData);

pub struct Meshes {
    ui_page: Resource<UIWidget>,
}

impl Meshes {
    pub fn new(context: &ContextRc) -> Self {
        let data = MeshesData {
            shared_data: context.shared_data().clone(),
            vertices_count: 0,
            indices_count: 0,
            meshes_names: HashMap::new(),
            pipeline_instances: HashMap::new(),
        };
        Self {
            ui_page: Self::create(context.shared_data(), context.message_hub(), data),
        }
    }

    pub fn update(&mut self) {
        inox_profiler::scoped_profile!("Meshes::update");

        if let Some(data) = self.ui_page.get_mut().data_mut::<MeshesData>() {
            if let Some(graphics_mesh) = data
                .shared_data
                .get_resource::<GraphicsMesh>(&GRAPHIC_MESH_UID)
            {
                data.vertices_count = graphics_mesh.get().vertex_count();
                data.indices_count = graphics_mesh.get().index_count();

                let golden_ratio = (5.0_f32.sqrt() - 1.0) / 2.0; // 0.61803398875
                let mut valid_meshes = Vec::new();
                graphics_mesh
                    .get()
                    .for_each_vertex_buffer_data(|mesh_id: &MeshId, range| {
                        let range = range.clone();
                        if let Some(entry) = data.meshes_names.get_mut(mesh_id) {
                            entry.1 = range;
                        } else {
                            let h = valid_meshes.len() as f32 * golden_ratio;
                            data.meshes_names.insert(
                                *mesh_id,
                                (
                                    "EMPTY".to_string(),
                                    range,
                                    Hsva::new(h, 0.85, 0.5, 1.0).into(),
                                ),
                            );
                        }
                        valid_meshes.push(*mesh_id);
                    });
                data.shared_data.for_each_resource(|handle, mesh: &Mesh| {
                    let name = mesh
                        .path()
                        .file_stem()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                        .to_string();
                    if let Some(entry) = data.meshes_names.get_mut(handle.id()) {
                        entry.0 = name;
                    } else {
                        let h = valid_meshes.len() as f32 * golden_ratio;
                        data.meshes_names.insert(
                            *handle.id(),
                            (
                                name,
                                mesh.vertices_range().clone(),
                                Hsva::new(h, 0.85, 0.5, 1.0).into(),
                            ),
                        );
                    }
                });

                data.pipeline_instances.clear();
                data.shared_data
                    .for_each_resource(|handle, pipeline: &Pipeline| {
                        let instances_count = graphics_mesh.get().instance_count(handle.id());

                        let mut instance_names = Vec::new();
                        graphics_mesh.get().for_each_instance(
                            handle.id(),
                            |mesh_id, _, _, _, _| {
                                let name = if let Some(entry) = data.meshes_names.get(mesh_id) {
                                    entry.0.clone()
                                } else {
                                    "EMPTY".to_string()
                                };
                                instance_names.push(name);
                            },
                        );
                        let name = pipeline
                            .path()
                            .file_stem()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                            .to_string();
                        data.pipeline_instances.insert(
                            *handle.id(),
                            (
                                format!("[{:?}] {}", handle.id(), name),
                                instances_count,
                                instance_names,
                            ),
                        );
                    });

                data.meshes_names.retain(|id, _| valid_meshes.contains(id));
            }
        }
    }

    fn create(
        shared_data: &SharedDataRc,
        message_hub: &MessageHubRc,
        data: MeshesData,
    ) -> Resource<UIWidget> {
        UIWidget::register(shared_data, message_hub, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any_mut().downcast_mut::<MeshesData>() {
                Window::new("Meshes")
                    .vscroll(true)
                    .title_bar(true)
                    .resizable(true)
                    .show(ui_context, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Total vertices: ");
                            inox_ui::DragValue::new(&mut data.vertices_count)
                                .speed(0)
                                .ui(ui);
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Total indices: ");
                            inox_ui::DragValue::new(&mut data.indices_count)
                                .speed(0)
                                .ui(ui);
                        });
                        ui.separator();

                        let bar_height = 64.;
                        let mut bar_charts = Vec::new();

                        data.meshes_names
                            .iter()
                            .for_each(|(_, (mesh_name, range, color))| {
                                let bars = vec![Bar::new(
                                    0., //vertices_range.start as _,
                                    range.len() as _,
                                )
                                .name(format!("{} - [{},{}]", mesh_name, range.start, range.end))
                                .base_offset(range.start as _)];
                                let bar_chart = BarChart::new(bars)
                                    .width(bar_height)
                                    .color(*color)
                                    .horizontal();
                                bar_charts.push(bar_chart);
                            });

                        ui.label("Vertex Buffer: ");
                        Plot::new("Vertex Buffer Plot")
                            .include_x(data.vertices_count as f32)
                            .include_y(bar_height)
                            .height(bar_height as _)
                            .set_margin_fraction(Vec2::new(0., 0.))
                            .show_background(false)
                            .show_axes([false, false])
                            .allow_boxed_zoom(false)
                            .allow_drag(false)
                            .allow_zoom(false)
                            .show(ui, |ui| {
                                bar_charts.into_iter().for_each(|bar_chart| {
                                    ui.bar_chart(bar_chart);
                                });
                            });
                        ui.separator();

                        data.pipeline_instances.iter_mut().for_each(
                            |(_id, (name, count, meshes))| {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(format!("Pipeline: {:?} - Total instances:", name));
                                    inox_ui::DragValue::new(count).speed(0).ui(ui);
                                });

                                ui.vertical(|ui| {
                                    meshes.iter().enumerate().for_each(|(index, mesh_name)| {
                                        ui.label(format!("[{}]: {:?}", index, mesh_name));
                                    });
                                });
                            },
                        );
                    });
            }
        })
    }
}

use std::collections::HashMap;

use inox_core::ContextRc;
use inox_graphics::{GraphicsMesh, Mesh, MeshId, Pipeline, GRAPHIC_MESH_UID};
use inox_messenger::MessageHubRc;
use inox_resources::{Resource, SerializableResource, SharedDataRc};
use inox_ui::{
    implement_widget_data,
    plot::{Bar, BarChart, Plot},
    UIWidget, Vec2, Widget, Window,
};

struct MeshesData {
    shared_data: SharedDataRc,
}
implement_widget_data!(MeshesData);

pub struct Meshes {
    ui_page: Resource<UIWidget>,
}

impl Meshes {
    pub fn new(context: &ContextRc) -> Self {
        let data = MeshesData {
            shared_data: context.shared_data().clone(),
        };
        Self {
            ui_page: Self::create(context.shared_data(), context.message_hub(), data),
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
                        if let Some(graphics_mesh) = data
                            .shared_data
                            .get_resource::<GraphicsMesh>(&GRAPHIC_MESH_UID)
                        {
                            let vertex_count = graphics_mesh.get().vertex_count();
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Total vertices: ");
                                let mut vertices_count = vertex_count;
                                let display_value =
                                    inox_ui::DragValue::new(&mut vertices_count).speed(0);
                                display_value.ui(ui);
                            });
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Total indices: ");
                                let mut indices_count = graphics_mesh.get().index_count();
                                let display_value =
                                    inox_ui::DragValue::new(&mut indices_count).speed(0);
                                display_value.ui(ui);
                            });
                            ui.separator();

                            data.shared_data
                                .for_each_resource(|handle, pipeline: &Pipeline| {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(format!(
                                            "Pipeline: {:?} - Total instances:",
                                            pipeline.data().identifier
                                        ));
                                        let mut instances_count =
                                            graphics_mesh.get().instance_count(handle.id());
                                        let display_value =
                                            inox_ui::DragValue::new(&mut instances_count).speed(0);
                                        display_value.ui(ui);
                                    });
                                });
                            ui.separator();

                            let mut all_meshes = HashMap::new();
                            data.shared_data.for_each_resource(|handle, mesh: &Mesh| {
                                all_meshes.insert(
                                    *handle.id(),
                                    mesh.path()
                                        .file_stem()
                                        .unwrap_or_default()
                                        .to_str()
                                        .unwrap_or_default()
                                        .to_string(),
                                );
                            });

                            let bar_height = 64.;
                            let mut bar_charts = Vec::new();
                            graphics_mesh.get().for_each_vertex_buffer_data(
                                |mesh_id: &MeshId, range| {
                                    let mesh_name = if let Some(name) = all_meshes.get(mesh_id) {
                                        name
                                    } else {
                                        "EMPTY"
                                    };
                                    let bars = vec![Bar::new(
                                        0., //vertices_range.start as _,
                                        range.len() as _,
                                    )
                                    .name(format!(
                                        "{} - [{},{}]",
                                        mesh_name, range.start, range.end
                                    ))
                                    .base_offset(range.start as _)];
                                    let bar_chart =
                                        BarChart::new(bars).width(bar_height).horizontal();
                                    bar_charts.push(bar_chart);
                                },
                            );

                            ui.label("Vertex Buffer: ");
                            Plot::new("Vertex Buffer Plot")
                                .include_x(vertex_count as f32)
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

                            ui.label("Instances: ");
                        }
                    });
            }
        })
    }
}

use inox_core::ContextRc;
use inox_graphics::RendererRw;

use inox_messenger::MessageHubRc;
use inox_resources::{Resource, SharedDataRc};
use inox_ui::{implement_widget_data, UIWidget, Widget, Window};

#[derive(Clone)]
struct GfxData {
    vertices_count: usize,
    indices_count: usize,
    meshes_count: usize,
    meshlets_count: usize,
    passes: Vec<(String, bool)>,
}
implement_widget_data!(GfxData);

#[derive(Clone)]
pub struct Gfx {
    ui_page: Resource<UIWidget>,
    renderer: RendererRw,
}

impl Gfx {
    pub fn new(context: &ContextRc, renderer: &RendererRw) -> Self {
        let data = GfxData {
            vertices_count: 0,
            indices_count: 0,
            meshes_count: 0,
            meshlets_count: 0,
            passes: Vec::new(),
        };
        Self {
            ui_page: Self::create(context.shared_data(), context.message_hub(), data),
            renderer: renderer.clone(),
        }
    }

    pub fn update(&mut self) {
        inox_profiler::scoped_profile!("Gfx::update");
        if let Some(data) = self.ui_page.get_mut().data_mut::<GfxData>() {
            {
                let renderer = self.renderer.read().unwrap();
                let render_context = renderer.render_context();
                data.vertices_count = render_context
                    .render_buffers
                    .vertex_positions
                    .read()
                    .unwrap()
                    .item_count();
                data.indices_count = render_context
                    .render_buffers
                    .indices
                    .read()
                    .unwrap()
                    .item_count();
                data.meshes_count = render_context
                    .render_buffers
                    .meshes
                    .read()
                    .unwrap()
                    .item_count();
                data.meshlets_count = render_context
                    .render_buffers
                    .meshlets
                    .read()
                    .unwrap()
                    .item_count();
            }

            if data.passes.is_empty() {
                let renderer = self.renderer.read().unwrap();
                for i in 0..renderer.num_passes() {
                    let name = renderer.pass_at(i).unwrap().name().to_string();
                    let is_enabled = renderer.is_pass_enabled(i);
                    data.passes.push((name, is_enabled));
                }
            } else {
                let mut renderer = self.renderer.write().unwrap();
                data.passes.iter().enumerate().for_each(|(i, (_n, b))| {
                    renderer.set_pass_enabled(i, *b);
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

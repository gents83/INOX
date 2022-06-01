use inox_core::ContextRc;
use inox_graphics::{CullingPass, DrawEvent, RendererRw};
use inox_math::{compute_frustum, Frustum, Mat4Ops, MatBase, Matrix4};
use inox_resources::Resource;
use inox_scene::{Camera, SceneId};
use inox_ui::{implement_widget_data, UIWidget, Window};

use super::{Hierarchy, Meshes};

pub struct InfoParams {
    pub is_active: bool,
    pub renderer: RendererRw,
    pub scene_id: SceneId,
}

struct Data {
    context: ContextRc,
    renderer: RendererRw,
    params: InfoParams,
    hierarchy: (bool, Option<Hierarchy>),
    meshes: (bool, Option<Meshes>),
    show_frustum: bool,
    freeze_culling_camera: bool,
    fps: u32,
    dt: u128,
    view: Matrix4,
    proj: Matrix4,
}
implement_widget_data!(Data);

pub struct Info {
    ui_page: Resource<UIWidget>,
}

impl Info {
    pub fn new(context: &ContextRc, params: InfoParams) -> Self {
        let data = Data {
            context: context.clone(),
            renderer: params.renderer.clone(),
            params,
            hierarchy: (false, None),
            meshes: (false, None),
            show_frustum: false,
            freeze_culling_camera: false,
            fps: 0,
            dt: 0,
            view: Matrix4::default_identity(),
            proj: Matrix4::default_identity(),
        };
        Self {
            ui_page: Self::create(data),
        }
    }

    pub fn is_active(&self) -> bool {
        if let Some(data) = self.ui_page.get().data::<Data>() {
            return data.params.is_active;
        }
        false
    }
    pub fn set_active(&self, is_active: bool) {
        if let Some(data) = self.ui_page.get_mut().data_mut::<Data>() {
            data.params.is_active = is_active;
        }
    }
    pub fn set_scene_id(&self, scene_id: &SceneId) {
        if let Some(data) = self.ui_page.get_mut().data_mut::<Data>() {
            data.params.scene_id = *scene_id;
        }
    }

    pub fn update(&mut self) {
        inox_profiler::scoped_profile!("Info::update");

        if let Some(data) = self.ui_page.get_mut().data_mut::<Data>() {
            data.fps = data.context.global_timer().fps();
            data.dt = data.context.global_timer().dt().as_millis();

            if data.hierarchy.0 && data.hierarchy.1.is_none() {
                data.hierarchy.1 = Hierarchy::new(
                    data.context.shared_data(),
                    data.context.message_hub(),
                    &data.params.scene_id,
                );
            } else if !data.hierarchy.0 && data.hierarchy.1.is_some() {
                data.hierarchy.1 = None;
            }

            if data.meshes.0 && data.meshes.1.is_none() {
                data.meshes.1 = Some(Meshes::new(&data.context));
            } else if data.meshes.1.is_some() {
                if !data.meshes.0 {
                    data.meshes.1 = None;
                } else {
                    data.meshes.1.as_mut().unwrap().update();
                }
            }

            if !data.freeze_culling_camera {
                if let Some(camera) = data
                    .context
                    .shared_data()
                    .match_resource(|c: &Camera| c.is_active())
                {
                    data.view = camera.get().view_matrix();
                    data.proj = camera.get().proj_matrix();
                }
            }
            let frustum = compute_frustum(&data.view, &data.proj);
            if let Some(culling_pass) = data.renderer.write().unwrap().pass_mut::<CullingPass>() {
                culling_pass.set_camera_data(data.view.translation(), &frustum);
            }
            if data.show_frustum {
                Self::show_frustum(data, &frustum);
            }
        }
    }

    fn show_frustum(data: &Data, frustum: &Frustum) {
        let color = [1., 1., 0., 1.];

        //NearPlane
        data.context.message_hub().send_event(DrawEvent::Line(
            frustum.ntr,
            frustum.ntl,
            color.into(),
        ));
        data.context.message_hub().send_event(DrawEvent::Line(
            frustum.ntr,
            frustum.nbr,
            color.into(),
        ));
        data.context.message_hub().send_event(DrawEvent::Line(
            frustum.ntl,
            frustum.nbl,
            color.into(),
        ));
        data.context.message_hub().send_event(DrawEvent::Line(
            frustum.nbr,
            frustum.nbl,
            color.into(),
        ));

        //FarPlane
        data.context.message_hub().send_event(DrawEvent::Line(
            frustum.ftr,
            frustum.ftl,
            color.into(),
        ));
        data.context.message_hub().send_event(DrawEvent::Line(
            frustum.ftr,
            frustum.fbr,
            color.into(),
        ));
        data.context.message_hub().send_event(DrawEvent::Line(
            frustum.ftl,
            frustum.fbl,
            color.into(),
        ));
        data.context.message_hub().send_event(DrawEvent::Line(
            frustum.fbr,
            frustum.fbl,
            color.into(),
        ));

        //LeftPlane
        data.context.message_hub().send_event(DrawEvent::Line(
            frustum.ftl,
            frustum.ntl,
            color.into(),
        ));
        data.context.message_hub().send_event(DrawEvent::Line(
            frustum.fbl,
            frustum.nbl,
            color.into(),
        ));

        //RightPlane
        data.context.message_hub().send_event(DrawEvent::Line(
            frustum.ftr,
            frustum.ntr,
            color.into(),
        ));
        data.context.message_hub().send_event(DrawEvent::Line(
            frustum.fbr,
            frustum.nbr,
            color.into(),
        ));
    }

    fn create(data: Data) -> Resource<UIWidget> {
        let shared_data = data.context.shared_data().clone();
        let message_hub = data.context.message_hub().clone();
        UIWidget::register(&shared_data, &message_hub, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any_mut().downcast_mut::<Data>() {
                if !data.params.is_active {
                    return;
                }
                Window::new("Debug")
                    .vscroll(true)
                    .title_bar(true)
                    .resizable(true)
                    .show(ui_context, |ui| {
                        ui.label(format!("FPS: {} - ms: {:?}", data.fps, data.dt));
                        ui.checkbox(&mut data.hierarchy.0, "Hierarchy");
                        ui.checkbox(&mut data.meshes.0, "Meshes");
                        ui.checkbox(&mut data.show_frustum, "Show Frustum");
                        ui.checkbox(&mut data.freeze_culling_camera, "Freeze Culling Camera");
                    });
            }
        })
    }
}

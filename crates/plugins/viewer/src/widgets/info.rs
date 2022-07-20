use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use inox_core::ContextRc;
use inox_graphics::{
    DrawEvent, MeshFlags, RendererRw, CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS,
    CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX, CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_SPHERE,
};
use inox_math::{compute_frustum, Degrees, Frustum, Mat4Ops, MatBase, Matrix4, NewAngle};
use inox_resources::Resource;
use inox_scene::{Camera, SceneId};
use inox_ui::{implement_widget_data, ComboBox, UIWidget, Window};

use super::{Hierarchy, Meshes};

pub struct InfoParams {
    pub is_active: bool,
    pub scene_id: SceneId,
    pub renderer: RendererRw,
    pub update_culling_camera: Arc<AtomicBool>,
}

#[derive(Default, Debug, PartialEq, Eq)]
enum MeshletDebug {
    #[default]
    None,
    Color,
    Sphere,
    BoundingBox,
}

struct Data {
    context: ContextRc,
    params: InfoParams,
    hierarchy: (bool, Option<Hierarchy>),
    meshes: (bool, Option<Meshes>),
    show_frustum: bool,
    freeze_culling_camera: bool,
    meshlet_debug: MeshletDebug,
    fps: u32,
    dt: u128,
    cam_matrix: Matrix4,
    near: f32,
    far: f32,
    fov: Degrees,
    aspect_ratio: f32,
}
implement_widget_data!(Data);

pub struct Info {
    ui_page: Resource<UIWidget>,
}

impl Info {
    pub fn new(context: &ContextRc, params: InfoParams) -> Self {
        let data = Data {
            context: context.clone(),
            params,
            hierarchy: (false, None),
            meshes: (false, None),
            show_frustum: false,
            freeze_culling_camera: false,
            meshlet_debug: MeshletDebug::None,
            fps: 0,
            dt: 0,
            cam_matrix: Matrix4::default_identity(),
            near: 0.,
            far: 0.,
            fov: Degrees::new(0.),
            aspect_ratio: 1.,
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
            data.params
                .update_culling_camera
                .store(!data.freeze_culling_camera, Ordering::SeqCst);
            if data.show_frustum {
                if !data.freeze_culling_camera {
                    if let Some(camera) = data
                        .context
                        .shared_data()
                        .match_resource(|c: &Camera| c.is_active())
                    {
                        let c = camera.get();
                        data.near = c.near_plane();
                        data.far = c.far_plane();
                        data.cam_matrix = c.transform();
                        data.fov = c.fov_in_degrees();
                        data.aspect_ratio = c.aspect_ratio();
                    }
                }
                let frustum = compute_frustum(
                    &data.cam_matrix,
                    data.near,
                    data.far,
                    data.fov,
                    data.aspect_ratio,
                );
                Self::show_frustum(data, &frustum);
            }
            if data.meshlet_debug == MeshletDebug::Sphere {
                Self::show_meshlets_sphere(data);
            }
        }
    }

    fn show_meshlets_sphere(data: &mut Data) {
        let renderer = data.params.renderer.read().unwrap();
        let render_context = renderer.render_context().read().unwrap();

        let mesh_flags: u32 = (MeshFlags::Visible | MeshFlags::Opaque).into();
        render_context
            .render_buffers
            .meshes
            .for_each_entry(|_i, mesh| {
                if mesh.mesh_flags == mesh_flags {
                    let matrix = Matrix4::from(mesh.matrix);
                    let scale = matrix.scale().z;
                    let meshlets = render_context.render_buffers.meshlets.data();
                    for i in mesh.meshlet_offset..mesh.meshlet_offset + mesh.meshlet_count {
                        let meshlet = &meshlets[i as usize];
                        let p = matrix.transform(
                            [
                                meshlet.center_radius[0],
                                meshlet.center_radius[1],
                                meshlet.center_radius[2],
                            ]
                            .into(),
                        );
                        let r = meshlet.center_radius[3] * scale;
                        data.context.message_hub().send_event(DrawEvent::Circle(
                            p,
                            r,
                            [1.0, 1.0, 0.0, 1.0].into(),
                            true,
                        ));
                    }
                }
            });
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
                    return false;
                }
                if let Some(response) = Window::new("Debug")
                    .vscroll(true)
                    .title_bar(true)
                    .resizable(true)
                    .show(ui_context, |ui| {
                        ui.label(format!("FPS: {} - ms: {:?}", data.fps, data.dt));
                        ui.checkbox(&mut data.hierarchy.0, "Hierarchy");
                        ui.checkbox(&mut data.meshes.0, "Meshes");
                        ui.checkbox(&mut data.show_frustum, "Show Frustum");
                        ui.checkbox(&mut data.freeze_culling_camera, "Freeze Culling Camera");
                        ui.horizontal(|ui| {
                            ui.label("Show Meshlets");
                            let combo_box = ComboBox::from_id_source("Meshlet Debug")
                                .selected_text(format!("{:?}", data.meshlet_debug))
                                .show_ui(ui, |ui| {
                                    let mut is_changed = false;
                                    is_changed |= ui
                                        .selectable_value(
                                            &mut data.meshlet_debug,
                                            MeshletDebug::None,
                                            "None",
                                        )
                                        .changed();
                                    is_changed |= ui
                                        .selectable_value(
                                            &mut data.meshlet_debug,
                                            MeshletDebug::Color,
                                            "Color",
                                        )
                                        .changed();
                                    is_changed |= ui
                                        .selectable_value(
                                            &mut data.meshlet_debug,
                                            MeshletDebug::Sphere,
                                            "Sphere",
                                        )
                                        .changed();
                                    is_changed |= ui
                                        .selectable_value(
                                            &mut data.meshlet_debug,
                                            MeshletDebug::BoundingBox,
                                            "Bounding Box",
                                        )
                                        .changed();
                                    is_changed
                                });
                            if let Some(is_changed) = combo_box.inner {
                                if is_changed {

                                    let renderer = data.params.renderer.read().unwrap();
                                    let mut render_context = renderer.render_context().write().unwrap();
                                    match &data.meshlet_debug {
                                        MeshletDebug::None => {
                                            render_context
                                                .constant_data
                                                .remove_flag(CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS)
                                                .remove_flag(
                                                    CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_SPHERE,
                                                )
                                                .remove_flag(
                                                    CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX,
                                                );
                                        }
                                        MeshletDebug::Color => {
                                            render_context
                                                .constant_data
                                                .remove_flag(
                                                    CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_SPHERE,
                                                )
                                                .remove_flag(
                                                    CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX,
                                                )
                                                .add_flag(CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS);
                                        }
                                        MeshletDebug::Sphere => {
                                            render_context
                                                .constant_data
                                                .remove_flag(
                                                    CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX,
                                                )
                                                .add_flag(CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS)
                                                .add_flag(CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_SPHERE);
                                        }
                                        MeshletDebug::BoundingBox => {
                                            render_context
                                                .constant_data
                                                .remove_flag(
                                                    CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_SPHERE,
                                                )
                                                .add_flag(CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS)
                                                .add_flag(
                                                    CONSTANT_DATA_FLAGS_DISPLAY_MESHLETS_BOUNDING_BOX,
                                                );
                                        }
                                    }
                                }
                            }
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

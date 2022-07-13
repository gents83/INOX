use std::{any::type_name, path::PathBuf};

use crate::{
    create_arrow, create_colored_quad, create_line, create_sphere, DrawEvent, Material,
    MaterialData, Mesh, MeshData, MeshFlags, RenderPipeline,
};

use inox_core::{ContextRc, System, SystemId, SystemUID};
use inox_messenger::{Listener, MessageHubRc};
use inox_resources::{
    ConfigBase, ConfigEvent, DataTypeResource, Handle, Resource, SerializableResource, SharedDataRc,
};
use inox_serialize::read_from_file;
use inox_uid::{generate_random_uid, generate_uid_from_string};

use super::config::Config;

/// A debug drawer
/// You can use this to draw things in the editor just sending events:
/// ```
/// use inox_editor::systems::{DebugDrawer, DrawEvent};
/// use inox_math::{Vector3, Zero};
/// use inox_messenger::{MessengerRw, Message};
///
/// let message_hub = MessengerRw::default();
/// let global_dispatcher = message_hub.read().unwrap().get_dispatcher().clone();
///     global_dispatcher
///     .write()
///     .unwrap()
///     .send(
///         DrawEvent::Sphere([2., 2., 2.].into(), 2., [1., 0., 0., 1.].into(), true)
///             .as_boxed(),
///     )
///     .ok();
///
///     global_dispatcher
///     .write()
///     .unwrap()
///     .send(
///         DrawEvent::Arrow(
///             Vector3::zero(),
///             [2., 2., 0.].into(),
///             [1., 0., 0., 1.].into(),
///             false,
///         )
///         .as_boxed(),
///     )
///     .ok();
/// ```

pub struct DebugDrawerSystem {
    config: Config,
    mesh_instance: Resource<Mesh>,
    wireframe_mesh_instance: Resource<Mesh>,
    default_pipeline: Handle<RenderPipeline>,
    wireframe_pipeline: Handle<RenderPipeline>,
    listener: Listener,
    shared_data: SharedDataRc,
    message_hub: MessageHubRc,
}

impl DebugDrawerSystem {
    pub fn new(context: &ContextRc) -> Self {
        let mesh_instance = Mesh::new_resource(
            context.shared_data(),
            context.message_hub(),
            generate_random_uid(),
            MeshData::default(),
            None,
        );
        mesh_instance
            .get_mut()
            .set_path(PathBuf::from("DebugDrawerMesh.debugdrawer").as_path())
            .set_flags(MeshFlags::Visible | MeshFlags::Opaque);
        //println!("DebugDrawerMesh {:?}", mesh_instance.id());
        let wireframe_mesh_instance = Mesh::new_resource(
            context.shared_data(),
            context.message_hub(),
            generate_random_uid(),
            MeshData::default(),
            None,
        );
        wireframe_mesh_instance
            .get_mut()
            .set_path(PathBuf::from("DebugDrawerWireframe.debugdrawer").as_path())
            .set_flags(MeshFlags::Visible | MeshFlags::Wireframe);
        //println!("DebugDrawerWireframeMesh {:?}", wireframe_mesh_instance.id());

        let listener = Listener::new(context.message_hub());
        listener.register::<DrawEvent>();

        Self {
            config: Config::default(),
            mesh_instance,
            wireframe_mesh_instance,
            default_pipeline: None,
            wireframe_pipeline: None,
            listener,
            shared_data: context.shared_data().clone(),
            message_hub: context.message_hub().clone(),
        }
    }

    fn auto_send_event(&self, event: DrawEvent) {
        self.message_hub.send_event(event);
    }

    fn update_events(&mut self) {
        inox_profiler::scoped_profile!("DebugDrawerSystem::update_events");

        let mut opaque_mesh_data = MeshData::default();
        let mut wireframe_mesh_data = MeshData::default();

        self.listener
            .process_messages(|e: &ConfigEvent<Config>| match e {
                ConfigEvent::Loaded(filename, config) => {
                    inox_profiler::scoped_profile!("Processing ConfigEvent");
                    if filename == self.config.get_filename() {
                        self.config = config.clone();

                        let default_pipeline = RenderPipeline::request_load(
                            &self.shared_data,
                            &self.message_hub,
                            self.config.default_pipeline.as_path(),
                            None,
                        );
                        let wireframe_pipeline = RenderPipeline::request_load(
                            &self.shared_data,
                            &self.message_hub,
                            self.config.wireframe_pipeline.as_path(),
                            None,
                        );
                        let material = Material::new_resource(
                            &self.shared_data,
                            &self.message_hub,
                            generate_random_uid(),
                            MaterialData::default(),
                            None,
                        );
                        self.mesh_instance.get_mut().set_material(material);
                        let wireframe_material = Material::new_resource(
                            &self.shared_data,
                            &self.message_hub,
                            generate_random_uid(),
                            MaterialData::default(),
                            None,
                        );
                        self.wireframe_mesh_instance
                            .get_mut()
                            .set_material(wireframe_material);
                        self.default_pipeline = Some(default_pipeline);
                        self.wireframe_pipeline = Some(wireframe_pipeline);
                    }
                }
            })
            .process_messages(|event: &DrawEvent| match *event {
                DrawEvent::Line(start, end, color) => {
                    inox_profiler::scoped_profile!("DrawEvent::Line");

                    let mesh_data = create_line(start, end, color);
                    wireframe_mesh_data.append_mesh_data_as_meshlet(mesh_data);
                }
                DrawEvent::BoundingBox(min, max, color) => {
                    inox_profiler::scoped_profile!("DrawEvent::BoundingBox");

                    self.auto_send_event(DrawEvent::Quad(
                        [min.x, min.y].into(),
                        [max.x, max.y].into(),
                        min.z,
                        color,
                        true,
                    ));
                    self.auto_send_event(DrawEvent::Quad(
                        [min.x, min.y].into(),
                        [max.x, max.y].into(),
                        max.z,
                        color,
                        true,
                    ));
                    self.auto_send_event(DrawEvent::Line(
                        [min.x, min.y, min.z].into(),
                        [min.x, min.y, max.z].into(),
                        color,
                    ));
                    self.auto_send_event(DrawEvent::Line(
                        [min.x, max.y, min.z].into(),
                        [min.x, max.y, max.z].into(),
                        color,
                    ));
                    self.auto_send_event(DrawEvent::Line(
                        [max.x, min.y, min.z].into(),
                        [max.x, min.y, max.z].into(),
                        color,
                    ));
                    self.auto_send_event(DrawEvent::Line(
                        [max.x, max.y, min.z].into(),
                        [max.x, max.y, max.z].into(),
                        color,
                    ));
                }
                DrawEvent::Quad(min, max, z, color, is_wireframe) => {
                    inox_profiler::scoped_profile!("DrawEvent::Quad");

                    if is_wireframe {
                        let mesh_data =
                            create_line([min.x, min.y, z].into(), [min.x, max.y, z].into(), color);
                        wireframe_mesh_data.append_mesh_data_as_meshlet(mesh_data);
                        let mesh_data =
                            create_line([min.x, max.y, z].into(), [max.x, max.y, z].into(), color);
                        wireframe_mesh_data.append_mesh_data_as_meshlet(mesh_data);
                        let mesh_data =
                            create_line([max.x, max.y, z].into(), [max.x, min.y, z].into(), color);
                        wireframe_mesh_data.append_mesh_data_as_meshlet(mesh_data);
                        let mesh_data =
                            create_line([max.x, min.y, z].into(), [min.x, min.y, z].into(), color);
                        wireframe_mesh_data.append_mesh_data_as_meshlet(mesh_data);
                    } else {
                        let mesh_data =
                            create_colored_quad([min.x, min.y, max.x, max.y].into(), z, color);
                        opaque_mesh_data.append_mesh_data_as_meshlet(mesh_data);
                    }
                }
                DrawEvent::Arrow(position, direction, color, is_wireframe) => {
                    inox_profiler::scoped_profile!("DrawEvent::Arrow");

                    let mesh_data = create_arrow(position, direction, color);
                    if is_wireframe {
                        wireframe_mesh_data.append_mesh_data_as_meshlet(mesh_data);
                    } else {
                        opaque_mesh_data.append_mesh_data_as_meshlet(mesh_data);
                    }
                }
                DrawEvent::Sphere(position, radius, color, is_wireframe) => {
                    inox_profiler::scoped_profile!("DrawEvent::Sphere");

                    if is_wireframe {
                        let mesh_data = create_sphere(position, radius, 16, 8, color);
                        wireframe_mesh_data.append_mesh_data_as_meshlet(mesh_data);
                    } else {
                        let mesh_data = create_sphere(position, radius, 16, 8, color);
                        opaque_mesh_data.append_mesh_data_as_meshlet(mesh_data);
                    }
                }
            });

        if !opaque_mesh_data.vertices.is_empty() {
            self.mesh_instance
                .get_mut()
                .set_mesh_data(opaque_mesh_data)
                .add_flag(MeshFlags::Visible);
        } else {
            self.mesh_instance.get_mut().remove_flag(MeshFlags::Visible);
        }
        if !wireframe_mesh_data.vertices.is_empty() {
            self.wireframe_mesh_instance
                .get_mut()
                .add_flag(MeshFlags::Visible)
                .set_mesh_data(wireframe_mesh_data);
        } else {
            self.wireframe_mesh_instance
                .get_mut()
                .remove_flag(MeshFlags::Visible);
        }
    }
}

unsafe impl Send for DebugDrawerSystem {}
unsafe impl Sync for DebugDrawerSystem {}

impl SystemUID for DebugDrawerSystem {
    fn system_id() -> SystemId
    where
        Self: Sized,
    {
        generate_uid_from_string(type_name::<Self>())
    }
}

impl System for DebugDrawerSystem {
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
    fn init(&mut self) {}

    fn run(&mut self) -> bool {
        self.update_events();
        true
    }

    fn uninit(&mut self) {
        self.listener.unregister::<ConfigEvent<Config>>();
    }
}

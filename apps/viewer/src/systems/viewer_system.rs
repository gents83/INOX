use std::{any::TypeId, path::Path};

use nrg_core::System;
use nrg_graphics::RenderPass;
use nrg_messenger::{read_messages, send_global_event, MessageChannel, MessengerRw};
use nrg_platform::{KeyEvent, MouseEvent, WindowEvent};
use nrg_resources::{DataTypeResource, Resource, SerializableResource, SharedData, SharedDataRc};
use nrg_scene::{Object, Scene};
use nrg_serialize::generate_random_uid;

use crate::config::Config;

pub struct ViewerSystem {
    shared_data: SharedDataRc,
    global_messenger: MessengerRw,
    config: Config,
    message_channel: MessageChannel,
    render_passes: Vec<Resource<RenderPass>>,
    scene: Resource<Scene>,
}

impl ViewerSystem {
    pub fn new(shared_data: SharedDataRc, global_messenger: MessengerRw, config: &Config) -> Self {
        let message_channel = MessageChannel::default();

        nrg_scene::register_resource_types(&shared_data);

        let scene = SharedData::add_resource::<Scene>(
            &shared_data,
            generate_random_uid(),
            Scene::default(),
        );

        Self {
            shared_data,
            global_messenger,
            config: config.clone(),
            message_channel,
            render_passes: Vec::new(),
            scene,
        }
    }

    fn window_init(&self) -> &Self {
        send_global_event(
            &self.global_messenger,
            WindowEvent::RequestChangeTitle(self.config.title.clone()),
        );
        send_global_event(
            &self.global_messenger,
            WindowEvent::RequestChangeSize(self.config.width, self.config.height),
        );
        send_global_event(
            &self.global_messenger,
            WindowEvent::RequestChangePos(self.config.pos_x, self.config.pos_y),
        );
        send_global_event(
            &self.global_messenger,
            WindowEvent::RequestChangeVisible(true),
        );
        self
    }
}

impl Drop for ViewerSystem {
    fn drop(&mut self) {
        nrg_scene::unregister_resource_types(&self.shared_data);
    }
}

impl System for ViewerSystem {
    fn should_run_when_not_focused(&self) -> bool {
        false
    }

    fn init(&mut self) {
        self.window_init();
        self.load_pipelines();

        self.global_messenger
            .write()
            .unwrap()
            .register_messagebox::<KeyEvent>(self.message_channel.get_messagebox())
            .register_messagebox::<MouseEvent>(self.message_channel.get_messagebox())
            .register_messagebox::<WindowEvent>(self.message_channel.get_messagebox());
    }

    fn run(&mut self) -> bool {
        self.update_events();

        self.scene.get_mut(|s| {
            s.update_hierarchy(&self.shared_data);
        });

        true
    }
    fn uninit(&mut self) {
        self.global_messenger
            .write()
            .unwrap()
            .unregister_messagebox::<KeyEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<MouseEvent>(self.message_channel.get_messagebox())
            .unregister_messagebox::<WindowEvent>(self.message_channel.get_messagebox());
    }
}

impl ViewerSystem {
    fn load_pipelines(&mut self) {
        for render_pass_data in self.config.render_passes.iter() {
            self.render_passes.push(RenderPass::create_from_data(
                &self.shared_data,
                &self.global_messenger,
                generate_random_uid(),
                render_pass_data.clone(),
            ));
        }
    }

    fn load_object(&mut self, filename: &Path) {
        self.scene.get_mut(|s| {
            s.clear();
            let object =
                Object::load_from_file(&self.shared_data, &self.global_messenger, filename);
            s.add_object(object);
        });
    }

    fn update_events(&mut self) -> &mut Self {
        nrg_profiler::scoped_profile!("update_events");

        read_messages(self.message_channel.get_listener(), |msg| {
            if msg.type_id() == TypeId::of::<KeyEvent>() {
                let _event = msg.as_any().downcast_ref::<KeyEvent>().unwrap();
            } else if msg.type_id() == TypeId::of::<MouseEvent>() {
                let _event = msg.as_any().downcast_ref::<MouseEvent>().unwrap();
            }
        });
        self
    }
}

use nrg_core::{JobHandlerRw, System, SystemId};
use nrg_messenger::{MessageChannel, MessengerRw};
use nrg_resources::{SharedData, SharedDataRw};
use nrg_scene::{Object, Scene, SceneRc, Transform};

pub struct UISystem {
    id: SystemId,
    shared_data: SharedDataRw,
    global_messenger: MessengerRw,
    message_channel: MessageChannel,
    job_handler: JobHandlerRw,
    ui_scene: SceneRc,
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
            global_messenger,
            job_handler,
            message_channel,
            ui_scene: SceneRc::default(),
        }
    }

    fn create_scene(&mut self) -> &mut Self {
        self.ui_scene = SharedData::add_resource::<Scene>(&self.shared_data, Scene::default());
        self
    }

    fn add_2d_quad(&mut self) -> &mut Self {
        let object = SharedData::add_resource::<Object>(&self.shared_data, Object::default());
        object
            .resource()
            .get_mut()
            .add_default_component::<Transform>(&self.shared_data);
        self.ui_scene.resource().get_mut().add_object(object);
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
        self.create_scene().add_2d_quad();
    }

    fn run(&mut self) -> bool {
        true
    }

    fn uninit(&mut self) {}
}

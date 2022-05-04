use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use inox_core::{define_plugin, ContextRc, Plugin, SystemUID, WindowSystem};

use inox_graphics::{
    rendering_system::RenderingSystem, update_system::UpdateSystem, DebugDrawerSystem, OpaquePass,
    Pass, PassEvent, RenderPass, RenderTarget, Renderer, RendererRw, DEFAULT_HEIGHT, DEFAULT_WIDTH,
    OPAQUE_PASS_NAME,
};
use inox_platform::Window;
use inox_resources::ConfigBase;
use inox_serialize::read_from_file;
use inox_ui::{UIPass, UISystem, UI_PASS_NAME};

use crate::{config::Config, systems::viewer_system::ViewerSystem};

pub struct Viewer {
    window: Option<Window>,
    renderer: RendererRw,
}
define_plugin!(Viewer);

impl Plugin for Viewer {
    fn create(context: &ContextRc) -> Self {
        let window = {
            Window::create(
                "SABI".to_string(),
                0,
                0,
                DEFAULT_WIDTH,
                DEFAULT_HEIGHT,
                PathBuf::from("").as_path(),
                context.message_hub(),
            )
        };
        let renderer = Renderer::new(window.handle(), context, false);
        let renderer = Arc::new(RwLock::new(renderer));

        Self::create_render_passes(context, window.width(), window.height());

        Viewer {
            window: Some(window),
            renderer,
        }
    }

    fn name(&self) -> &str {
        "inox_viewer"
    }

    fn prepare(&mut self, context: &ContextRc) {
        let window_system = WindowSystem::new(self.window.take().unwrap(), context);
        let render_update_system = UpdateSystem::new(self.renderer.clone(), context);
        let rendering_draw_system = RenderingSystem::new(self.renderer.clone(), context);
        let debug_drawer_system = DebugDrawerSystem::new(context);
        let ui_render_pass = context
            .shared_data()
            .match_resource(|r: &RenderPass| r.data().name == UI_PASS_NAME);
        let ui_system = UISystem::new(context, &ui_render_pass.unwrap());
        let system = ViewerSystem::new(context);

        context.add_system(inox_core::Phases::PlatformUpdate, window_system, None);
        context.add_system(
            inox_core::Phases::Render,
            render_update_system,
            Some(&[RenderingSystem::system_id()]),
        );
        context.add_system(
            inox_core::Phases::Render,
            rendering_draw_system,
            Some(&[UpdateSystem::system_id()]),
        );
        context.add_system(inox_core::Phases::Update, system, None);
        context.add_system(inox_core::Phases::Update, ui_system, None);
        context.add_system(inox_core::Phases::Update, debug_drawer_system, None);
    }

    fn unprepare(&mut self, context: &ContextRc) {
        context.remove_system(inox_core::Phases::Update, &DebugDrawerSystem::system_id());
        context.remove_system(inox_core::Phases::Update, &UISystem::system_id());
        context.remove_system(inox_core::Phases::Update, &ViewerSystem::system_id());

        context.remove_system(
            inox_core::Phases::PlatformUpdate,
            &WindowSystem::system_id(),
        );
        context.remove_system(inox_core::Phases::Render, &UpdateSystem::system_id());
        context.remove_system(inox_core::Phases::Render, &RenderingSystem::system_id());
    }

    fn load_config(&mut self, context: &ContextRc) {
        let config = Config::default();
        let shared_data = context.shared_data().clone();

        read_from_file(
            config.get_filepath(self.name()).as_path(),
            context.shared_data().serializable_registry(),
            Box::new(move |data: Config| {
                if let Some(ui_pass) =
                    shared_data.match_resource(|r: &RenderPass| r.data().name == UI_PASS_NAME)
                {
                    ui_pass.get_mut().pipelines(data.ui_pass_pipelines);
                }
                if let Some(opaque_pass) =
                    shared_data.match_resource(|r: &RenderPass| r.data().name == OPAQUE_PASS_NAME)
                {
                    opaque_pass.get_mut().pipelines(data.opaque_pass_pipelines);
                }
            }),
        );
    }
}

impl Viewer {
    fn create_render_passes(context: &ContextRc, width: u32, height: u32) {
        let opaque_pass = OpaquePass::create(context);
        let ui_pass = UIPass::create(context);

        let opaque_pass_render_target = RenderTarget::Texture {
            width,
            height,
            read_back: false,
        };
        opaque_pass
            .pass()
            .get_mut()
            .render_target(opaque_pass_render_target)
            .depth_target(opaque_pass_render_target);

        context
            .message_hub()
            .send_event(PassEvent::Add(Box::new(opaque_pass)));
        context
            .message_hub()
            .send_event(PassEvent::Add(Box::new(ui_pass)));
    }
}

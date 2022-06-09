use std::path::PathBuf;

use inox_core::{define_plugin, ContextRc, Plugin, SystemUID, WindowSystem};

use inox_graphics::{
    rendering_system::RenderingSystem, update_system::UpdateSystem, CullingPass, DebugDrawerSystem,
    OpaquePass, Pass, RenderPass, RenderTarget, Renderer, RendererRw, TransparentPass,
    DEFAULT_HEIGHT, DEFAULT_WIDTH, OPAQUE_PASS_NAME,
};
use inox_platform::Window;
use inox_resources::ConfigBase;
use inox_scene::{ObjectSystem, ScriptSystem};
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

        Self::create_render_passes(context, &renderer, window.width(), window.height());

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
        let ui_system = UISystem::new(context, self.renderer.clone());

        let system = ViewerSystem::new(context, &self.renderer);
        let object_system = ObjectSystem::new(context.shared_data());
        let script_system = ScriptSystem::new(context);

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

        context.add_system(inox_core::Phases::Update, object_system, None);
        context.add_system(
            inox_core::Phases::Update,
            script_system,
            Some(&[ObjectSystem::system_id()]),
        );

        context.add_system(inox_core::Phases::Update, system, None);
        context.add_system(inox_core::Phases::Update, ui_system, None);
        context.add_system(inox_core::Phases::Update, debug_drawer_system, None);
    }

    fn unprepare(&mut self, context: &ContextRc) {
        context.remove_system(inox_core::Phases::Update, &DebugDrawerSystem::system_id());
        context.remove_system(inox_core::Phases::Update, &UISystem::system_id());
        context.remove_system(inox_core::Phases::Update, &ViewerSystem::system_id());

        context.remove_system(inox_core::Phases::Update, &ScriptSystem::system_id());
        context.remove_system(inox_core::Phases::Update, &ObjectSystem::system_id());

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
                    shared_data.match_resource(|r: &RenderPass| r.name() == UI_PASS_NAME)
                {
                    ui_pass.get_mut().set_pipelines(&data.ui_pass_pipelines);
                }
                if let Some(opaque_pass) =
                    shared_data.match_resource(|r: &RenderPass| r.name() == OPAQUE_PASS_NAME)
                {
                    opaque_pass
                        .get_mut()
                        .set_pipelines(&data.opaque_pass_pipelines);
                }
            }),
        );
    }
}

impl Viewer {
    fn create_render_passes(context: &ContextRc, renderer: &RendererRw, width: u32, height: u32) {
        const ONLY_OPAQUE_PASS: bool = true;
        if ONLY_OPAQUE_PASS {
            Self::create_opaque_pass(context, renderer);
        } else {
            Self::create_full_render_passes(context, renderer, width, height);
        }
    }
    fn create_opaque_pass(context: &ContextRc, renderer: &RendererRw) {
        let opaque_pass = OpaquePass::create(context);

        renderer.write().unwrap().add_pass(opaque_pass);
    }
    fn create_full_render_passes(
        context: &ContextRc,
        renderer: &RendererRw,
        width: u32,
        height: u32,
    ) {
        let culling_pass = CullingPass::create(context);
        let opaque_pass = OpaquePass::create(context);
        let transparent_pass = TransparentPass::create(context);
        let ui_pass = UIPass::create(context);

        let opaque_pass_render_target = RenderTarget::Texture {
            width,
            height,
            read_back: false,
        };
        opaque_pass
            .render_pass()
            .get_mut()
            .render_target(opaque_pass_render_target)
            .depth_target(opaque_pass_render_target);
        transparent_pass
            .render_pass()
            .get_mut()
            .render_target_from_texture(
                opaque_pass
                    .render_pass()
                    .get()
                    .render_texture()
                    .as_ref()
                    .unwrap(),
            )
            .depth_target_from_texture(
                opaque_pass
                    .render_pass()
                    .get()
                    .depth_texture()
                    .as_ref()
                    .unwrap(),
            );

        renderer
            .write()
            .unwrap()
            .add_pass(culling_pass)
            .add_pass(opaque_pass)
            .add_pass(transparent_pass)
            .add_pass(ui_pass);
    }
}

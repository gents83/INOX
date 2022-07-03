use std::path::PathBuf;

use inox_core::{define_plugin, ContextRc, Plugin, SystemUID, WindowSystem};

use inox_graphics::{
    rendering_system::RenderingSystem, update_system::UpdateSystem, BlitPass, DebugDrawerSystem,
    DefaultPass, Pass, RenderPass, RenderTarget, Renderer, RendererRw, TextureFormat,
    WireframePass, DEFAULT_HEIGHT, DEFAULT_PASS_NAME, DEFAULT_WIDTH, WIREFRAME_PASS_NAME,
};
use inox_platform::Window;
use inox_resources::ConfigBase;
use inox_scene::{ObjectSystem, ScriptSystem};
use inox_serialize::read_from_file;
use inox_ui::{UIPass, UISystem, UI_PASS_NAME};

use crate::{config::Config, systems::viewer_system::ViewerSystem};

const ADD_WIREFRAME_PASS: bool = true;
const ADD_UI_PASS: bool = true;

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
        let mut ui_system = if ADD_UI_PASS {
            Some(UISystem::new(context, self.renderer.clone()))
        } else {
            None
        };

        let system = ViewerSystem::new(context, &self.renderer, ADD_UI_PASS);
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
        if let Some(ui_system) = ui_system.take() {
            context.add_system(inox_core::Phases::Update, ui_system, None);
        }
        if ADD_WIREFRAME_PASS {
            let debug_drawer_system = DebugDrawerSystem::new(context);
            context.add_system(inox_core::Phases::Update, debug_drawer_system, None);
        }
    }

    fn unprepare(&mut self, context: &ContextRc) {
        if ADD_WIREFRAME_PASS {
            context.remove_system(inox_core::Phases::Update, &DebugDrawerSystem::system_id());
        }
        if ADD_UI_PASS {
            context.remove_system(inox_core::Phases::Update, &UISystem::system_id());
        }
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
                    ui_pass.get_mut().set_pipeline(&data.ui_pass_pipeline);
                }
                if let Some(default_pass) =
                    shared_data.match_resource(|r: &RenderPass| r.name() == DEFAULT_PASS_NAME)
                {
                    default_pass
                        .get_mut()
                        .set_pipeline(&data.opaque_pass_pipeline);
                }
                if let Some(wireframe_pass) =
                    shared_data.match_resource(|r: &RenderPass| r.name() == WIREFRAME_PASS_NAME)
                {
                    wireframe_pass
                        .get_mut()
                        .set_pipeline(&data.wireframe_pass_pipeline);
                }
            }),
        );
    }
}

impl Viewer {
    fn create_render_passes(context: &ContextRc, renderer: &RendererRw, width: u32, height: u32) {
        Self::create_default_pass(context, renderer, width, height);
        if ADD_WIREFRAME_PASS {
            Self::create_wireframe_pass(context, renderer);
        }
        Self::create_blit_pass(context, renderer);
        if ADD_UI_PASS {
            Self::create_ui_pass(context, renderer, width, height);
        }
    }
    fn create_default_pass(context: &ContextRc, renderer: &RendererRw, width: u32, height: u32) {
        let default_pass = DefaultPass::create(context);

        default_pass
            .render_pass()
            .get_mut()
            .add_render_target(RenderTarget::Texture {
                width,
                height,
                format: TextureFormat::Rgba32Float,
                read_back: false,
            })
            .add_render_target(RenderTarget::Texture {
                width,
                height,
                format: TextureFormat::Rgba32Float,
                read_back: false,
            })
            .add_depth_target(RenderTarget::Texture {
                width,
                height,
                format: TextureFormat::Depth32Float,
                read_back: false,
            });

        renderer.write().unwrap().add_pass(default_pass);
    }
    fn create_blit_pass(context: &ContextRc, renderer: &RendererRw) {
        let mut blit_pass = BlitPass::create(context);

        if let Some(default_pass) = renderer.read().unwrap().pass::<DefaultPass>() {
            let default_pass = default_pass.render_pass().get();
            let render_target_textures = default_pass.render_textures_id();
            blit_pass.set_source(render_target_textures[0]);
        }
        renderer.write().unwrap().add_pass(blit_pass);
    }
    fn create_wireframe_pass(context: &ContextRc, renderer: &RendererRw) {
        let wireframe_pass = WireframePass::create(context);

        if let Some(default_pass) = renderer.read().unwrap().pass::<DefaultPass>() {
            if let Some(wireframe_pass) = renderer.read().unwrap().pass::<WireframePass>() {
                let mut wireframe_pass = wireframe_pass.render_pass().get_mut();
                let default_pass = default_pass.render_pass().get();
                default_pass.render_textures().iter().for_each(|texture| {
                    wireframe_pass.add_render_target_from_texture(texture);
                });
                if let Some(depth_target) = default_pass.depth_texture() {
                    wireframe_pass.add_depth_target_from_texture(depth_target);
                }
            }
        }
        renderer.write().unwrap().add_pass(wireframe_pass);
    }
    fn create_ui_pass(context: &ContextRc, renderer: &RendererRw, width: u32, height: u32) {
        let ui_pass = UIPass::create(context);
        if let Some(blit_pass) = renderer.read().unwrap().pass::<BlitPass>() {
            blit_pass
                .render_pass()
                .get_mut()
                .add_render_target(RenderTarget::Texture {
                    width,
                    height,
                    format: TextureFormat::Rgba8Unorm,
                    read_back: false,
                });
        }
        renderer.write().unwrap().add_pass(ui_pass);
    }
    /*
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
    }*/
}

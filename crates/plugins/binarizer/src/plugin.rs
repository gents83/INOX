use std::sync::atomic::AtomicBool;

use inox_commands::CommandParser;
use inox_core::{define_plugin, ContextRc, Plugin, SystemId, SystemUID};
use inox_platform::{
    PLATFORM_TYPE_ANDROID_NAME, PLATFORM_TYPE_IOS_NAME, PLATFORM_TYPE_PC, PLATFORM_TYPE_PC_NAME,
    PLATFORM_TYPE_WEB, PLATFORM_TYPE_WEB_NAME,
};
use inox_resources::{PC_FOLDER, WEB_FOLDER};

use crate::{Binarizer, BinarizerParameters};

#[repr(C)]
#[derive(Default)]
pub struct BinarizerPlugin {
    pc_id: SystemId,
    web_id: SystemId,
}
define_plugin!(BinarizerPlugin);

impl Plugin for BinarizerPlugin {
    fn create(_context: &ContextRc) -> Self {
        BinarizerPlugin::default()
    }
    fn name(&self) -> &str {
        "inox_binarizer"
    }
    fn prepare(&mut self, context: &ContextRc) {
        let command_parser = CommandParser::from_command_line();
        let mut platform = command_parser.get_values_of::<String>("platform");
        if platform.is_empty() {
            platform.push(PLATFORM_TYPE_PC_NAME.to_string());
        }
        let mut shader_paths = Vec::new();
        if command_parser.has("preprocess_shader") {
            let values = command_parser.get_values_of::<String>("preprocess_shader");
            if !values.is_empty() {
                shader_paths = values[0]
                    .as_str()
                    .split(',')
                    .map(|s| s.to_string())
                    .collect();
            }
        }
        let info = BinarizerParameters {
            should_end_on_completion: AtomicBool::new(true),
            optimize_meshes: AtomicBool::new(true),
            preprocess_shaders_paths: shader_paths,
        };
        for name in platform.iter() {
            let name = name.as_str();
            match name {
                PLATFORM_TYPE_PC_NAME => {
                    let system = Binarizer::<PLATFORM_TYPE_PC>::new(
                        context,
                        inox_resources::Data::data_raw_folder(),
                        inox_resources::Data::data_folder().join(PC_FOLDER),
                        &info,
                    );
                    self.pc_id = Binarizer::<PLATFORM_TYPE_PC>::system_id();
                    context.add_system(inox_core::Phases::PreUpdate, system, None);
                }
                PLATFORM_TYPE_WEB_NAME => {
                    let system = Binarizer::<PLATFORM_TYPE_WEB>::new(
                        context,
                        inox_resources::Data::data_raw_folder(),
                        inox_resources::Data::data_folder().join(WEB_FOLDER),
                        &info,
                    );
                    self.web_id = Binarizer::<PLATFORM_TYPE_WEB>::system_id();
                    context.add_system(inox_core::Phases::PreUpdate, system, None);
                }
                PLATFORM_TYPE_ANDROID_NAME => {
                    inox_log::debug_log!("Binarization for Android platform is not supported yet");
                }
                PLATFORM_TYPE_IOS_NAME => {
                    inox_log::debug_log!("Binarization for IOS platform is not supported yet");
                }
                _ => {}
            }
        }
    }

    fn unprepare(&mut self, context: &ContextRc) {
        context.remove_system(inox_core::Phases::PreUpdate, &self.pc_id);
        context.remove_system(inox_core::Phases::PreUpdate, &self.web_id);
    }
    fn load_config(&mut self, _context: &ContextRc) {}
}

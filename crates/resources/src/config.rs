use std::path::PathBuf;

use inox_commands::CommandParser;
use inox_filesystem::convert_from_local_path;
use inox_messenger::implement_message;

use crate::Data;

pub const CONFIG_FOLDER: &str = "config";

pub trait ConfigBase: 'static + PartialEq + Send + Sync {
    #[inline]
    fn get_folder(&self) -> PathBuf {
        Data::platform_data_folder().join(CONFIG_FOLDER)
    }
    #[inline]
    fn get_filepath(&self, plugin_name: &str) -> PathBuf {
        convert_from_local_path(
            Data::platform_data_folder().as_path(),
            self.get_folder()
                .join(plugin_name)
                .join(self.get_filename())
                .as_path(),
        )
    }
    fn get_filename(&self) -> &'static str;
}

pub enum ConfigEvent<T>
where
    T: ConfigBase,
{
    Loaded(String, T),
}
implement_message!(
    ConfigEvent<ConfigBase>,
    [conversion = message_from_command_parser],
    [policy = compare_and_discard]
);

unsafe impl<T> Send for ConfigEvent<T> where T: ConfigBase {}
unsafe impl<T> Sync for ConfigEvent<T> where T: ConfigBase {}

impl<T> ConfigEvent<T>
where
    T: ConfigBase,
{
    fn compare_and_discard(&self, other: &Self) -> bool {
        match self {
            Self::Loaded(filename, data) => match other {
                Self::Loaded(other_filename, other_data) => {
                    filename == other_filename && data == other_data
                }
            },
        }
    }
    fn message_from_command_parser(_command_parser: CommandParser) -> Option<Self> {
        None
    }
}

use std::{path::PathBuf, process::Command};

use nrg_messenger::{get_events_from_string, Message, MessageBox, MessengerRw};
use nrg_resources::{SharedDataRw, DATA_FOLDER, DATA_RAW_FOLDER};
use nrg_serialize::deserialize;
use nrg_ui::{
    implement_widget_data, menu, DialogEvent, DialogOp, TopBottomPanel, UIWidget, UIWidgetRc,
};

pub struct MainMenu {
    ui_page: UIWidgetRc,
}

impl MainMenu {
    pub fn new(shared_data: &SharedDataRw, global_messenger: &MessengerRw) -> Self {
        let ui_page = Self::create(shared_data, global_messenger);
        Self { ui_page }
    }

    fn create(shared_data: &SharedDataRw, global_messenger: &MessengerRw) -> UIWidgetRc {
        struct MenuData {
            show_debug_info: bool,
            global_dispatcher: MessageBox,
        }
        implement_widget_data!(MenuData);
        let data = MenuData {
            show_debug_info: false,
            global_dispatcher: global_messenger.read().unwrap().get_dispatcher().clone(),
        };

        UIWidget::register(shared_data, data, |ui_data, ui_context| {
            if let Some(data) = ui_data.as_any().downcast_mut::<MenuData>() {
                TopBottomPanel::top("main_menu")
                    .resizable(false)
                    .show(ui_context, |ui| {
                        menu::bar(ui, |ui| {
                            menu::menu(ui, "File", |ui| {
                                if ui.button("New").clicked() {
                                    let mut command = Command::new("nrg_content_browser");
                                    let op: &str = DialogOp::New.into();
                                    command
                                        .arg("New File")
                                        .arg(PathBuf::from(DATA_RAW_FOLDER).to_str().unwrap())
                                        .arg(op)
                                        .arg("false");
                                    Self::process_command_result(
                                        &mut command,
                                        data.global_dispatcher.clone(),
                                    );
                                }
                                if ui.button("Open").clicked() {
                                    let mut command = Command::new("nrg_content_browser");
                                    let op: &str = DialogOp::Open.into();
                                    command
                                        .arg("Open File")
                                        .arg(PathBuf::from(DATA_FOLDER).to_str().unwrap())
                                        .arg(op)
                                        .arg("false");
                                    Self::process_command_result(
                                        &mut command,
                                        data.global_dispatcher.clone(),
                                    );
                                }
                                if ui.button("Save").clicked() {
                                    let mut command = Command::new("nrg_content_browser");
                                    let op: &str = DialogOp::Save.into();
                                    command
                                        .arg("Save File")
                                        .arg(PathBuf::from(DATA_RAW_FOLDER).to_str().unwrap())
                                        .arg(op)
                                        .arg("true");
                                    Self::process_command_result(
                                        &mut command,
                                        data.global_dispatcher.clone(),
                                    );
                                }
                                if ui.button("Exit").clicked() {}
                            });
                            menu::menu(ui, "Settings", |ui| {
                                ui.checkbox(&mut data.show_debug_info, "Debug Info");
                            });
                        });
                    });
            }
        })
    }

    fn process_command_result(command: &mut Command, dispatcher: MessageBox) {
        let result = command.output();
        match result {
            Ok(output) => {
                let string = String::from_utf8(output.stdout).unwrap();
                println!("{}", string);
                for e in get_events_from_string(string) {
                    let event: DialogEvent = deserialize(e);
                    dispatcher.write().unwrap().send(event.as_boxed()).ok();
                }
            }
            Err(_) => {
                println!("Failed to execute process");
            }
        }
    }
}

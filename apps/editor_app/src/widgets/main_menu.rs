use std::{
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use nrg_messenger::{get_events_from_string, Message, MessageBox, MessengerRw};
use nrg_platform::WindowEvent;
use nrg_resources::{SharedDataRw, DATA_FOLDER, DATA_RAW_FOLDER};
use nrg_serialize::deserialize;
use nrg_ui::{
    implement_widget_data, menu, DialogEvent, DialogOp, TopBottomPanel, UIWidget, UIWidgetRc,
};

struct MenuData {
    show_debug_info: Arc<AtomicBool>,
    global_dispatcher: MessageBox,
}
implement_widget_data!(MenuData);

pub struct MainMenu {
    ui_page: UIWidgetRc,
}

impl MainMenu {
    pub fn new(
        shared_data: &SharedDataRw,
        global_messenger: &MessengerRw,
        show_debug_info: Arc<AtomicBool>,
    ) -> Self {
        let data = MenuData {
            show_debug_info,
            global_dispatcher: global_messenger.read().unwrap().get_dispatcher().clone(),
        };
        let ui_page = Self::create(shared_data, data);
        Self { ui_page }
    }

    fn create(shared_data: &SharedDataRw, data: MenuData) -> UIWidgetRc {
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
                                if ui.button("Exit").clicked() {
                                    data.global_dispatcher
                                        .write()
                                        .unwrap()
                                        .send(WindowEvent::Close.as_boxed())
                                        .ok();
                                }
                            });
                            menu::menu(ui, "Settings", |ui| {
                                let mut show_debug_info =
                                    data.show_debug_info.load(Ordering::SeqCst);
                                ui.checkbox(&mut show_debug_info, "Debug Info");
                                data.show_debug_info
                                    .store(show_debug_info, Ordering::SeqCst);
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

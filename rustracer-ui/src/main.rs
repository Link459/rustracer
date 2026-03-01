use std::{
    error::Error,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Child, Command},
    str::FromStr,
};

use egui::{Align2, Id, LayerId, Popup, PopupAnchor, PopupCloseBehavior, RectAlign, RichText};
use egui_file::FileDialog;
use rustracer::{
    render::Skybox,
    scene::Scene,
    settings::{PresentSettings, SceneSettings, Settings},
};

#[derive(Default)]
struct SettingsApp {
    settings: Settings,

    output_path: String,
    output_file_dialogue: Option<FileDialog>,

    error: String,
    error_open: bool,

    scene_file_dialogue: Option<FileDialog>,

    child: Option<Child>,
}

impl SettingsApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut s = Self::default();
        s.output_path = s.settings.output.to_str().unwrap().to_string();
        return s;
    }

    fn update_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.label("Path: ");
            if ui.text_edit_singleline(&mut self.output_path).clicked() {
                let mut dialogue = egui_file::FileDialog::select_folder()
                    .show_new_folder(true)
                    .show_rename(false)
                    .initial_path(&self.output_path);
                dialogue.open();
                self.output_file_dialogue = Some(dialogue);
            }

            if let Some(dialogue) = &mut self.output_file_dialogue {
                if dialogue.show(ctx).selected() {
                    if let Some(file) = dialogue.path() {
                        self.output_path = file.to_str().unwrap().to_string();
                        //self.opened_file = Some(file.to_path_buf());
                    }
                }
            }
        });

        if ui
            .checkbox(&mut self.settings.log_messages, "Log Messages")
            .hovered()
        {
            ui.label("Whether to log messages to the console");
        }

        let selected = &mut self.settings.present_settings;
        egui::ComboBox::from_label("Present Settings")
            .selected_text(format!("{:?}", selected))
            .show_ui(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(selected, PresentSettings::OnceDone, "Once Done");
                    ui.label("Renders the scene once and presents it");
                });

                ui.horizontal(|ui| {
                    ui.selectable_value(selected, PresentSettings::Accumulate, "Accumulate");
                    ui.label("Continously renders the scene and accumulates the result");
                });
            });
        ui.horizontal(|ui| {
            ui.label("Width: ");
            ui.add(egui::DragValue::new(
                &mut self.settings.render_settings.width,
            ));
        });

        ui.horizontal(|ui| {
            ui.label("Height: ");
            ui.add(egui::DragValue::new(
                &mut self.settings.render_settings.height,
            ));
        });
        ui.horizontal(|ui| {
            ui.label("Max Depth: ");
            ui.add(egui::DragValue::new(
                &mut self.settings.render_settings.max_depth,
            ));
        });
        ui.horizontal(|ui| {
            ui.label("Samples: ");
            ui.add(egui::DragValue::new(
                &mut self.settings.render_settings.samples,
            ));
        });

        let selected = &mut self.settings.render_settings.skybox;
        egui::ComboBox::from_label("Skybox")
            .selected_text(format!("{:?}", selected))
            .show_ui(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(selected, Skybox::Sky, "Sky");
                    ui.label("A procedually generated sky");
                });
                ui.horizontal(|ui| {
                    ui.selectable_value(selected, Skybox::Night, "Night");
                    ui.label("A pitch black skybox");
                });
                ui.horizontal(|ui| {
                    //ui.selectable_value(selected, , "Night");
                    ui.label("A pitch black skybox");
                });
            });
    }

    fn scene(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let mut scene_path = match &self.settings.scene_settings {
            SceneSettings::Index(_) => "".to_string(),
            SceneSettings::Path(path_buf) => path_buf.to_str().unwrap().to_string(),
        };
        if ui.text_edit_singleline(&mut scene_path).clicked() {
            let filter = Box::new({
                let ext = Some(OsStr::new("ron"));
                move |path: &Path| -> bool { path.extension() == ext }
            });
            let mut dialogue = egui_file::FileDialog::open_file()
                .show_files_filter(filter)
                .show_new_folder(false)
                .show_rename(false);
            dialogue.open();
            self.scene_file_dialogue = Some(dialogue);
        }

        if let Some(dialogue) = &mut self.scene_file_dialogue {
            if dialogue.show(ctx).selected() {
                if let Some(file) = dialogue.path() {
                    let str = file.to_str();
                    match str {
                        Some(path) => match PathBuf::from_str(path) {
                            Ok(path_buf) => {
                                self.settings.scene_settings = SceneSettings::Path(path_buf)
                            }
                            Err(e) => self.set_error(e),
                        },
                        _ => (),
                    }
                }
            }
        }
        let mut scenes = rustracer::world_options::get_scenes();
        scenes.insert(0, ("None", || Scene::default()));

        let mut current = "None";
        match self.settings.scene_settings {
            SceneSettings::Index(idx) => current = scenes[idx].0,
            _ => (),
        }

        let mut idx = 0;
        egui::ComboBox::from_label("Premade Scene: ")
            .selected_text(current)
            .show_ui(ui, |ui| {
                for (name, _) in scenes {
                    if ui.selectable_label(false, name).clicked() {
                        current = name;
                        if name != "None" {
                            self.settings.scene_settings = SceneSettings::Index(idx - 1);
                        }
                    }
                    idx += 1;
                }
            });
    }

    fn save_settings(&mut self) {
        let serialized = toml::to_string(&self.settings);
        match serialized {
            Ok(x) => {
                if let Err(e) = std::fs::write("settings.toml", x) {
                    self.set_error(e);
                }
            }
            Err(e) => {
                self.set_error(e);
                return;
            }
        }
    }

    fn spawn(&mut self) {
        if let Some(ref mut child) = self.child {
            if let Err(e) = child.kill() {
                self.set_error(e);
                return;
            }
        }
        let handle = Command::new("cargo")
            .arg("run")
            .arg("-p")
            .arg("rustracer")
            .arg("--")
            .arg("--settings")
            .arg("settings.toml")
            .spawn();

        match handle {
            Ok(child) => self.child = Some(child),
            Err(e) => {
                self.set_error(e);
                return;
            }
        }
    }

    fn runner(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Start").clicked() {
                self.save_settings();
                self.spawn();
            }

            if ui.button("Kill").clicked() {
                if let Some(ref mut child) = self.child {
                    child.kill().unwrap();
                    self.child = None;
                }
            }
        });
    }

    fn set_error<E: Error>(&mut self, error: E) {
        let str = format!("Error: {}", error.to_string());
        self.set_error_string(str);
    }

    fn set_error_string(&mut self, str: String) {
        self.error = str;
        self.error_open = true;
    }

    fn error(&mut self, ctx: &egui::Context) {
        //self.error_open = !error.is_empty();
        let mut open = self.error_open;

        let layer = LayerId::new(egui::Order::Foreground, Id::new("ErrorPopupLayer"));
        let align = RectAlign {
            parent: Align2::CENTER_CENTER,
            child: Align2::CENTER_CENTER,
        };
        let _popup = Popup::new(
            Id::new("ErrorPopup"),
            ctx.clone(),
            PopupAnchor::ParentRect(ctx.content_rect()),
            layer,
        )
        .open_bool(&mut open)
        .align(align)
        .close_behavior(PopupCloseBehavior::CloseOnClick)
        .show(|ui| {
            let text = RichText::new(format!("Error: {}", self.error))
                .color(ui.style_mut().visuals.error_fg_color);
            ui.label(text);
        });
        self.error_open = open;
    }
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let title = RichText::new("Rustracer").heading().size(32.0);
            ui.label(title);
            ui.heading("Settings");
            self.update_settings(ui, ctx);
            ui.separator();
            ui.heading("Scene");
            self.scene(ui, ctx);
            ui.separator();
            ui.heading("Run");
            self.runner(ui);

            self.error(ctx);
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "rustracer",
        native_options,
        Box::new(|cc| Ok(Box::new(SettingsApp::new(cc)))),
    )?;

    return Ok(());
}

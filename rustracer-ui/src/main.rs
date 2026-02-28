use std::{ffi::OsStr, path::Path};

use egui::{Align2, Id, LayerId, Popup, PopupAnchor, PopupCloseBehavior, RectAlign, RichText};
use egui_file::FileDialog;
use rustracer::{
    render::Background,
    scene::Scene,
    settings::{PresentSettings, Settings},
};

#[derive(Default)]
struct SettingsApp {
    settings: Settings,

    output_path: String,
    output_file_dialogue: Option<FileDialog>,

    error: String,
    error_open: bool,

    scene_path: String,
    scene_file_dialogue: Option<FileDialog>,
    scene_idx: Option<usize>,
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

        let selected = &mut self.settings.render_settings.background;
        egui::ComboBox::from_label("Skybox")
            .selected_text(format!("{:?}", selected))
            .show_ui(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(selected, Background::Sky, "Sky");
                    ui.label("A procedually generated sky");
                });
                ui.horizontal(|ui| {
                    ui.selectable_value(selected, Background::Night, "Night");
                    ui.label("A pitch black skybox");
                });
                ui.horizontal(|ui| {
                    //ui.selectable_value(selected, , "Night");
                    ui.label("A pitch black skybox");
                });
            });
    }

    fn scene(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if ui.text_edit_singleline(&mut self.scene_path).clicked() {
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
                    self.scene_path = file.to_str().unwrap().to_string();
                }
            }
        }
        let mut scenes = rustracer::world_options::get_scenes();
        scenes.insert(0, ("None", || Scene::default()));

        let mut current = "None";
        if self.scene_idx.is_some() {
            current = scenes[self.scene_idx.unwrap()].0;
        }

        let mut idx = 0;
        egui::ComboBox::from_label("Premade Scene: ")
            .selected_text(current)
            .show_ui(ui, |ui| {
                for (name, _) in scenes {
                    println!("{},{}", idx, name);
                    if ui.selectable_label(false, name).clicked() {
                        current = name;
                        if name == "None" {
                            self.scene_idx = None;
                        } else {
                            self.scene_idx = Some(idx - 1);
                        }
                    }
                    idx += 1;
                }
            });
    }

    fn runner(&mut self, ui: &mut egui::Ui) {
        if ui.button("Start").clicked() {
            let serialized = toml::to_string(&self.settings).unwrap();
            std::fs::write("settings.toml", serialized).unwrap();

            let handle = std::process::Command::new("cargo")
                .arg("run")
                .arg("-p")
                .arg("rustracer")
                .arg("--")
                .arg("--settings")
                .arg("settings.toml")
                .spawn();
            if handle.is_err() {
                self.set_error("Failed to spawn rustracer");
            }
        }
    }

    fn set_error(&mut self, str: &str) {
        self.error = str.to_string();
        self.error_open = true;
    }

    fn error(&mut self, ctx: &egui::Context) {
        //self.error_open = !error.is_empty();
        let mut open = self.error_open;

        let layer = LayerId::new(egui::Order::Foreground, Id::new("PopupLayer"));
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

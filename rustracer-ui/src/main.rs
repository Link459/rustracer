use std::{path::PathBuf, str::FromStr};

use egui::{
    Align2, AtomExt, Id, LayerId, Popup, PopupAnchor, PopupCloseBehavior, Pos2, Rect, RectAlign,
    ResizeDirection, RichText,
};
use rustracer::settings::{PresentSettings, Settings};

#[derive(Default)]
struct SettingsApp {
    settings: Settings,
    output: String,
    error: String,
    error_open: bool,
}

impl SettingsApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut s = Self::default();
        s.output = s.settings.output.to_str().unwrap().to_string();
        return s;
    }

    fn update_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Path: ");
            if ui.text_edit_singleline(&mut self.output).changed() {
                self.settings.output = PathBuf::from_str(&self.output).unwrap();
            }
        });

        if ui
            .checkbox(&mut self.settings.log_messages, "Log Messages")
            .hovered()
        {
            ui.label("Whether to log messages to the console");
        }

        let mut selected = &mut self.settings.present_settings;
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
    }

    fn runner(&mut self, ui: &mut egui::Ui) {
        if ui.button("Start").clicked() {
            let handle = std::process::Command::new("cargo")
                .arg("run")
                .arg("-p")
                .arg("rustracer")
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
            self.update_settings(ui);
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

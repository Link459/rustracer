struct SettingsApp {}

impl SettingsApp {
        fn new(cc: &eframe::CreationContext<'_>) -> Self {
            // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
            // Restore app state using cc.storage (requires the "persistence" feature).
            // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
            // for e.g. egui::PaintCallback.
            Self::default()
        }
    }

    impl eframe::App for SettingsApp {
        fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Rustracer");
            });
        }
    }

fn main() {
    let ui_event_loop =
        winit::event_loop::EventLoop::<eframe::UserEvent>::with_user_event().build()?;
    let native_options = eframe::NativeOptions::default();
    let mut ui_app = eframe::create_native(
        "rustracer",
        native_options,
        Box::new(|cc| Ok(Box::new(SettingsApp::new(cc)))),
        &ui_event_loop,
    );
    ui_event_loop.run_app(&mut ui_app);
}

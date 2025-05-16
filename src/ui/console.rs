use eframe::egui;
use crate::app_state::MyApp;

pub fn render(app: &mut MyApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("console_panel").exact_height(200.0).show(ctx, |ui| {
        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
            ui.vertical(|ui| {
                for message in &app.console_messages {
                    ui.label(egui::RichText::new(message).monospace());
                }
            });
        });
    });
}

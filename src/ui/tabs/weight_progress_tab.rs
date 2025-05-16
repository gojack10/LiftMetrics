use crate::app_state::MyApp;
use eframe::egui;

pub fn render(app: &mut MyApp, ui: &mut egui::Ui, _ctx: &egui::Context) {
    ui.heading("Weight Progress");
    // todo
    app.display_status_message(ui);
} 
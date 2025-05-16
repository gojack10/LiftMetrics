use crate::app_state::MyApp;
use eframe::egui;
use egui_extras::DatePickerButton;

pub fn render(app: &mut MyApp, ui: &mut egui::Ui, _ctx: &egui::Context) {
    ui.heading("Log Exercise");
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        ui.label("Workout Date:");
        ui.add(DatePickerButton::new(&mut app.log_exercise_date));
    });
    ui.label(format!("selected date: {}", app.log_exercise_date.format("%Y-%m-%d")));

    // todo: dynamic exercise list (current_exercises_log)
    // todo: "log workout" button

    ui.add_space(10.0);
    app.display_status_message(ui);
}

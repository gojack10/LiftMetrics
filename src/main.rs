use eframe::{App, NativeOptions};
use eframe::egui;
mod db_init;

struct LifeMetrics {}

impl App for LifeMetrics {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("LifeMetrics");
        });
    }
}

fn main() {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::Vec2::new(600.0, 800.0)),
        ..Default::default()
    };
    let database_path = "lifemetrics.db";
    if !std::path::Path::new(database_path).exists() {
        if let Err(e) = db_init::init(database_path) {
            eprintln!("Failed to initialize database: {}", e);
            return;
        }
    }
    let _ = eframe::run_native("LifeMetrics", options, Box::new(|_cc| Ok(Box::new(LifeMetrics {}))));
}
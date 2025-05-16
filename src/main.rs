use eframe::NativeOptions;
use eframe::egui;
use egui::{FontDefinitions, FontFamily, FontData};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use log::error;


mod db_init;
mod types;
mod app_state;
mod ui;
mod logging;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
enum Tab {
    #[default]
    LogWeight,
    LogExercise,
    WeightProgress,
    ExerciseProgress,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
enum DietPhase {
    #[default]
    Bulk,
    Cut,
    Maintain,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
enum ExerciseMetric {
    #[default]
    Weight,
}

fn main() {
    let (sender, receiver) = mpsc::channel();

    // Set up env_logger to use a custom writer that sends messages to the 'sender'
    logging::init_logger(sender);

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::Vec2::new(600.0, 800.0)),
        ..Default::default()
    };
    let database_path = "liftmetrics.db";
    if !std::path::Path::new(database_path).exists() {
        if let Err(e) = db_init::init(database_path) {
            error!("failed to initialize database: {}", e);
            return;
        }
    }

    let db_conn = match Connection::open(database_path) {
        Ok(conn) => Arc::new(Mutex::new(conn)),
        Err(e) => {
            error!("failed to open database connection: {}", e);
            return;
        }
    };

    let mut app = app_state::MyApp {
        db_conn: Some(db_conn.clone()),
        recent_weight_logs: Vec::new(),
        // Pass the receiver to the app state
        // TODO: Add receiver field to MyApp struct
        log_receiver: receiver,
        ..Default::default()
    };

    if let Ok(conn) = db_conn.lock() {
        match conn.query_row(
            "SELECT id FROM diet_cycles WHERE actual_end_date IS NULL ORDER BY start_date DESC LIMIT 1",
            [],
            |row| row.get(0),
        ) {
            Ok(active_id) => {
                app.active_diet_cycle_id = Some(active_id);
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // No active cycle, which is fine
            }
            Err(e) => error!("failed to load active diet cycle: {}", e),
        }

        let mut stmt = match conn.prepare("SELECT id, name FROM exercises ORDER BY name COLLATE NOCASE") {
            Ok(s) => s,
            Err(e) => {
                error!("failed to prepare statement for loading exercises: {}", e);
                return;
            }
        };
        
        let exercise_iter_map = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get::<usize, String>(1)?))
        });

        match exercise_iter_map {
            Ok(exercise_iter) => {
                for exercise_result in exercise_iter {
                    match exercise_result {
                        Ok((id, name_str)) => {
                            app.all_exercises_for_dropdown.push((id, name_str.clone()));
                        }
                        Err(e) => error!("failed to process exercise row: {}", e),
                    }
                }
            }
            Err(e) => {
                 error!("failed to query exercises: {}", e);
            }
        }
    } else {
        error!("failed to lock db connection for initial load.");
    }
    if app.active_diet_cycle_id.is_some() {
        app.fetch_recent_weight_logs();
    }

    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "JetBrainsMonoNerdFont".to_owned(),
        FontData::from_static(include_bytes!("ui/JetBrainsMonoNerdFont-Regular.ttf")).into(),
    );

    fonts.families.get_mut(&FontFamily::Proportional).unwrap()
        .insert(0, "JetBrainsMonoNerdFont".to_owned());
    fonts.families.get_mut(&FontFamily::Monospace).unwrap()
        .insert(0, "JetBrainsMonoNerdFont".to_owned());

    let _ = eframe::run_native("LiftMetrics", options, Box::new(|cc| {
        cc.egui_ctx.set_fonts(fonts);
        Ok(Box::new(app))
    }));
}

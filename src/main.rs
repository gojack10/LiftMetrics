use eframe::NativeOptions;
use eframe::egui;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use egui_extras::DatePickerButton;

mod db_init;
mod types;
mod app_state;
mod ui;

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
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::Vec2::new(600.0, 800.0)),
        ..Default::default()
    };
    let database_path = "lifemetrics.db";
    if !std::path::Path::new(database_path).exists() {
        if let Err(e) = db_init::init(database_path) {
            eprintln!("failed to initialize database: {}", e);
            return;
        }
    }

    let db_conn = match Connection::open(database_path) {
        Ok(conn) => Arc::new(Mutex::new(conn)),
        Err(e) => {
            eprintln!("failed to open database connection: {}", e);
            return;
        }
    };

    let mut app = app_state::MyApp {
        db_conn: Some(db_conn.clone()),
        recent_weight_logs: Vec::new(),
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
            Err(e) => eprintln!("failed to load active diet cycle: {}", e),
        }

        let mut stmt = match conn.prepare("SELECT id, name FROM exercises ORDER BY name COLLATE NOCASE") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("failed to prepare statement for loading exercises: {}", e);
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
                        Err(e) => eprintln!("failed to process exercise row: {}", e),
                    }
                }
            }
            Err(e) => {
                 eprintln!("failed to query exercises: {}", e);
            }
        }
    } else {
        eprintln!("failed to lock db connection for initial load.");
    }
    if app.active_diet_cycle_id.is_some() {
        app.fetch_recent_weight_logs();
    }

    let _ = eframe::run_native("LifeMetrics", options, Box::new(|_cc| Ok(Box::new(app))));
}

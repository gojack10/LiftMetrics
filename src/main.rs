use eframe::{App, NativeOptions};
use eframe::egui;
use egui_plot::PlotMemory;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use chrono::NaiveDate;
use chrono::Datelike;
use std::fmt::Display;
use egui_extras::DatePickerButton;

mod db_init;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
enum Tab {
    #[default]
    LogWeight,
    LogExercise,
    WeightProgress,
    ExerciseProgress,
}

impl Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
enum DietPhase {
    #[default]
    Bulk,
    Cut,
    Maintain,
}

impl Display for DietPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
enum ExerciseMetric {
    #[default]
    Weight,
    Reps,
    Volume,
}

impl Display for ExerciseMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default, Clone, Debug)]
struct SetEntry {
    reps: i32,
    weight_lbs: f64,
    rpe: Option<f64>,
    notes: Option<String>,
}

#[derive(Default, Clone, Debug)]
struct ExerciseLogEntry {
    exercise_name: String,
    sets: Vec<SetEntry>,
}


#[derive(Default)]
struct MyApp {
    active_tab: Tab,
    db_conn: Option<Arc<Mutex<Connection>>>,
    log_weight_input_lbs: String,
    show_diet_cycle_popup: bool,
    new_diet_phase: DietPhase,
    new_diet_start_date: String, // Should be YYYY-MM-DD
    new_diet_planned_end_date: String, // Should be YYYY-MM-DD
    active_diet_cycle_id: Option<i64>,
    log_exercise_date: NaiveDate, // Already initialized to today
    current_exercises_log: Vec<ExerciseLogEntry>,
    exercise_name_input_buffer: String,
    available_exercise_names: Vec<String>,
    weight_progress_data: Vec<(f64, f64)>,
    smoothed_weight_progress_data: Vec<(f64, f64)>,
    weight_plot_memory: PlotMemory,
    exercise_progress_selected_exercise_id: Option<i64>,
    all_exercises_for_dropdown: Vec<(i64, String)>,
    exercise_progress_data: Vec<(f64, f64)>,
    smoothed_exercise_progress_data: Vec<(f64, f64)>,
    exercise_plot_memory: PlotMemory,
    selected_exercise_metric: ExerciseMetric,
    status_message: String,
    last_status_time: Instant,
    recent_weight_logs: Vec<(String, f64)>, // For (log_date, weight_lbs)
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Potentially show diet cycle popup on top of everything if no active cycle
        if self.active_diet_cycle_id.is_none() && !self.show_diet_cycle_popup {
            // Automatically show popup if no active cycle and it's not already shown.
            // This could be triggered once after initial load.
            // For now, let's make it explicit via a button or on tab entry.
        }

        self.render_diet_cycle_popup(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("LifeMetrics");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, Tab::LogWeight, Tab::LogWeight.to_string());
                ui.selectable_value(&mut self.active_tab, Tab::LogExercise, Tab::LogExercise.to_string());
                ui.selectable_value(&mut self.active_tab, Tab::WeightProgress, Tab::WeightProgress.to_string());
                ui.selectable_value(&mut self.active_tab, Tab::ExerciseProgress, Tab::ExerciseProgress.to_string());
            });
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            match self.active_tab {
                Tab::LogWeight => self.render_log_weight_tab(ui, ctx),
                Tab::LogExercise => self.render_log_exercise_tab(ui, ctx),
                Tab::WeightProgress => self.render_weight_progress_tab(ui, ctx),
                Tab::ExerciseProgress => self.render_exercise_progress_tab(ui, ctx),
            }
        });
    }
}

impl MyApp {
    fn render_diet_cycle_popup(&mut self, ctx: &egui::Context) {
        if self.show_diet_cycle_popup {
            egui::Window::new("Setup New Diet Cycle")
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical_centered_justified(|ui| {
                        ui.label("Phase:");
                        egui::ComboBox::from_id_source("diet_phase_combo")
                            .selected_text(self.new_diet_phase.to_string())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.new_diet_phase, DietPhase::Bulk, DietPhase::Bulk.to_string());
                                ui.selectable_value(&mut self.new_diet_phase, DietPhase::Cut, DietPhase::Cut.to_string());
                                ui.selectable_value(&mut self.new_diet_phase, DietPhase::Maintain, DietPhase::Maintain.to_string());
                            });

                        ui.label("Start Date (YYYY-MM-DD):");
                        ui.text_edit_singleline(&mut self.new_diet_start_date);

                        ui.label("Planned End Date (YYYY-MM-DD):");
                        ui.text_edit_singleline(&mut self.new_diet_planned_end_date);

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked() {
                                // TODO: Validate and Save to DB
                                if let (Ok(start_date), Ok(planned_end_date)) = (
                                    NaiveDate::parse_from_str(&self.new_diet_start_date, "%Y-%m-%d"),
                                    NaiveDate::parse_from_str(&self.new_diet_planned_end_date, "%Y-%m-%d")
                                ) {
                                    if let Some(conn_mutex) = &self.db_conn {
                                        if let Ok(mut conn) = conn_mutex.lock() {
                                            match conn.execute(
                                                "INSERT INTO diet_cycles (phase, start_date, planned_end_date) VALUES (?1, ?2, ?3)",
                                                rusqlite::params![
                                                    self.new_diet_phase.to_string(),
                                                    start_date.format("%Y-%m-%d").to_string(),
                                                    planned_end_date.format("%Y-%m-%d").to_string()
                                                ],
                                            ) {
                                                Ok(_) => {
                                                    self.active_diet_cycle_id = Some(conn.last_insert_rowid());
                                                    self.status_message = "New diet cycle saved.".to_string();
                                                    self.last_status_time = Instant::now();
                                                    self.show_diet_cycle_popup = false;
                                                    self.fetch_recent_weight_logs(); // Fetch logs for the new cycle
                                                }
                                                Err(e) => {
                                                    self.status_message = format!("Error saving diet cycle: {}", e);
                                                    self.last_status_time = Instant::now();
                                                    eprintln!("Error saving diet cycle: {}", e);
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    self.status_message = "Invalid date format. Use YYYY-MM-DD.".to_string();
                                    self.last_status_time = Instant::now();
                                }
                            }
                            if ui.button("Cancel").clicked() {
                                self.show_diet_cycle_popup = false;
                                // Optionally reset fields
                                // self.new_diet_start_date = NaiveDate::today().format("%Y-%m-%d").to_string();
                                // self.new_diet_planned_end_date = NaiveDate::today().format("%Y-%m-%d").to_string();
                            }
                        });
                    });
                });
        }
    }

    fn render_log_weight_tab(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("Log Weight");
        if self.active_diet_cycle_id.is_none() {
            ui.label("No active diet cycle. Please set one up.");
            if ui.button("Setup Diet Cycle").clicked() {
                self.new_diet_start_date = NaiveDate::today().format("%Y-%m-%d").to_string();
                self.new_diet_planned_end_date = (NaiveDate::today() + chrono::Duration::days(90)).format("%Y-%m-%d").to_string();
                self.show_diet_cycle_popup = true;
            }
        } else {
            // Fetch recent logs if the tab is shown and logs are empty (e.g. first time or after cycle change)
            // A more robust way might be to track if a refresh is needed.
            if self.recent_weight_logs.is_empty() {
                 self.fetch_recent_weight_logs();
            }

            ui.label(format!("Active Diet Cycle ID: {}", self.active_diet_cycle_id.unwrap()));
            
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("Weight (lbs):");
                ui.text_edit_singleline(&mut self.log_weight_input_lbs).desired_width(100.0);
            });

            if ui.button("Log Weight").clicked() {
                if let Some(active_cycle_id) = self.active_diet_cycle_id {
                    match self.log_weight_input_lbs.trim().parse::<f64>() {
                        Ok(weight_val) => {
                            if weight_val > 0.0 {
                                if let Some(conn_mutex) = &self.db_conn {
                                    if let Ok(conn) = conn_mutex.lock() {
                                        let log_date_str = NaiveDate::today().format("%Y-%m-%d").to_string();
                                        match conn.execute(
                                            "INSERT INTO weight_logs (diet_cycle_id, log_date, weight_lbs) VALUES (?1, ?2, ?3)",
                                            rusqlite::params![active_cycle_id, log_date_str, weight_val],
                                        ) {
                                            Ok(_) => {
                                                self.status_message = format!("Weight {} lbs logged successfully.", weight_val);
                                                self.log_weight_input_lbs.clear();
                                                self.fetch_recent_weight_logs(); // Refresh recent logs
                                            }
                                            Err(e) => {
                                                self.status_message = format!("Error logging weight: {}", e);
                                                eprintln!("Error logging weight: {}", e);
                                            }
                                        }
                                    } else {
                                        self.status_message = "Failed to acquire database lock.".to_string();
                                    }
                                }
                            } else {
                                self.status_message = "Weight must be a positive number.".to_string();
                            }
                        }
                        Err(_) => {
                            self.status_message = "Invalid weight input. Please enter a number.".to_string();
                        }
                    }
                } else {
                    self.status_message = "No active diet cycle to log weight against.".to_string();
                }
                self.last_status_time = Instant::now();
            }

            if !self.recent_weight_logs.is_empty() {
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);
                ui.label("Recent Weight Logs:");
                egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                    egui::Grid::new("recent_weight_logs_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            for (date, weight) in &self.recent_weight_logs {
                                ui.label(date);
                                ui.label(format!("{:.1} lbs", weight));
                                ui.end_row();
                            }
                        });
                });
            }
        }
        self.display_status_message(ui);
    }

    fn fetch_recent_weight_logs(&mut self) {
        if let Some(active_cycle_id) = self.active_diet_cycle_id {
            if let Some(conn_mutex) = &self.db_conn {
                if let Ok(conn) = conn_mutex.lock() {
                    let mut stmt = match conn.prepare(
                        "SELECT log_date, weight_lbs FROM weight_logs 
                         WHERE diet_cycle_id = ?1 
                         ORDER BY log_date DESC, id DESC LIMIT 5", // Get last 5
                    ) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Failed to prepare statement for recent logs: {}", e);
                            self.status_message = format!("Error preparing to fetch recent logs: {}", e);
                            self.last_status_time = Instant::now();
                            return;
                        }
                    };
                    let logs_iter = match stmt.query_map(rusqlite::params![active_cycle_id], |row| {
                        Ok((row.get(0)?, row.get(1)?)) // (log_date: String, weight_lbs: f64)
                    }) {
                        Ok(iter) => iter,
                        Err(e) => {
                            eprintln!("Failed to query recent logs: {}", e);
                            self.status_message = format!("Error fetching recent logs: {}", e);
                            self.last_status_time = Instant::now();
                            return;
                        }
                    };

                    let mut new_logs = Vec::new();
                    for log_result in logs_iter {
                        match log_result {
                            Ok(log_entry) => {
                                new_logs.push(log_entry);
                            }
                            Err(e) => {
                                eprintln!("Error processing recent log row: {}", e);
                                self.status_message = format!("Error processing recent log: {}", e);
                                self.last_status_time = Instant::now();
                            }
                        }
                    }
                    self.recent_weight_logs = new_logs; // Replace old logs
                } else {
                    self.status_message = "Failed to acquire DB lock for recent logs.".to_string();
                    self.last_status_time = Instant::now();
                }
            }
        } else {
             self.recent_weight_logs.clear(); // No active cycle, so no recent logs
        }
    }

    fn render_log_exercise_tab(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("Log Exercise");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Workout Date:");
            ui.add(DatePickerButton::new(&mut self.log_exercise_date));
        });
        ui.label(format!("Selected date: {}", self.log_exercise_date.format("%Y-%m-%d")));


        // TODO: Dynamic Exercise List (current_exercises_log)
        // TODO: "Log Workout" Button

        ui.add_space(10.0);
        self.display_status_message(ui);
    }

    fn render_weight_progress_tab(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("Weight Progress");
        // TODO
        self.display_status_message(ui);
    }

    fn render_exercise_progress_tab(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("Exercise Progress");
        // TODO
        self.display_status_message(ui);
    }
    
    fn display_status_message(&mut self, ui: &mut egui::Ui) {
        if !self.status_message.is_empty() && self.last_status_time.elapsed().as_secs() < 5 {
            ui.add_space(5.0);
            ui.label(&self.status_message);
        } else if self.last_status_time.elapsed().as_secs() >= 5 {
            self.status_message.clear();
        }
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

    let db_conn = match Connection::open(database_path) {
        Ok(conn) => Arc::new(Mutex::new(conn)),
        Err(e) => {
            eprintln!("Failed to open database connection: {}", e);
            return;
        }
    };

    let mut app = MyApp {
        db_conn: Some(db_conn.clone()), // Clone Arc for app
        log_exercise_date: NaiveDate::today(),
        last_status_time: Instant::now(),
        recent_weight_logs: Vec::new(), // Initialize recent_weight_logs
        ..Default::default()
    };

    // Load initial state from DB
    if let Ok(mut conn) = db_conn.lock() { // Changed to mut conn for potential writes if needed, though not here
        // Check for active diet cycle
        match conn.query_row(
            "SELECT id FROM diet_cycles WHERE actual_end_date IS NULL ORDER BY start_date DESC LIMIT 1",
            [],
            |row| row.get(0),
        ) {
            Ok(active_id) => {
                app.active_diet_cycle_id = Some(active_id);
                // If an active cycle is found, fetch its recent logs
                // This requires `app` to be mutable here, or call a method on `app` that takes `&mut self`
                // For simplicity, we'll let the tab rendering handle initial fetch.
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // No active cycle, which is fine
            }
            Err(e) => eprintln!("Failed to load active diet cycle: {}", e),
        }

        // Pre-load exercise names
        let mut stmt = match conn.prepare("SELECT id, name FROM exercises ORDER BY name COLLATE NOCASE") {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to prepare statement for loading exercises: {}", e);
                // Continue without exercises if statement prep fails
                Vec::new(); // To satisfy type checker for all_exercises_for_dropdown
                Vec::new(); // To satisfy type checker for available_exercise_names
                stmt = conn.prepare("SELECT 1, 'dummy' WHERE 1=0").unwrap(); // dummy to satisfy type
                // return; // Or handle more gracefully
            }
        };
        
        let exercise_iter_map = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        });

        match exercise_iter_map {
            Ok(exercise_iter) => {
                for exercise_result in exercise_iter {
                    match exercise_result {
                        Ok((id, name_str)) => {
                            app.all_exercises_for_dropdown.push((id, name_str.clone()));
                            app.available_exercise_names.push(name_str);
                        }
                        Err(e) => eprintln!("Failed to process exercise row: {}", e),
                    }
                }
            }
            Err(e) => {
                 eprintln!("Failed to query exercises: {}", e);
            }
        }
        // Initial fetch of recent weight logs if an active cycle exists
        // This needs to be done after app is fully initialized and active_diet_cycle_id is set.
        // The current logic in render_log_weight_tab handles fetching if recent_weight_logs is empty.
    } else {
        eprintln!("Failed to lock DB connection for initial load.");
    }

    let _ = eframe::run_native("LifeMetrics", options, Box::new(|_cc| Ok(Box::new(app))));
}
            match exercise_result {
                Ok((id, name_str)) => {
                    app.all_exercises_for_dropdown.push((id, name_str.clone()));
                    app.available_exercise_names.push(name_str);
                }
                Err(e) => eprintln!("Failed to process exercise row: {}", e),
            }
        }
    } else {
        eprintln!("Failed to lock DB connection for initial load.");
    }

    let _ = eframe::run_native("LifeMetrics", options, Box::new(|_cc| Ok(Box::new(app))));
}

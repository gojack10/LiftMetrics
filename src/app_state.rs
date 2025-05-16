use crate::types::{Tab, DietPhase}; // ExerciseMetric not directly used yet here
use eframe::{App, egui};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use chrono::NaiveDate;

// Forward declare UI modules that will be called by impl App for MyApp
// This assumes a src/ui/mod.rs will exist and declare these submodules.
// pub mod ui; // This might be better in main.rs or lib.rs

pub struct MyApp {
    pub(crate) active_tab: Tab,
    pub(crate) db_conn: Option<Arc<Mutex<Connection>>>,
    pub(crate) log_weight_input_lbs: String,
    pub(crate) show_diet_cycle_popup: bool,
    pub(crate) new_diet_phase: DietPhase,
    pub(crate) new_diet_start_date: String, 
    pub(crate) new_diet_planned_end_date: String,
    pub(crate) active_diet_cycle_id: Option<i64>,
    pub(crate) log_exercise_date: NaiveDate,
    pub(crate) selected_weigh_in_date: NaiveDate, // Added for weigh-in date picker
    pub(crate) all_exercises_for_dropdown: Vec<(i64, String)>,
    pub(crate) status_message: String,
    pub(crate) last_status_time: Instant,
    pub(crate) recent_weight_logs: Vec<(String, f64)>,
    pub(crate) previous_active_tab: Option<Tab>, // Added to track tab changes for date reset, made pub(crate)
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            active_tab: Tab::default(),
            previous_active_tab: None, // Initialize previous_active_tab
            db_conn: None,
            log_weight_input_lbs: String::default(),
            show_diet_cycle_popup: false,
            new_diet_phase: DietPhase::default(),
            new_diet_start_date: String::default(),
            new_diet_planned_end_date: String::default(),
            active_diet_cycle_id: None,
            log_exercise_date: chrono::Local::now().date_naive(),
            selected_weigh_in_date: chrono::Local::now().date_naive(), // Initialize selected_weigh_in_date
            all_exercises_for_dropdown: Vec::default(),
            status_message: String::default(),
            last_status_time: Instant::now(),
            recent_weight_logs: Vec::default(),
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.active_diet_cycle_id.is_none() && !self.show_diet_cycle_popup {
            // Logic for automatically showing popup can be refined or triggered elsewhere
        }

        // Call the diet cycle popup renderer from the ui module
        crate::ui::popups::diet_cycle_popup::render(self, ctx);

        // Logic to reset weigh-in date when LogWeight tab becomes active
        if self.active_tab == Tab::LogWeight {
            if self.previous_active_tab.is_none() || self.previous_active_tab != Some(Tab::LogWeight) {
                self.selected_weigh_in_date = chrono::Local::now().date_naive();
            }
        }
        self.previous_active_tab = Some(self.active_tab);


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
                Tab::LogWeight => {
                    // selected_weigh_in_date is already reset if tab just became active
                    crate::ui::tabs::log_weight_tab::render(self, ui, ctx);
                }
                Tab::LogExercise => crate::ui::tabs::log_exercise_tab::render(self, ui, ctx),
                Tab::WeightProgress => crate::ui::tabs::weight_progress_tab::render(self, ui, ctx),
                Tab::ExerciseProgress => crate::ui::tabs::exercise_progress_tab::render(self, ui, ctx),
            }
        });
    }
}

impl MyApp {
    pub(crate) fn display_status_message(&mut self, ui: &mut egui::Ui) {
        if !self.status_message.is_empty() && self.last_status_time.elapsed().as_secs() < 5 {
            ui.add_space(5.0);
            ui.label(&self.status_message);
        } else if self.last_status_time.elapsed().as_secs() >= 5 {
            self.status_message.clear();
        }
    }

    pub(crate) fn fetch_recent_weight_logs(&mut self) {
        if let Some(active_cycle_id) = self.active_diet_cycle_id {
            if let Some(conn_mutex) = &self.db_conn {
                if let Ok(conn) = conn_mutex.lock() {
                    let mut stmt = match conn.prepare(
                        "SELECT log_date, weight_lbs FROM weight_logs 
                         WHERE diet_cycle_id = ?1 
                         ORDER BY log_date DESC, id DESC LIMIT 5",
                    ) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("failed to prepare statement for recent logs: {}", e);
                            self.status_message = format!("error preparing to fetch recent logs: {}", e);
                            self.last_status_time = Instant::now();
                            return;
                        }
                    };
                    let logs_iter = match stmt.query_map(rusqlite::params![active_cycle_id], |row| {
                        Ok((row.get(0)?, row.get(1)?))
                    }) {
                        Ok(iter) => iter,
                        Err(e) => {
                            eprintln!("failed to query recent logs: {}", e);
                            self.status_message = format!("error fetching recent logs: {}", e);
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
                                eprintln!("error processing recent log row: {}", e);
                                self.status_message = format!("error processing recent log: {}", e);
                                self.last_status_time = Instant::now();
                            }
                        }
                    }
                    self.recent_weight_logs = new_logs;
                } else {
                    self.status_message = "failed to acquire db lock for recent logs.".to_string();
                    self.last_status_time = Instant::now();
                }
            }
        } else {
             self.recent_weight_logs.clear();
        }
    }
}

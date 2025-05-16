use crate::app_state::MyApp;
use crate::types::DietPhase;
use eframe::egui;
use rusqlite;
use chrono::NaiveDate;
use std::time::Instant;
use log::error;

pub fn render(app: &mut MyApp, ctx: &egui::Context) {
    if app.show_diet_cycle_popup {
        egui::Window::new("Setup New Diet Cycle")
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    ui.label("Phase:");
                    egui::ComboBox::new("diet_phase_combo_salt", "")
                        .selected_text(app.new_diet_phase.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut app.new_diet_phase, DietPhase::Bulk, DietPhase::Bulk.to_string());
                            ui.selectable_value(&mut app.new_diet_phase, DietPhase::Cut, DietPhase::Cut.to_string());
                            ui.selectable_value(&mut app.new_diet_phase, DietPhase::Maintain, DietPhase::Maintain.to_string());
                        });

                    ui.label("Start Date (YYYY-MM-DD):");
                    ui.add(egui::TextEdit::singleline(&mut app.new_diet_start_date));

                    ui.label("Planned End Date (YYYY-MM-DD):");
                    ui.add(egui::TextEdit::singleline(&mut app.new_diet_planned_end_date));

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            if let (Ok(start_date), Ok(planned_end_date)) = (
                                NaiveDate::parse_from_str(&app.new_diet_start_date, "%Y-%m-%d"),
                                NaiveDate::parse_from_str(&app.new_diet_planned_end_date, "%Y-%m-%d")
                            ) {
                                let mut save_successful = false;
                                let mut new_active_id: Option<i64> = None;
                                let mut error_message: Option<String> = None;

                                if let Some(conn_mutex) = &app.db_conn {
                                    match conn_mutex.lock() {
                                        Ok(conn) => {
                                            match conn.execute(
                                                "INSERT INTO diet_cycles (phase, start_date, planned_end_date) VALUES (?1, ?2, ?3)",
                                                rusqlite::params![
                                                    app.new_diet_phase.to_string(),
                                                    start_date.format("%Y-%m-%d").to_string(),
                                                    planned_end_date.format("%Y-%m-%d").to_string()
                                                ],
                                            ) {
                                                Ok(_) => {
                                                    new_active_id = Some(conn.last_insert_rowid());
                                                    save_successful = true;
                                                }
                                                Err(e) => {
                                                    error_message = Some(format!("error saving diet cycle: {}", e));
                                                    error!("error saving diet cycle: {}", e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                             error_message = Some(format!("failed to acquire db lock: {}", e));
                                             error!("failed to acquire db lock: {}", e);
                                        }
                                    }
                                } 

                                if save_successful {
                                    app.active_diet_cycle_id = new_active_id;
                                    app.console_messages.push(format!("[STATUS] new diet cycle saved.\n"));
                                    app.last_status_time = Instant::now();
                                    app.show_diet_cycle_popup = false;
                                    app.fetch_recent_weight_logs();
                                } else if let Some(msg) = error_message {
                                    app.console_messages.push(format!("[STATUS] {}\n", msg));
                                    app.last_status_time = Instant::now();
                                } else {
                                    // This case might need more specific error handling if there are other failure modes
                                    app.console_messages.push(format!("[STATUS] failed to save diet cycle.\n"));
                                    app.last_status_time = Instant::now();
                                }
                            } else {
                                app.console_messages.push(format!("[STATUS] invalid date format. use yyyy-mm-dd.\n"));
                                app.last_status_time = Instant::now();
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            app.show_diet_cycle_popup = false;
                        }
                    });
                });
            });
    }
}

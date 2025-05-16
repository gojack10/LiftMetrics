use crate::app_state::MyApp;
use eframe::egui;
use egui_extras::DatePickerButton; // Added for date picker
use rusqlite;
use chrono;
use std::time::Instant;

pub fn render(app: &mut MyApp, ui: &mut egui::Ui, _ctx: &egui::Context) {
    ui.heading("Log Weight");
    if app.active_diet_cycle_id.is_none() {
        ui.label("no active diet cycle. please set one up.");
        if ui.button("Setup Diet Cycle").clicked() {
            app.new_diet_start_date = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
            app.new_diet_planned_end_date = (chrono::Local::now().date_naive() + chrono::Duration::days(90)).format("%Y-%m-%d").to_string();
            app.show_diet_cycle_popup = true;
        }
    } else {
        if app.recent_weight_logs.is_empty() {
             app.fetch_recent_weight_logs();
        }

        ui.label(format!("active diet cycle id: {}", app.active_diet_cycle_id.unwrap()));
        
        ui.add_space(10.0);

        // Date Picker for Weigh-in Date
        ui.horizontal(|ui| {
            ui.label("Weigh-in Date:");
            ui.add(DatePickerButton::new(&mut app.selected_weigh_in_date));
        });
        ui.add_space(5.0); // Add a little space after the date picker

        ui.horizontal(|ui| {
            ui.label("Weight (lbs):");
            ui.add(egui::TextEdit::singleline(&mut app.log_weight_input_lbs).desired_width(100.0));
        });

        if ui.button("Log Weight").clicked() {
            if let Some(active_cycle_id) = app.active_diet_cycle_id {
                match app.log_weight_input_lbs.trim().parse::<f64>() {
                    Ok(weight_val) => {
                        if weight_val > 0.0 {
                            let mut log_successful = false;
                            let mut db_error_message: Option<String> = None;

                            if let Some(conn_mutex) = &app.db_conn {
                                match conn_mutex.lock() {
                                    Ok(conn) => {
                                        // Use the selected weigh-in date
                                        let log_date_str = app.selected_weigh_in_date.format("%Y-%m-%d").to_string();
                                        match conn.execute(
                                            "INSERT INTO weight_logs (diet_cycle_id, log_date, weight_lbs) VALUES (?1, ?2, ?3)",
                                            rusqlite::params![active_cycle_id, log_date_str, weight_val],
                                        ) {
                                            Ok(_) => {
                                                log_successful = true;
                                            }
                                            Err(e) => {
                                                db_error_message = Some(format!("error logging weight: {}", e));
                                                eprintln!("error logging weight: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        db_error_message = Some(format!("failed to acquire database lock: {}", e));
                                        eprintln!("failed to acquire database lock: {}", e);
                                    }
                                }
                            }

                            if log_successful {
                                app.status_message = format!("weight {} lbs logged successfully.", weight_val);
                                app.log_weight_input_lbs.clear();
                                app.fetch_recent_weight_logs();
                            } else if let Some(msg) = db_error_message {
                                app.status_message = msg;
                            } else {
                                app.status_message = "failed to log weight: db connection not available.".to_string();
                            }
                        } else {
                            app.status_message = "weight must be a positive number.".to_string();
                        }
                    }
                    Err(_) => {
                        app.status_message = "invalid weight input. please enter a number.".to_string();
                    }
                }
            } else {
                app.status_message = "no active diet cycle to log weight against.".to_string();
            }
            app.last_status_time = Instant::now();
        }

        if !app.recent_weight_logs.is_empty() {
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
                        for (date, weight) in &app.recent_weight_logs {
                            ui.label(date);
                            ui.label(format!("{:.1} lbs", weight));
                            ui.end_row();
                        }
                    });
            });
        }
    }
    app.display_status_message(ui);
}

use eframe::egui;
use crate::app_state::MyApp;

pub fn render(app: &mut MyApp, ctx: &egui::Context) {
    let panel_frame = egui::Frame::none().fill(egui::Color32::BLACK); // Set background to black

    egui::TopBottomPanel::bottom("console_panel").exact_height(200.0).frame(panel_frame).show(ctx, |ui| {
        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
            ui.vertical(|ui| {
                for message_str in &app.console_messages { // renamed for clarity
                    ui.horizontal(|ui| {
                        let mut display_message = message_str.trim_start();

                        // part 1: the prompt "$"
                        // messages from the logger already include "$ ", so we check for that.
                        // otherwise, we add the prompt.
                        if display_message.starts_with("$ ") {
                            ui.label(egui::RichText::new("$").color(egui::Color32::GREEN).monospace());
                            display_message = &display_message[2..]; // remove "$ " from message to be processed
                        } else {
                            // add the prompt if not present in the message string itself
                            ui.label(egui::RichText::new("$").color(egui::Color32::GREEN).monospace());
                        }
                        ui.add_space(4.0); // always add a small space after the prompt

                        // part 2: the level "[level]" (light blue) and part 3: text (white)
                        let mut level_processed = false;
                        if display_message.starts_with('[') {
                            if let Some(end_bracket_idx) = display_message.find(']') {
                                // a level is valid if it's "[somelevel]" followed by a space, or just "[somelevel]" at the end.
                                let is_followed_by_space_or_is_end = display_message.len() == end_bracket_idx + 1 ||
                                                                     (display_message.len() > end_bracket_idx + 1 && display_message.chars().nth(end_bracket_idx + 1) == Some(' '));

                                if is_followed_by_space_or_is_end {
                                    let level_text = &display_message[..=end_bracket_idx];
                                    ui.label(egui::RichText::new(level_text).color(egui::Color32::LIGHT_BLUE).monospace());

                                    let rest_of_message = display_message[end_bracket_idx + 1..].trim_start();
                                    if !rest_of_message.is_empty() {
                                        ui.add_space(4.0); // space after level, before text
                                        ui.label(egui::RichText::new(rest_of_message).color(egui::Color32::WHITE).monospace());
                                    }
                                    level_processed = true;
                                }
                            }
                        }

                        if !level_processed {
                            // if no level was processed (or found in the right format),
                            // print the remaining display_message as white text.
                            // this handles plain text messages or messages where [level] isn't followed by space.
                            if !display_message.is_empty() {
                                ui.label(egui::RichText::new(display_message).color(egui::Color32::WHITE).monospace());
                            }
                        }
                    });
                }
            });
        });
    });
}


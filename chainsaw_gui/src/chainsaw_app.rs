// src/chainsaw_app.rs

use eframe::{egui, App, NativeOptions};
use native_dialog::FileDialog;
use petgraph::graph::{DiGraph, NodeIndex};
use std::path::PathBuf;
use crate::utils::{ChainsawEvent, correlate_events, read_csv_files, parse_timestamp};
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct ChainsawApp {
    csv_file_paths: Vec<PathBuf>,
    csv_loaded: bool,
    graph: DiGraph<(), ()>,
    loaded_events: Vec<ChainsawEvent>,
    selected_type: Option<String>,
    sort_criteria: SortCriteria,
    search_query: String,
    unique_types: HashSet<String>, // Add the unique_types field
    delta: Duration, // Add the delta field
}

#[derive(Clone, PartialEq)]
enum SortCriteria {
    Date(Duration),
    SID,
    Path,
}

impl Default for SortCriteria {
    fn default() -> Self {
        SortCriteria::Date(Duration::from_secs(0))
    }
}

impl Default for ChainsawApp {
    fn default() -> Self {
        Self {
            csv_file_paths: vec![],
            csv_loaded: false,
            graph: DiGraph::new(),
            loaded_events: Vec::new(),
            selected_type: None,
            sort_criteria: SortCriteria::Date(Duration::from_secs(0)),
            search_query: String::new(), // Add search_query initialization
            unique_types: HashSet::new(),
            delta: Duration::from_secs(0),
        }
    }
}


impl eframe::App for ChainsawApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                if !self.csv_loaded {
                    if ui.button("Load CSV files").clicked() {
                        self.load_csv_files();
                    }
                }

                if self.csv_loaded {
                    ui.separator();
                    ui.heading("Event Type Selection");

                    // Add "Select All" checkbox
                    let mut select_all = self.selected_type.is_none();
                    if ui.checkbox(&mut select_all, "Select All").clicked() {
                        if select_all {
                            self.selected_type = None;
                        } else {
                            self.selected_type = Some(String::new());
                        }
                    }

                    if !select_all {
                        // Dropdown for selecting event type
                        egui::ComboBox::from_label("Event Type")
                            .selected_text(self.selected_type.clone().unwrap_or_default())
                            .show_ui(ui, |ui| {
                                for event in &self.loaded_events {
                                    if let Some(detection) = &event.detections {
                                        ui.selectable_value(
                                            &mut self.selected_type,
                                            Some(detection.clone()),
                                            detection,
                                        );
                                    }
                                }
                            });
                    }

                    ui.separator();
                    ui.heading("Sort Criteria");

                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.sort_criteria, SortCriteria::Date(Duration::from_secs(3600)), "Date");
                        ui.selectable_value(&mut self.sort_criteria, SortCriteria::SID, "SID");
                        ui.selectable_value(&mut self.sort_criteria, SortCriteria::Path, "Path");
                    });

                    ui.separator();
                    ui.heading("Search");

                    // Add search field with autocomplete
                    ui.horizontal(|ui| {
                        ui.label("Search:");
                        ui.text_edit_singleline(&mut self.search_query);
                    });

                    // Autocomplete suggestions
                    if !self.search_query.is_empty() {
                        let suggestions: Vec<&str> = self
                            .loaded_events
                            .iter()
                            .flat_map(|event| {
                                vec![
                                    event.timestamp.as_deref(),
                                    event.detections.as_deref(),
                                    event.path.as_deref(),
                                    event.computer.as_deref(),
                                    event.user.as_deref(),
                                    event.user_sid.as_deref(),
                                    event.member_sid.as_deref(),
                                ]
                            })
                            .filter_map(|field| field)
                            .filter(|&field| field.contains(&self.search_query))
                            .collect();

                        for suggestion in suggestions {
                            if ui.button(suggestion).clicked() {
                                self.search_query = suggestion.to_string();
                            }
                        }
                    }

                    ui.separator();
                    self.display_sorted_events(ui);
                }
            });
        });
    }
}


impl ChainsawApp {
    pub fn load_csv_files(&mut self) {
        match FileDialog::new()
            .add_filter("CSV Files", &["csv"])
            .show_open_multiple_file()
        {
            Ok(file_paths) => {
                println!("Selected files: {:?}", file_paths);

                // Read CSV files into events
                match read_csv_files(&file_paths) {
                    Ok(events) => {
                        self.loaded_events = events;
                        self.csv_loaded = true; // Set the flag to indicate files are loaded
                        self.extract_unique_types();
                    }
                    Err(e) => {
                        println!("Error reading CSV files: {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("File dialog encountered an error: {:?}", e);
            }
        }
    }

    fn extract_unique_types(&mut self) {
        self.unique_types = self.loaded_events.iter()
            .filter_map(|event| event.detections.clone())
            .collect();
    }

    fn display_csv_file_types(&mut self, ui: &mut egui::Ui) {
        ui.label("Select CSV file type:");
        for csv_type in &self.unique_types {
            if ui.button(csv_type).clicked() {
                self.selected_type = Some(csv_type.clone());
            }
        }
        if ui.button("Select All").clicked() {
            self.selected_type = None;
        }
    }

    fn display_sorting_options(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Sort by:");
            if ui.button("Date").clicked() {
                // Let user enter delta value
                let mut delta_secs = self.delta.as_secs() as i32;
                ui.add(egui::DragValue::new(&mut delta_secs).speed(1).clamp_range(0..=3600));
                self.delta = Duration::from_secs(delta_secs as u64);
                self.sort_criteria = SortCriteria::Date(self.delta);
            }
            if ui.button("SID").clicked() {
                self.sort_criteria = SortCriteria::SID;
            }
            if ui.button("Path").clicked() {
                self.sort_criteria = SortCriteria::Path;
            }
        });
    }
}
impl ChainsawApp {
    fn display_sorted_events(&self, ui: &mut egui::Ui) {
        // Filter events based on the selected type
        let filtered_events: Vec<&ChainsawEvent> = match &self.selected_type {
            Some(selected_type) if !selected_type.is_empty() => self.loaded_events.iter()
                .filter(|event| event.detections.as_deref() == Some(selected_type))
                .collect(),
            _ => self.loaded_events.iter().collect(),
        };

        // Filter events based on the search query
        let filtered_events: Vec<&ChainsawEvent> = filtered_events.into_iter()
            .filter(|event| {
                event.timestamp.as_deref().unwrap_or_default().contains(&self.search_query) ||
                event.detections.as_deref().unwrap_or_default().contains(&self.search_query) ||
                event.path.as_deref().unwrap_or_default().contains(&self.search_query) ||
                event.computer.as_deref().unwrap_or_default().contains(&self.search_query) ||
                event.user.as_deref().unwrap_or_default().contains(&self.search_query) ||
                event.user_sid.as_deref().unwrap_or_default().contains(&self.search_query) ||
                event.member_sid.as_deref().unwrap_or_default().contains(&self.search_query)
            })
            .collect();

        // Sort events based on the selected criteria
        let mut sorted_events = filtered_events.to_vec();
        match self.sort_criteria {
            SortCriteria::Date(delta) => {
                sorted_events.sort_by(|a, b| {
                    let ts_a = parse_timestamp(a.timestamp.as_deref().unwrap_or_default()).unwrap_or(Utc::now());
                    let ts_b = parse_timestamp(b.timestamp.as_deref().unwrap_or_default()).unwrap_or(Utc::now());
                    (ts_a - ts_b).num_seconds().abs().cmp(&(delta.as_secs() as i64))
                });
            }
            SortCriteria::SID => {
                sorted_events.sort_by(|a, b| a.user_sid.cmp(&b.user_sid));
            }
            SortCriteria::Path => {
                sorted_events.sort_by(|a, b| a.path.cmp(&b.path));
            }
        }

        // Display sorted events in columns
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.columns(9, |columns| {
                columns[0].label("Timestamp");
                columns[1].label("Detections");
                columns[2].label("Path");
                columns[3].label("Event ID");
                columns[4].label("Record ID");
                columns[5].label("Computer");
                columns[6].label("User");
                columns[7].label("User SID");
                columns[8].label("Member SID");

                for event in sorted_events {
                    columns[0].label(event.timestamp.as_deref().unwrap_or_default());
                    columns[1].label(event.detections.as_deref().unwrap_or_default());
                    columns[2].label(event.path.as_deref().unwrap_or_default());
                    columns[3].label(&event.event_id.map_or(String::new(), |id| id.to_string()));
                    columns[4].label(&event.record_id.map_or(String::new(), |id| id.to_string()));
                    columns[5].label(event.computer.as_deref().unwrap_or_default());
                    columns[6].label(event.user.as_deref().unwrap_or_default());
                    columns[7].label(event.user_sid.as_deref().unwrap_or_default());
                    columns[8].label(event.member_sid.as_deref().unwrap_or_default());
                }
            });
        });
    }
}


impl ChainsawApp {
    pub fn run() {
        let app = ChainsawApp::default();
        let native_options = NativeOptions::default();

        eframe::run_native(
            "ChainsawApp",
            native_options,
            Box::new(move |_ctx| Ok(Box::new(app.clone()) as Box<dyn App>)),
        ).expect("Failed to run native");
    }
}

use eframe::{egui, App, NativeOptions};
use native_dialog::FileDialog;
use petgraph::graph::{DiGraph, NodeIndex};
use std::path::PathBuf;
use crate::utils::{ChainsawEvent, correlate_events, read_csv_files};
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc, Duration}; // Import chrono::Duration
use petgraph::visit::EdgeRef;
use chrono::FixedOffset;
use egui::Ui;

#[derive(Clone)]
pub struct ChainsawApp {
    csv_file_paths: Vec<PathBuf>,
    csv_loaded: bool,
    loaded_events: Vec<ChainsawEvent>,
    selected_type: Option<String>,
    sort_criteria: SortCriteria,
    search_query: String,
    unique_types: HashSet<String>,
    delta: Duration, // Use chrono::Duration
    show_correlated_events: bool,
    correlated_graph: Option<DiGraph<ChainsawEvent, ()>>, // Use ChainsawEvent for nodes
    selected_node: Option<NodeIndex>, // Add field for selected node
    state_stack: Vec<AppState>, // Stack for managing states
}

#[derive(Clone, PartialEq)]
enum SortCriteria {
    Date(Duration),
    SID,
    Path,
    event_id,
    computer,
    user,
}

#[derive(Clone)]
enum AppState {
    Main,
    SearchResults,
    CorrelatedEvents,
    EventDetails(NodeIndex),
}

impl Default for SortCriteria {
    fn default() -> Self {
        SortCriteria::Date(Duration::seconds(0))
    }
}

impl Default for ChainsawApp {
    fn default() -> Self {
        Self {
            csv_file_paths: vec![],
            csv_loaded: false,
            loaded_events: Vec::new(),
            selected_type: None,
            sort_criteria: SortCriteria::Date(Duration::seconds(0)), // Set default delta to 1 minute
            search_query: String::new(),
            unique_types: HashSet::new(),
            delta: Duration::minutes(1), // Set default delta to 1 minute
            show_correlated_events: false,
            correlated_graph: None,
            selected_node: None, // Initialize selected_node
            state_stack: vec![AppState::Main], // Initialize with the main state
        }
    }
}

impl eframe::App for ChainsawApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                match self.state_stack.last().unwrap_or(&AppState::Main) {
                    AppState::Main => self.display_main(ui),
                    AppState::SearchResults => self.display_search_results(ui),
                    AppState::CorrelatedEvents => self.display_correlated_events(ui),
                    AppState::EventDetails(node) => self.display_event_details(ui, *node),
                }

                if self.state_stack.len() > 1 {
                    if ui.button("Back").clicked() {
                        self.state_stack.pop();
                    }
                }
            });
        });
    }
}

impl ChainsawApp {
    fn display_main(&mut self, ui: &mut Ui) {
        if !self.csv_loaded {
            if ui.button("Load CSV files").clicked() {
                self.load_csv_files();
            }
        }

        if self.csv_loaded {
            ui.separator();
            ui.heading("Event Type Selection");

            let mut select_all = self.selected_type.is_none();
            if ui.checkbox(&mut select_all, "Select All").clicked() {
                if select_all {
                    self.selected_type = None;
                } else {
                    self.selected_type = Some(String::new());
                }
            }

            if !select_all {
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
                ui.selectable_value(&mut self.sort_criteria, SortCriteria::Date(Duration::hours(1)), "Date");
                ui.selectable_value(&mut self.sort_criteria, SortCriteria::SID, "SID");
                ui.selectable_value(&mut self.sort_criteria, SortCriteria::Path, "Path");
                ui.selectable_value(&mut self.sort_criteria, SortCriteria::user, "User");
                ui.selectable_value(&mut self.sort_criteria, SortCriteria::computer, "Computer");
            });

            ui.separator();
            ui.heading("Search");

            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.search_query);
            });

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

            // Button to show correlated events
            if ui.button("Show Correlated Events").clicked() {
                self.show_correlated_events = true;
                self.correlated_graph = Some(correlate_events(&self.loaded_events, self.delta));
                self.state_stack.push(AppState::CorrelatedEvents); // Push state to stack
            }

            if !self.show_correlated_events {
                self.display_sorted_events(ui);
            }
        }
    }

    fn display_search_results(&mut self, ui: &mut Ui) {
        ui.label("Search Results");
        // Implement search results display logic
    }

    fn display_correlated_events(&mut self, ui: &mut Ui) {
        if let Some(graph) = &self.correlated_graph {
            let mut clicked_node = None;

            egui::ScrollArea::vertical().show(ui, |ui| {
                for node_index in graph.node_indices() {
                    let node_label = format!("Node {}", node_index.index() + 1);

                    // Create a button for each node
                    let button_response = ui.button(&node_label);

                    // Check if the button is clicked
                    if button_response.clicked() {
                        clicked_node = Some(node_index);
                        println!("Node {} clicked", node_index.index() + 1);
                    }

                    // Display node label horizontally
                    ui.horizontal(|ui| {
                        ui.label(&node_label);
                    });
                }
            });

            if let Some(node) = clicked_node {
                self.state_stack.push(AppState::EventDetails(node));
            }
        } else {
            ui.label("No correlated events found.");
        }
    }

    fn display_event_details(&mut self, ui: &mut Ui, node: NodeIndex) {
        if let Some(graph) = &self.correlated_graph {
            let node_event = &graph[node];
            ui.label("Event Details:");
            self.display_node_details(ui, node_event);
            ui.separator();
            ui.label("Related Events:");

            for edge in graph.edges_directed(node, petgraph::Direction::Outgoing) {
                let target_node_index = edge.target();
                let edge_label = format!("{} -> {}", node.index() + 1, target_node_index.index() + 1);

                ui.horizontal(|ui| {
                    ui.label(&edge_label);
                    if ui.button("Show Details").clicked() {
                        self.state_stack.push(AppState::EventDetails(target_node_index));
                    }
                });
            }
        }
    }

    fn display_node_details(&self, ui: &mut Ui, node_event: &ChainsawEvent) {
        ui.horizontal(|ui| {
            ui.label("Timestamp:");
            if let Some(timestamp) = &node_event.timestamp {
                ui.label(timestamp);
            } else {
                ui.label("N/A");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Detections:");
            if let Some(detections) = &node_event.detections {
                ui.label(detections);
            } else {
                ui.label("N/A");
            }
        });
        // Add similar blocks for other fields of ChainsawEvent
    }

    fn display_sorted_events(&self, ui: &mut egui::Ui) {
        let filtered_events: Vec<&ChainsawEvent> = match &self.selected_type {
            Some(selected_type) if !selected_type.is_empty() => self.loaded_events.iter()
                .filter(|event| event.detections.as_deref() == Some(selected_type))
                .collect(),
            _ => self.loaded_events.iter().collect(),
        };

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

        let mut sorted_events = filtered_events.to_vec();
        match self.sort_criteria {
            SortCriteria::Date(delta) => {
                sorted_events.sort_by(|a, b| {
                    let ts_a = Self::parse_timestamp(a.timestamp.as_deref().unwrap_or_default()).unwrap_or(Utc::now());
                    let ts_b = Self::parse_timestamp(b.timestamp.as_deref().unwrap_or_default()).unwrap_or(Utc::now());
                    (ts_a - ts_b).num_seconds().abs().cmp(&(delta.num_seconds()))
                });
            }
            SortCriteria::SID => {
                sorted_events.sort_by(|a, b| a.user_sid.cmp(&b.user_sid));
            }
            SortCriteria::Path => {
                sorted_events.sort_by(|a, b| a.path.cmp(&b.path));
            }
            
            SortCriteria::event_id => {
                sorted_events.sort_by(|a, b| a.event_id.cmp(&b.event_id));
            }
            SortCriteria::computer => {
                sorted_events.sort_by(|a, b| a.computer.cmp(&b.computer));
            }
            
            SortCriteria::user => {
                sorted_events.sort_by(|a, b| a.user.cmp(&b.user));
            }
        }

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

    fn parse_timestamp(ts: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
        let parsed_datetime: DateTime<FixedOffset> = DateTime::parse_from_rfc3339(ts)?;
        let datetime_utc: DateTime<Utc> = parsed_datetime.into();
        Ok(datetime_utc)
    }

    fn load_csv_files(&mut self) {
        match FileDialog::new()
            .add_filter("CSV Files", &["csv"])
            .show_open_multiple_file()
        {
            Ok(file_paths) => {
                println!("Selected files: {:?}", file_paths);

                match read_csv_files(&file_paths) {
                    Ok(events) => {
                        self.loaded_events = events;
                        self.csv_loaded = true;
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

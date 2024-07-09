// chainsaw_app.rs

use eframe::{egui, App, Frame, NativeOptions};
use native_dialog::FileDialog;
use petgraph::graph::{DiGraph, NodeIndex};
use std::path::PathBuf;
use egui::Shape::Circle as EguiCircle;
use egui::style::HandleShape::Circle;
use petgraph::visit::EdgeRef;
use crate::utils::{ChainsawEvent, correlate_events, read_csv_files};



#[derive(Clone)]
pub struct ChainsawApp {
    csv_file_paths: Vec<PathBuf>,
    csv_loaded: bool,
    graph: DiGraph<NodeIndex, ()>,
    loaded_events: Vec<ChainsawEvent>,
}

impl Default for ChainsawApp {
    fn default() -> Self {
        Self {
            csv_file_paths: vec![],
            csv_loaded: false,
            graph: DiGraph::new(),
            loaded_events: Vec::new(),
        }
    }
}
impl eframe::App for ChainsawApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.label("Chainsaw GUI");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.csv_loaded {
                if ui.button("Load CSV files").clicked() {
                    self.load_csv_files();
                    println!("CSV files loaded"); // Debug print
                    println!("Events count: {}", self.loaded_events.len()); // Debug print
                }
            }

            if self.csv_loaded {
                println!("Displaying nodes and edges"); // Debug print

                // Example: correlate events into graph
                self.graph = correlate_events(&self.loaded_events);
                println!("Nodes count: {}", self.graph.node_count()); // Debug print
                println!("Edges count: {}", self.graph.edge_count()); // Debug print

                // Display nodes with their names
                for node_index in self.graph.node_indices() {
                    if let Some(event) = self.loaded_events.get(node_index.index()) {
                        let node_name = format!(
                            "{} - {}",
                            event.timestamp.as_deref().unwrap_or_default(),
                            event.path.as_deref().unwrap_or_default()
                        );
                        ui.label(node_name);
                    } else {
                        ui.label(format!("Node {}", node_index.index()));
                    }
                }

                // Draw lines or arrows between nodes
                for edge in self.graph.edge_references() {
                    let _source_index = edge.source();
                    let _target_index = edge.target();

                    // Example fixed positions for simplicity
                    let source_pos = egui::Pos2::new(50.0, 50.0);
                    let target_pos = egui::Pos2::new(150.0, 150.0);

                    let points = [source_pos, target_pos];
                    let stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);

                    // Draw a line segment between source and target
                    ui.painter().line_segment(points, stroke);
                }
            }
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

                // Convert Vec<PathBuf> to Vec<String>
                let file_paths_str: Vec<String> = file_paths
                    .iter()
                    .map(|path| path.to_string_lossy().to_string())
                    .collect();

                // Read CSV files into events
                match read_csv_files(&file_paths_str) {
                    Ok(events) => {
                        self.loaded_events = events;
                        self.csv_loaded = true; // Set the flag to indicate files are loaded
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

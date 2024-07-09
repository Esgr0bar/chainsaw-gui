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
    graph: DiGraph<(), ()>, // Changed DiGraph to use unit type for node and edge data
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
                }
            }

            if self.csv_loaded {
                // Correlate events into graph if not already correlated
                if self.graph.node_count() == 0 {
                    self.graph = correlate_events(&self.loaded_events);
                }

                // Calculate positions for nodes in a circular layout
                let node_count = self.graph.node_count() as f32;
                let center = egui::Pos2::new(300.0, 300.0);
                let radius = 200.0;
                let angle_step = 2.0 * std::f32::consts::PI / node_count;

                // Draw nodes in a circular layout
                for (i, node_index) in self.graph.node_indices().enumerate() {
                    // Calculate position for each node
                    let angle = angle_step * i as f32;
                    let x = center.x + radius * angle.cos();
                    let y = center.y + radius * angle.sin();
                    let node_pos = egui::Pos2::new(x, y);

                    // Display node name near the circle
                    let label_pos = egui::Pos2::new(x + 20.0, y); // Adjust label position as needed
                    let node_name = if let Some(event) = self.loaded_events.get(node_index.index()) {
                        format!(
                            "{} - {}",
                            event.timestamp.as_deref().unwrap_or_default(),
                            event.path.as_deref().unwrap_or_default()
                        )
                    } else {
                        format!("Node {}", node_index.index())
                    };

                    // Use ui.label to display the node name
                    ui.label(node_name.clone()).interact_rect.left_top();

                    // Draw connections (edges) between nodes
                    for edge in self.graph.edges(node_index) {
                        if let target_index = edge.target() {
                            // Calculate target position
                            if let Some(target_pos) = self.node_position(target_index, node_count, center, radius, angle_step) {
                                let points = [node_pos, target_pos];
                                let stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);
                                ui.painter().line_segment(points, stroke);
                            }
                        }
                    }
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

    fn node_position(&self, node_index: NodeIndex, node_count: f32, center: egui::Pos2, radius: f32, angle_step: f32) -> Option<egui::Pos2> {
        if node_index.index() < node_count as usize {
            let angle = angle_step * node_index.index() as f32;
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            Some(egui::Pos2::new(x, y))
        } else {
            None
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

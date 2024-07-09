use eframe::egui::{CentralPanel, TopBottomPanel, ScrollArea, Context};
use eframe::{App, NativeOptions, Frame};
use native_dialog::FileDialog;
use petgraph::graph::DiGraph;
use std::path::PathBuf;
use crate::utils::{read_csv_files, correlate_events, ChainsawEvent};

pub struct ChainsawApp {
    csv_file_paths: Vec<PathBuf>,
    events: Vec<ChainsawEvent>,
    csv_loaded: bool,
    graph: DiGraph<(), ()>, // Assuming DiGraph is from petgraph
}

impl Default for ChainsawApp {
    fn default() -> Self {
        Self {
            csv_file_paths: vec![],
            events: vec![],
            csv_loaded: false,
            graph: DiGraph::new(),
        }
    }
}

impl Clone for ChainsawApp {
    fn clone(&self) -> Self {
        Self {
            csv_file_paths: self.csv_file_paths.clone(),
            events: self.events.clone(),
            csv_loaded: self.csv_loaded,
            graph: self.graph.clone(),
        }
    }
}

impl App for ChainsawApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.label("Chainsaw GUI");
        });

        CentralPanel::default().show(ctx, |ui| {
            if ui.button("Load CSV files").clicked() {
                self.load_csv_files();
            }

            if !self.csv_file_paths.is_empty() && !self.csv_loaded {
                println!("CSV file paths: {:?}", self.csv_file_paths);
                let paths: Vec<String> = self.csv_file_paths.iter().map(|p| p.to_string_lossy().to_string()).collect();
                match read_csv_files(&paths) {
                    Ok(events) => {
                        println!("Parsed events: {:?}", events);
                        self.events = events;
                        self.csv_loaded = true; // Set the flag to indicate files have been loaded
                    }
                    Err(e) => {
                        println!("Failed to read CSV files: {:?}", e);
                        self.csv_loaded = false; // Ensure we retry if the user attempts to load again
                    }
                }
            }

            if !self.events.is_empty() {
                self.graph = correlate_events(&self.events);
                println!("Correlated events: {:?}", self.graph);
                ScrollArea::both().show(ui, |ui| {
                    ui.label(format!("{:?}", self.graph));
                });
            }
        });
    }
}

impl ChainsawApp {
    pub fn load_csv_files(&mut self) {
        match FileDialog::new().add_filter("CSV Files", &["csv"]).show_open_multiple_file() {
            Ok(file_paths) => {
                println!("Selected files: {:?}", file_paths);
                self.csv_file_paths = file_paths;
                self.csv_loaded = false; // Reset the flag to indicate files need to be loaded
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

    pub fn update_events(&mut self) {
        if !self.csv_file_paths.is_empty() && !self.csv_loaded {
            println!("CSV file paths: {:?}", self.csv_file_paths);
            let paths: Vec<String> = self.csv_file_paths.iter().map(|p| p.to_string_lossy().to_string()).collect();
            match read_csv_files(&paths) {
                Ok(events) => {
                    println!("Parsed events: {:?}", events);
                    self.events = events;
                    self.csv_loaded = true; // Set the flag to indicate files have been loaded
                    self.graph = correlate_events(&self.events); // Update the graph
                }
                Err(e) => {
                    println!("Failed to read CSV files: {:?}", e);
                    self.csv_loaded = false; // Ensure we retry if the user attempts to load again
                }
            }
        }
    }
}

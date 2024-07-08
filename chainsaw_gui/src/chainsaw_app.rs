use crate::utils::{read_csv_files, correlate_events, ChainsawEvent};
use std::path::PathBuf;
use eframe::egui::{CentralPanel, TopBottomPanel, ScrollArea, Context}; // Updated from CtxRef
use eframe::{App, NativeOptions, Frame}; // Directly from eframe

pub struct ChainsawApp {
    csv_file_paths: Vec<PathBuf>,
    events: Vec<ChainsawEvent>,
} 

impl Default for ChainsawApp {
    fn default() -> Self {
        Self {
            csv_file_paths: vec![],
            events: vec![],
        }
    }
}

impl ChainsawApp {
    pub fn load_csv_files(&mut self) {
        match nfd::open_file_dialog(None, None) {
            Ok(nfd::Response::Okay(file_path)) => {
                let path_str = file_path.clone();
                let path_buf = PathBuf::from(path_str);
                self.csv_file_paths.push(path_buf);
            }
            Ok(nfd::Response::OkayMultiple(file_paths)) => {
                for file_path in file_paths {
                    let path_str = file_path.clone();
                    let path_buf = PathBuf::from(path_str);
                    self.csv_file_paths.push(path_buf);
                }
            }
            _ => {
                println!("File dialog cancelled or encountered an error.");
            }
        }
    }
    pub fn update(&mut self, ctx: &Context, _frame: &Frame) { // Updated CtxRef to Context
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.label("Chainsaw GUI");
        });

        CentralPanel::default().show(ctx, |ui| {
            if ui.button("Load CSV files").clicked() {
                self.load_csv_files();
            }

            if !self.csv_file_paths.is_empty() {
                if let Ok(events) = read_csv_files(&self.csv_file_paths.iter().map(|p| p.to_string_lossy().to_string()).collect::<Vec<_>>()) {
                    self.events = events;
                }
            }

            if !self.events.is_empty() {
                let graph = correlate_events(&self.events);
                ScrollArea::both().show(ui, |ui| {
                    ui.label(format!("{:?}", graph));
                });
            }
        });
    }

    pub fn run(&mut self) {
        let native_options = NativeOptions::default();
        eframe::run_native(
            "ChainsawApp",
            native_options,
            Box::new(|cc| Ok(Box::new(ChainsawApp::default())))
        ).expect("Failed to run native");
    }
}

impl App for ChainsawApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.update(ctx, frame);
    }
}

use csv::ReaderBuilder;
use chrono::{DateTime, Utc, FixedOffset, Duration};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use petgraph::graph::{DiGraph, NodeIndex};
use std::io;
use std::fs::File;
use std::io::BufReader;


#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct ChainsawEvent {
    pub timestamp: Option<String>,
    pub detections: Option<String>,
    pub path: Option<String>,
    pub event_id: Option<u32>,
    pub record_id: Option<u32>,
    pub computer: Option<String>,
    pub user: Option<String>,
    pub user_sid: Option<String>,
    pub member_sid: Option<String>,
}

pub fn read_csv_files(file_paths: &[PathBuf]) -> io::Result<Vec<ChainsawEvent>> {
    let mut events = Vec::new();

    for path in file_paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new().from_reader(reader);

        for result in csv_reader.records() {
            let record = result?;
            let event = ChainsawEvent {
                timestamp: record.get(0).map(|s| s.to_string()),
                detections: record.get(1).map(|s| s.to_string()),
                path: record.get(2).map(|s| s.to_string()),
                event_id: record.get(3).and_then(|s| s.parse().ok()),
                record_id: record.get(4).and_then(|s| s.parse().ok()),
                computer: record.get(5).map(|s| s.to_string()),
                user: record.get(6).map(|s| s.to_string()),
                user_sid: record.get(7).map(|s| s.to_string()),
                member_sid: record.get(8).map(|s| s.to_string()),
            };
            events.push(event);
        }
    }

    Ok(events)
}


pub fn correlate_events(events: &[ChainsawEvent], delta: Duration) -> DiGraph<ChainsawEvent, ()> {
    let mut graph = DiGraph::new();
    let mut event_to_node: HashMap<usize, NodeIndex> = HashMap::new();

    for (i, event) in events.iter().enumerate() {
        let node_index = graph.add_node(event.clone());
        event_to_node.insert(i, node_index);
    }

    for i in 0..events.len() {
        let event_i = &events[i];

        for j in (i + 1)..events.len() {
            let event_j = &events[j];

            // Example correlation logic: correlate events if they have the same user_sid
            if event_i.user_sid == event_j.user_sid {
                let node_i = event_to_node[&i];
                let node_j = event_to_node[&j];

                // Add edge if it doesn't exist already
                if !graph.contains_edge(node_i, node_j) {
                    graph.add_edge(node_i, node_j, ());
                }
            }

            // Add more correlation logic based on other fields as needed
            // Example: correlate events if they have the same path
            // if event_i.path == event_j.path { ... }
        }
    }

    graph
}

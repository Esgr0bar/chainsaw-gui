// utils.rs

use csv;
use serde::{Serialize, Deserialize};
use std::error::Error;
use csv::ReaderBuilder;
use std::collections::HashMap;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashSet;

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

/// Reads CSV files from given paths and deserializes them into ChainsawEvent structs.
pub fn read_csv_files(paths: &[String]) -> Result<Vec<ChainsawEvent>, Box<dyn Error>> {
    let mut events = Vec::new();

    for path in paths {
        let mut rdr = ReaderBuilder::new().from_path(path)?;
        for result in rdr.deserialize() {
            let record: ChainsawEvent = result?;
            events.push(record);
        }
    }

    Ok(events)
}

pub fn correlate_events(events: &[ChainsawEvent]) -> DiGraph<NodeIndex, ()> {
    let mut graph = DiGraph::new();

    // HashMap to store nodes (indices) corresponding to each event index
    let mut event_to_node: HashMap<usize, NodeIndex> = HashMap::new();

    // Create nodes in the graph for each event
    for (index, event) in events.iter().enumerate() {
        let node_index = graph.add_node(NodeIndex::new(index));
        event_to_node.insert(index, node_index);
        println!("Added node {} for event {:?}", index, event); // Debug print
    }

    // Connect all nodes in the graph
    for i in 0..events.len() {
        for j in 0..events.len() {
            if i != j {
                if let Some(node1) = event_to_node.get(&i) {
                    if let Some(node2) = event_to_node.get(&j) {
                        graph.add_edge(*node1, *node2, ());
                        println!("Added edge between node {} and node {}", i, j); // Debug print
                    }
                }
            }
        }
    }

    println!("Graph nodes count: {}", graph.node_count()); // Debug print
    println!("Graph edges count: {}", graph.edge_count()); // Debug print

    graph
}

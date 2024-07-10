use csv::ReaderBuilder;
use chrono::{DateTime, Utc, FixedOffset};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use petgraph::graph::{DiGraph, NodeIndex};

/// Parses an RFC3339 timestamp string into DateTime<Utc>.
pub fn parse_timestamp(ts: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    // Parse the timestamp into DateTime<FixedOffset>
    let parsed_datetime: DateTime<FixedOffset> = DateTime::parse_from_rfc3339(ts)?;

    // Convert to DateTime<Utc>
    let datetime_utc: DateTime<Utc> = parsed_datetime.into();

    Ok(datetime_utc)
}

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
pub fn read_csv_files(paths: &[PathBuf]) -> Result<Vec<ChainsawEvent>, Box<dyn Error>> {
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

/// Correlates events into a directed graph (DiGraph).
pub fn correlate_events(events: &[ChainsawEvent]) -> DiGraph<(), ()> {
    let mut graph = DiGraph::new();
    let mut event_to_node: HashMap<usize, NodeIndex> = HashMap::new();

    // Add nodes to the graph
    for (i, event) in events.iter().enumerate() {
        let node_index = graph.add_node(());
        event_to_node.insert(i, node_index);
        println!("Added node {} for event {:?}", node_index.index(), event);
    }

    // Add edges between nodes based on a simplified event correlation logic
    for i in 0..events.len() {
        if let Some(&source_node) = event_to_node.get(&i) {
            // For demonstration, let's add an edge to the next event in the list (a linear chain)
            if i + 1 < events.len() {
                if let Some(&target_node) = event_to_node.get(&(i + 1)) {
                    // Check if the edge already exists before adding
                    if !graph.contains_edge(source_node, target_node) {
                        graph.add_edge(source_node, target_node, ());
                        println!("Added edge between node {} and node {}", source_node.index(), target_node.index());
                    }
                }
            }
        }
    }

    println!("Graph nodes count: {}", graph.node_count());
    println!("Graph edges count: {}", graph.edge_count());
    graph
}

use csv;
use petgraph::graph::DiGraph;
use serde::{Serialize, Deserialize};
use std::error::Error;
use csv::ReaderBuilder;
use std::collections::HashMap;
use petgraph::graph::{ NodeIndex};
use petgraph::dot::{Dot, Config};

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
/// Correlates ChainsawEvents based on common attributes.
pub fn correlate_events(events: &[ChainsawEvent]) -> DiGraph<(), ()> {
    let mut graph = DiGraph::new();

    // Create a mapping from event_id to node index in the graph
    let mut event_id_to_node: HashMap<Option<u32>, NodeIndex> = HashMap::new();
    for (index, event) in events.iter().enumerate() {
        let node_index = graph.add_node(());
        event_id_to_node.insert(event.event_id, node_index);
    }

    // Create edges based on timestamp and event_id similarity
    for i in 0..events.len() {
        for j in (i + 1)..events.len() {
            let event1 = &events[i];
            let event2 = &events[j];

            // Example condition: correlate based on timestamp and event_id
            if event1.timestamp == event2.timestamp || event1.event_id == event2.event_id {
                if let Some(node1) = event_id_to_node.get(&event1.event_id) {
                    if let Some(node2) = event_id_to_node.get(&event2.event_id) {
                        graph.add_edge(*node1, *node2, ());
                    }
                }
            }
        }
    }

    graph
}
fn visualize_graph(graph: &DiGraph<(), ()>, events: &[ChainsawEvent]) {
    let dot = Dot::with_config(graph, &[Config::EdgeNoLabel]);

    // Output DOT format
    println!("{:?}", dot);

    // You can use a DOT renderer to visualize the graph in an external tool
    // For demonstration, we'll print out the DOT format here
    println!("Generated DOT format for visualization:");

    // Print out events for demonstration
    for (index, event) in events.iter().enumerate() {
    println!("Event {}: {:?}", index + 1, event);
}
}

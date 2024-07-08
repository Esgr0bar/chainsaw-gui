use csv;
use petgraph::graph::DiGraph;
use petgraph::algo::tarjan_scc;
use serde::{Serialize, Deserialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct ChainsawEvent {
    pub timestamp: String,
    pub detections: String,
    pub path: String,
    pub event_id: u32,
    pub record_id: u32,
    pub computer: String,
    pub user: Option<String>,
    pub user_sid: Option<String>,
    pub member_sid: Option<String>,
}

/// Reads CSV files from given paths and deserializes them into ChainsawEvent structs.
pub fn read_csv_files(paths: &[String]) -> Result<Vec<ChainsawEvent>, Box<dyn Error>> {
    let mut events = Vec::new();
    for path in paths {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        for result in rdr.deserialize() {
            let event: ChainsawEvent = result?;
            events.push(event);
        }
    }
    Ok(events)
}

/// Correlates ChainsawEvents based on common attributes.
pub fn correlate_events(events: &[ChainsawEvent]) -> DiGraph<(), ()> {
    let mut graph = DiGraph::new();

    // Create a mapping from event_id to node index in the graph
    let mut event_id_to_node = std::collections::HashMap::new();
    for event in events {
        let node_index = graph.add_node(());
        event_id_to_node.insert(event.event_id, node_index);
    }

    // Create edges based on event_id and path similarity
    for i in 0..events.len() {
        for j in (i + 1)..events.len() {
            let event1 = &events[i];
            let event2 = &events[j];

            if event1.event_id == event2.event_id || event1.path == event2.path {
                if let Some(node1) = event_id_to_node.get(&event1.event_id) {
                    if let Some(node2) = event_id_to_node.get(&event2.event_id) {
                        graph.add_edge(*node1, *node2, ());
                    }
                }
            }
        }
    }

    // Optional: Perform Strongly Connected Components (SCC) analysis
    let sccs = tarjan_scc(&graph);

    // Print SCCs for demonstration (you can replace this with your further logic)
    for scc in &sccs {
        println!("SCC:");
        for &node in scc {
            println!("  {:?}", events[node.index()]);
        }
    }

    graph
}

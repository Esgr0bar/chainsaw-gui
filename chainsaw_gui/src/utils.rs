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

pub fn parse_timestamp(ts: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    let parsed_datetime: DateTime<FixedOffset> = DateTime::parse_from_rfc3339(ts)?;
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

pub fn correlate_events(events: &[ChainsawEvent], delta: Duration) -> DiGraph<String, ()> {
    let mut graph = DiGraph::new();
    let mut event_to_node: HashMap<usize, NodeIndex> = HashMap::new();
    let mut correlation_map: HashMap<String, Vec<usize>> = HashMap::new();

    for (i, event) in events.iter().enumerate() {
        let label = format!("{:?}", event); // Use the event details as the label
        let node_index = graph.add_node(label);
        event_to_node.insert(i, node_index);

        if let Some(timestamp) = &event.timestamp {
            if let Ok(ts) = parse_timestamp(timestamp) {
                let time_key = ts.timestamp().to_string();
                correlation_map.entry(time_key).or_default().push(i);

                for j in (1..=delta.num_seconds()).chain(1..=delta.num_seconds()) {
                    let adjusted_time_key = (ts + chrono::Duration::seconds(j)).timestamp().to_string();
                    correlation_map.entry(adjusted_time_key).or_default().push(i);
                }
            }
        }

        if let Some(sid) = &event.user_sid {
            correlation_map.entry(sid.clone()).or_default().push(i);
        }
        if let Some(path) = &event.path {
            correlation_map.entry(path.clone()).or_default().push(i);
        }
    }

    for correlated_indices in correlation_map.values() {
        for &source in correlated_indices {
            for &target in correlated_indices {
                if source != target {
                    let source_node = event_to_node[&source];
                    let target_node = event_to_node[&target];
                    if !graph.contains_edge(source_node, target_node) {
                        graph.add_edge(source_node, target_node, ());
                    }
                }
            }
        }
    }

    graph
}

// Timeline analysis utilities
use crate::core::FileEntry;
use std::time::SystemTime;

pub struct TimelineAnalyzer {
    events: Vec<TimelineEvent>,
}

#[derive(Clone)]
pub struct TimelineEvent {
    pub timestamp: SystemTime,
    pub event_type: EventType,
    pub path: std::path::PathBuf,
    pub description: String,
}

#[derive(Clone)]
pub enum EventType {
    Created,
    Modified,
    Accessed,
    Deleted,
}

impl TimelineAnalyzer {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn add_entry(&mut self, entry: &FileEntry) {
        if let Some(created) = entry.created {
            self.events.push(TimelineEvent {
                timestamp: created,
                event_type: EventType::Created,
                path: entry.path.clone(),
                description: format!("File created: {}", entry.path.display()),
            });
        }

        if let Some(modified) = entry.modified {
            self.events.push(TimelineEvent {
                timestamp: modified,
                event_type: EventType::Modified,
                path: entry.path.clone(),
                description: format!("File modified: {}", entry.path.display()),
            });
        }
    }

    pub fn get_events(&self) -> &[TimelineEvent] {
        &self.events
    }

    pub fn get_events_in_range(&self, start: SystemTime, end: SystemTime) -> Vec<&TimelineEvent> {
        self.events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect()
    }
}



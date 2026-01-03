use crate::core::FileEntry;
use crate::forensic::hashing::{calculate_file_md5, calculate_file_sha256};
use crate::forensic::metadata::{extract_exif, extract_office_metadata, extract_pdf_metadata};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub struct ForensicAnalyzer {
    // Analyzer state
}

impl ForensicAnalyzer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn analyze_file(&self, entry: &FileEntry) -> ForensicReport {
        let md5_hash = calculate_file_md5(&entry.path).unwrap_or_else(|_| "ERROR".to_string());
        let sha256_hash = calculate_file_sha256(&entry.path).unwrap_or_else(|_| "ERROR".to_string());
        
        let mut metadata_entries = vec![];
        
        // Try to extract metadata based on file type
        let file_type = self.detect_file_type(&entry.path);
        
        match file_type.as_str() {
            "jpg" | "jpeg" | "png" | "tiff" => {
                if let Ok(meta) = extract_exif(&entry.path) {
                    for (key, value) in meta {
                        metadata_entries.push(MetadataEntry { key, value });
                    }
                }
            }
            "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" => {
                if let Ok(meta) = extract_office_metadata(&entry.path) {
                    for (key, value) in meta {
                        metadata_entries.push(MetadataEntry { key, value });
                    }
                }
            }
            "pdf" => {
                if let Ok(meta) = extract_pdf_metadata(&entry.path) {
                    for (key, value) in meta {
                        metadata_entries.push(MetadataEntry { key, value });
                    }
                }
            }
            _ => {}
        }

        // Add basic file metadata
        if let Some(created) = entry.created {
            if let Ok(time) = created.duration_since(std::time::UNIX_EPOCH) {
                metadata_entries.push(MetadataEntry {
                    key: "Created".to_string(),
                    value: format!("{:?}", time),
                });
            }
        }
        if let Some(modified) = entry.modified {
            if let Ok(time) = modified.duration_since(std::time::UNIX_EPOCH) {
                metadata_entries.push(MetadataEntry {
                    key: "Modified".to_string(),
                    value: format!("{:?}", time),
                });
            }
        }
        metadata_entries.push(MetadataEntry {
            key: "Size".to_string(),
            value: format!("{} bytes", entry.size),
        });

        ForensicReport {
            path: entry.path.clone(),
            md5_hash,
            sha256_hash,
            file_type,
            metadata: metadata_entries,
        }
    }

    pub fn calculate_md5(&self, path: &Path) -> String {
        calculate_file_md5(path).unwrap_or_else(|_| "ERROR".to_string())
    }

    pub fn calculate_sha256(&self, path: &Path) -> String {
        calculate_file_sha256(path).unwrap_or_else(|_| "ERROR".to_string())
    }

    pub fn detect_file_type(&self, path: &Path) -> String {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| "unknown".to_string())
    }

    pub fn extract_metadata(&self, entry: &FileEntry) -> Vec<MetadataEntry> {
        // Extract extended metadata
        let report = self.analyze_file(entry);
        report.metadata
    }

    pub fn build_timeline(&self, entries: &[FileEntry]) -> Timeline {
        Timeline::new(entries)
    }
}

pub struct ForensicReport {
    pub path: std::path::PathBuf,
    pub md5_hash: String,
    pub sha256_hash: String,
    pub file_type: String,
    pub metadata: Vec<MetadataEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataEntry {
    pub key: String,
    pub value: String,
}

pub struct Timeline {
    pub events: Vec<TimelineEvent>,
}

pub struct TimelineEvent {
    pub timestamp: std::time::SystemTime,
    pub event_type: String,
    pub path: std::path::PathBuf,
}

impl Timeline {
    pub fn new(entries: &[FileEntry]) -> Self {
        let mut events = Vec::new();
        
        for entry in entries {
            if let Some(created) = entry.created {
                events.push(TimelineEvent {
                    timestamp: created,
                    event_type: "CREATED".to_string(),
                    path: entry.path.clone(),
                });
            }
            if let Some(modified) = entry.modified {
                events.push(TimelineEvent {
                    timestamp: modified,
                    event_type: "MODIFIED".to_string(),
                    path: entry.path.clone(),
                });
            }
        }

        events.sort_by_key(|e| e.timestamp);
        
        Self { events }
    }
}


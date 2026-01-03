// Export forensic reports to JSON and CSV
use crate::forensic::analyzer::{ForensicReport, MetadataEntry};
use crate::core::FileEntry;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use anyhow::Result;
use csv::Writer;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportableReport {
    pub path: String,
    pub md5_hash: String,
    pub sha256_hash: String,
    pub file_type: String,
    pub size: u64,
    pub is_deleted: bool,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub metadata: Vec<MetadataEntry>,
}

impl From<&ForensicReport> for ExportableReport {
    fn from(report: &ForensicReport) -> Self {
        ExportableReport {
            path: report.path.to_string_lossy().to_string(),
            md5_hash: report.md5_hash.clone(),
            sha256_hash: report.sha256_hash.clone(),
            file_type: report.file_type.clone(),
            size: 0, // Will be filled from FileEntry
            is_deleted: false, // Will be filled from FileEntry
            created: None,
            modified: None,
            metadata: report.metadata.clone(),
        }
    }
}

pub struct ReportExporter;

impl ReportExporter {
    pub fn export_json<P: AsRef<Path>>(
        &self,
        reports: &[ForensicReport],
        output_path: P,
    ) -> Result<()> {
        let exportable: Vec<ExportableReport> = reports.iter().map(ExportableReport::from).collect();
        let json = serde_json::to_string_pretty(&exportable)?;
        
        let mut file = File::create(output_path)?;
        file.write_all(json.as_bytes())?;
        
        Ok(())
    }
    
    pub fn export_csv<P: AsRef<Path>>(
        &self,
        reports: &[ForensicReport],
        output_path: P,
    ) -> Result<()> {
        let mut wtr = Writer::from_path(output_path)?;
        
        // Write header
        wtr.write_record(&[
            "Path",
            "MD5",
            "SHA256",
            "File Type",
            "Size",
            "Is Deleted",
            "Created",
            "Modified",
            "Metadata Count",
        ])?;
        
        // Write data
        for report in reports {
            let metadata_count = report.metadata.len();
            wtr.write_record(&[
                report.path.to_string_lossy().to_string(),
                report.md5_hash.clone(),
                report.sha256_hash.clone(),
                report.file_type.clone(),
                "0".to_string(), // Size would need FileEntry
                "false".to_string(), // Is deleted would need FileEntry
                String::new(), // Created
                String::new(), // Modified
                metadata_count.to_string(),
            ])?;
        }
        
        wtr.flush()?;
        Ok(())
    }
    
    pub fn export_files_csv<P: AsRef<Path>>(
        &self,
        entries: &[FileEntry],
        output_path: P,
    ) -> Result<()> {
        let mut wtr = Writer::from_path(output_path)?;
        
        // Write header
        wtr.write_record(&[
            "Path",
            "Size",
            "Is Deleted",
            "Created",
            "Modified",
        ])?;
        
        // Write data
        for entry in entries {
            let created = entry.created
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default();
            
            let modified = entry.modified
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default();
            
            wtr.write_record(&[
                entry.path.to_string_lossy().to_string(),
                entry.size.to_string(),
                entry.is_deleted.to_string(),
                created,
                modified,
            ])?;
        }
        
        wtr.flush()?;
        Ok(())
    }
    
    pub fn export_timeline_csv<P: AsRef<Path>>(
        &self,
        timeline: &crate::forensic::analyzer::Timeline,
        output_path: P,
    ) -> Result<()> {
        let mut wtr = Writer::from_path(output_path)?;
        
        // Write header
        wtr.write_record(&[
            "Timestamp",
            "Event Type",
            "Path",
        ])?;
        
        // Write data
        for event in &timeline.events {
            let timestamp = event.timestamp
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default();
            
            wtr.write_record(&[
                timestamp,
                event.event_type.clone(),
                event.path.to_string_lossy().to_string(),
            ])?;
        }
        
        wtr.flush()?;
        Ok(())
    }
}


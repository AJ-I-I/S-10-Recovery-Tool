// File carving capabilities for recovering files from raw data
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct CarvedFile {
    pub offset: u64,
    pub size: u64,
    pub file_type: String,
    pub signature: Vec<u8>,
}

pub struct FileCarver {
    signatures: Vec<FileSignature>,
}

#[derive(Debug, Clone)]
struct FileSignature {
    name: String,
    header: Vec<u8>,
    footer: Option<Vec<u8>>,
    min_size: u64,
    max_size: Option<u64>,
}

impl FileCarver {
    pub fn new() -> Self {
        let mut signatures = Vec::new();
        
        // PDF
        signatures.push(FileSignature {
            name: "PDF".to_string(),
            header: b"%PDF-".to_vec(),
            footer: Some(b"%%EOF".to_vec()),
            min_size: 100,
            max_size: Some(100 * 1024 * 1024), // 100MB max
        });
        
        // JPEG
        signatures.push(FileSignature {
            name: "JPEG".to_string(),
            header: vec![0xFF, 0xD8, 0xFF],
            footer: Some(vec![0xFF, 0xD9]),
            min_size: 100,
            max_size: Some(50 * 1024 * 1024), // 50MB max
        });
        
        // PNG
        signatures.push(FileSignature {
            name: "PNG".to_string(),
            header: vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
            footer: None,
            min_size: 100,
            max_size: Some(50 * 1024 * 1024),
        });
        
        // ZIP
        signatures.push(FileSignature {
            name: "ZIP".to_string(),
            header: vec![0x50, 0x4B, 0x03, 0x04],
            footer: Some(vec![0x50, 0x4B, 0x05, 0x06]), // EOCD
            min_size: 100,
            max_size: Some(2 * 1024 * 1024 * 1024), // 2GB max
        });
        
        // MP4
        signatures.push(FileSignature {
            name: "MP4".to_string(),
            header: vec![0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70], // ftyp box
            footer: None,
            min_size: 1000,
            max_size: Some(5 * 1024 * 1024 * 1024), // 5GB max
        });
        
        // DOCX (ZIP-based)
        signatures.push(FileSignature {
            name: "DOCX".to_string(),
            header: vec![0x50, 0x4B, 0x03, 0x04],
            footer: Some(vec![0x50, 0x4B, 0x05, 0x06]),
            min_size: 1000,
            max_size: Some(100 * 1024 * 1024),
        });
        
        Self { signatures }
    }
    
    pub fn carve_file<P: AsRef<Path>>(
        &self,
        source: P,
        output_dir: P,
        offset: u64,
        size: u64,
    ) -> Result<CarvedFile> {
        let mut source_file = File::open(&source)?;
        source_file.seek(SeekFrom::Start(offset))?;
        
        let mut buffer = vec![0u8; size.min(1024 * 1024) as usize]; // Read up to 1MB for signature
        source_file.read_exact(&mut buffer)?;
        
        // Detect file type
        let file_type = self.detect_file_type(&buffer)?;
        
        // Create output file
        let output_path = output_dir.as_ref().join(format!("carved_{}_{}.{}", offset, size, file_type.to_lowercase()));
        let mut output_file = File::create(&output_path)?;
        
        // Write header
        output_file.write_all(&buffer)?;
        
        // If more data, read and write the rest
        if size > buffer.len() as u64 {
            source_file.seek(SeekFrom::Start(offset + buffer.len() as u64))?;
            let remaining = size - buffer.len() as u64;
            let mut remaining_buffer = vec![0u8; remaining.min(10 * 1024 * 1024) as usize];
            let bytes_read = source_file.read(&mut remaining_buffer)?;
            output_file.write_all(&remaining_buffer[..bytes_read])?;
        }
        
        Ok(CarvedFile {
            offset,
            size,
            file_type,
            signature: buffer[..size.min(16) as usize].to_vec(),
        })
    }
    
    pub fn scan_for_files<P: AsRef<Path>>(
        &self,
        source: P,
        output_dir: P,
    ) -> Result<Vec<CarvedFile>> {
        let mut source_file = File::open(&source)?;
        let file_size = source_file.metadata()?.len();
        let mut carved_files = Vec::new();
        
        let mut buffer = vec![0u8; 64 * 1024]; // 64KB buffer
        let mut offset = 0u64;
        
        while offset < file_size {
            source_file.seek(SeekFrom::Start(offset))?;
            let bytes_read = source_file.read(&mut buffer)?;
            
            if bytes_read == 0 {
                break;
            }
            
            // Search for file signatures
            for sig in &self.signatures {
                if let Some(pos) = self.find_signature(&buffer[..bytes_read], &sig.header) {
                    let file_offset = offset + pos as u64;
                    
                    // Try to find footer if available
                    let file_size = if let Some(ref footer) = sig.footer {
                        self.find_file_size(&mut source_file, file_offset, &sig.header, footer, sig.min_size, sig.max_size)?
                    } else {
                        // Estimate size or use max
                        sig.max_size.unwrap_or(10 * 1024 * 1024)
                    };
                    
                    // Carve the file
                    if let Ok(carved) = self.carve_file(&source, &output_dir, file_offset, file_size) {
                        carved_files.push(carved);
                        offset = file_offset + file_size; // Skip past this file
                        break;
                    }
                }
            }
            
            offset += bytes_read as u64 - 1024; // Overlap by 1KB to catch files at boundaries
        }
        
        Ok(carved_files)
    }
    
    fn detect_file_type(&self, buffer: &[u8]) -> Result<String> {
        for sig in &self.signatures {
            if buffer.len() >= sig.header.len() && buffer.starts_with(&sig.header) {
                return Ok(sig.name.clone());
            }
        }
        Ok("UNKNOWN".to_string())
    }
    
    fn find_signature(&self, buffer: &[u8], signature: &[u8]) -> Option<usize> {
        buffer.windows(signature.len()).position(|window| window == signature)
    }
    
    fn find_file_size(
        &self,
        file: &mut File,
        start_offset: u64,
        _header: &[u8],
        footer: &[u8],
        min_size: u64,
        max_size: Option<u64>,
    ) -> Result<u64> {
        let max_search = max_size.unwrap_or(100 * 1024 * 1024); // Default 100MB
        let mut buffer = vec![0u8; 64 * 1024];
        let mut offset = start_offset + min_size;
        let end_offset = start_offset + max_search;
        
        while offset < end_offset {
            file.seek(SeekFrom::Start(offset))?;
            let bytes_read = file.read(&mut buffer)?;
            
            if bytes_read == 0 {
                break;
            }
            
            if let Some(pos) = buffer[..bytes_read].windows(footer.len()).position(|w| w == footer) {
                return Ok(offset + pos as u64 + footer.len() as u64 - start_offset);
            }
            
            offset += bytes_read as u64 - footer.len() as u64;
        }
        
        // If footer not found, return estimated size
        Ok(max_size.unwrap_or(min_size * 10))
    }
}

impl Default for FileCarver {
    fn default() -> Self {
        Self::new()
    }
}


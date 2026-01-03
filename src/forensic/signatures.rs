// Advanced file signature detection using magic numbers
use std::fs::File;
use std::io::Read;
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct FileSignature {
    pub name: String,
    pub extension: String,
    pub mime_type: String,
    pub header: Vec<u8>,
    pub footer: Option<Vec<u8>>,
    pub offset: usize, // Offset from start where signature begins
}

pub struct SignatureDetector {
    signatures: Vec<FileSignature>,
}

impl SignatureDetector {
    pub fn new() -> Self {
        let mut signatures = Vec::new();
        
        // Images
        signatures.push(FileSignature {
            name: "JPEG".to_string(),
            extension: "jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            header: vec![0xFF, 0xD8, 0xFF],
            footer: Some(vec![0xFF, 0xD9]),
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "PNG".to_string(),
            extension: "png".to_string(),
            mime_type: "image/png".to_string(),
            header: vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
            footer: None,
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "GIF".to_string(),
            extension: "gif".to_string(),
            mime_type: "image/gif".to_string(),
            header: b"GIF87a".to_vec(),
            footer: None,
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "GIF89a".to_string(),
            extension: "gif".to_string(),
            mime_type: "image/gif".to_string(),
            header: b"GIF89a".to_vec(),
            footer: None,
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "BMP".to_string(),
            extension: "bmp".to_string(),
            mime_type: "image/bmp".to_string(),
            header: b"BM".to_vec(),
            footer: None,
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "TIFF".to_string(),
            extension: "tiff".to_string(),
            mime_type: "image/tiff".to_string(),
            header: vec![0x49, 0x49, 0x2A, 0x00], // Little-endian
            footer: None,
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "TIFF".to_string(),
            extension: "tiff".to_string(),
            mime_type: "image/tiff".to_string(),
            header: vec![0x4D, 0x4D, 0x00, 0x2A], // Big-endian
            footer: None,
            offset: 0,
        });
        
        // Documents
        signatures.push(FileSignature {
            name: "PDF".to_string(),
            extension: "pdf".to_string(),
            mime_type: "application/pdf".to_string(),
            header: b"%PDF-".to_vec(),
            footer: Some(b"%%EOF".to_vec()),
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "ZIP".to_string(),
            extension: "zip".to_string(),
            mime_type: "application/zip".to_string(),
            header: vec![0x50, 0x4B, 0x03, 0x04],
            footer: Some(vec![0x50, 0x4B, 0x05, 0x06]),
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "DOCX".to_string(),
            extension: "docx".to_string(),
            mime_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
            header: vec![0x50, 0x4B, 0x03, 0x04],
            footer: None,
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "XLSX".to_string(),
            extension: "xlsx".to_string(),
            mime_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
            header: vec![0x50, 0x4B, 0x03, 0x04],
            footer: None,
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "PPTX".to_string(),
            extension: "pptx".to_string(),
            mime_type: "application/vnd.openxmlformats-officedocument.presentationml.presentation".to_string(),
            header: vec![0x50, 0x4B, 0x03, 0x04],
            footer: None,
            offset: 0,
        });
        
        // Media
        signatures.push(FileSignature {
            name: "MP4".to_string(),
            extension: "mp4".to_string(),
            mime_type: "video/mp4".to_string(),
            header: vec![0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70],
            footer: None,
            offset: 4,
        });
        
        signatures.push(FileSignature {
            name: "MP3".to_string(),
            extension: "mp3".to_string(),
            mime_type: "audio/mpeg".to_string(),
            header: vec![0xFF, 0xFB],
            footer: None,
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "MP3 ID3v2".to_string(),
            extension: "mp3".to_string(),
            mime_type: "audio/mpeg".to_string(),
            header: b"ID3".to_vec(),
            footer: None,
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "AVI".to_string(),
            extension: "avi".to_string(),
            mime_type: "video/x-msvideo".to_string(),
            header: b"RIFF".to_vec(),
            footer: None,
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "WAV".to_string(),
            extension: "wav".to_string(),
            mime_type: "audio/wav".to_string(),
            header: b"RIFF".to_vec(),
            footer: None,
            offset: 0,
        });
        
        // Archives
        signatures.push(FileSignature {
            name: "RAR".to_string(),
            extension: "rar".to_string(),
            mime_type: "application/x-rar-compressed".to_string(),
            header: b"Rar!".to_vec(),
            footer: None,
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "7Z".to_string(),
            extension: "7z".to_string(),
            mime_type: "application/x-7z-compressed".to_string(),
            header: b"7z\xBC\xAF\x27\x1C".to_vec(),
            footer: None,
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "GZIP".to_string(),
            extension: "gz".to_string(),
            mime_type: "application/gzip".to_string(),
            header: vec![0x1F, 0x8B],
            footer: None,
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "TAR".to_string(),
            extension: "tar".to_string(),
            mime_type: "application/x-tar".to_string(),
            header: vec![0x75, 0x73, 0x74, 0x61, 0x72], // ustar
            footer: None,
            offset: 257,
        });
        
        // Executables
        signatures.push(FileSignature {
            name: "PE".to_string(),
            extension: "exe".to_string(),
            mime_type: "application/x-msdownload".to_string(),
            header: vec![0x4D, 0x5A], // MZ
            footer: None,
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "ELF".to_string(),
            extension: "elf".to_string(),
            mime_type: "application/x-executable".to_string(),
            header: b"\x7FELF".to_vec(),
            footer: None,
            offset: 0,
        });
        
        signatures.push(FileSignature {
            name: "Mach-O".to_string(),
            extension: "o".to_string(),
            mime_type: "application/x-mach-binary".to_string(),
            header: vec![0xFE, 0xED, 0xFA, 0xCE],
            footer: None,
            offset: 0,
        });
        
        Self { signatures }
    }
    
    pub fn detect_file_type<P: AsRef<Path>>(&self, path: P) -> Result<Option<FileSignature>> {
        let mut file = File::open(path)?;
        let mut buffer = vec![0u8; 512]; // Read first 512 bytes
        let bytes_read = file.read(&mut buffer)?;
        
        if bytes_read == 0 {
            return Ok(None);
        }
        
        for sig in &self.signatures {
            let start = sig.offset;
            let end = start + sig.header.len();
            
            if end <= buffer.len() && buffer[start..end] == sig.header {
                return Ok(Some(sig.clone()));
            }
        }
        
        Ok(None)
    }
    
    pub fn detect_from_bytes(&self, data: &[u8]) -> Option<FileSignature> {
        for sig in &self.signatures {
            let start = sig.offset;
            let end = start + sig.header.len();
            
            if end <= data.len() && data[start..end] == sig.header {
                return Some(sig.clone());
            }
        }
        None
    }
    
    pub fn get_all_signatures(&self) -> &[FileSignature] {
        &self.signatures
    }
}

impl Default for SignatureDetector {
    fn default() -> Self {
        Self::new()
    }
}


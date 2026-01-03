use sha2::Sha256;
use sha2::digest::Digest;
use std::fs;
use std::path::Path;

pub fn calculate_file_md5(path: &Path) -> Result<String, std::io::Error> {
    let data = fs::read(path)?;
    let hash = md5::compute(&data);
    Ok(format!("{:x}", hash))
}

pub fn calculate_file_sha256(path: &Path) -> Result<String, std::io::Error> {
    use std::fs::File;
    use std::io::Read;
    
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}


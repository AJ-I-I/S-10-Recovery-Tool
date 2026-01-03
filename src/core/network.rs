// Network drive support
use std::path::{Path, PathBuf};

#[cfg(windows)]
pub fn is_network_path(path: &Path) -> bool {
    path.to_string_lossy().starts_with("\\\\") || 
    path.to_string_lossy().starts_with("//")
}

#[cfg(not(windows))]
pub fn is_network_path(path: &Path) -> bool {
    path.to_string_lossy().starts_with("//") ||
    path.to_string_lossy().starts_with("smb://") ||
    path.to_string_lossy().starts_with("nfs://")
}

pub fn normalize_network_path(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    
    #[cfg(windows)]
    {
        // Windows UNC paths: \\server\share
        if path_str.starts_with("\\\\") {
            return PathBuf::from(path_str.as_ref());
        }
    }
    
    #[cfg(not(windows))]
    {
        // Unix-style network paths
        if path_str.starts_with("//") {
            return PathBuf::from(path_str.as_ref());
        }
        if path_str.starts_with("smb://") {
            let normalized = path_str.replace("smb://", "//");
            return PathBuf::from(normalized);
        }
        if path_str.starts_with("nfs://") {
            let normalized = path_str.replace("nfs://", "//");
            return PathBuf::from(normalized);
        }
    }
    
    PathBuf::from(path)
}

pub fn check_network_access(path: &Path) -> bool {
    if !is_network_path(path) {
        return true; // Not a network path, assume accessible
    }
    
    // Try to read directory to check access
    std::fs::read_dir(path).is_ok()
}

pub fn get_network_drives() -> Vec<PathBuf> {
    let mut drives = Vec::new();
    
    #[cfg(windows)]
    {
        use std::process::Command;
        // Try to list network drives using net use command
        if let Ok(output) = Command::new("net")
            .args(&["use"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains("\\\\") {
                    if let Some(start) = line.find("\\\\") {
                        if let Some(end) = line[start..].find(' ') {
                            let drive = &line[start..start + end];
                            drives.push(PathBuf::from(drive));
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(not(windows))]
    {
        // Check common mount points
        let mount_points = vec!["/mnt", "/media", "/net"];
        for mount in mount_points {
            if let Ok(entries) = std::fs::read_dir(mount) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        drives.push(path);
                    }
                }
            }
        }
    }
    
    drives
}


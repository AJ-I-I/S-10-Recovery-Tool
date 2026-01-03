// Bookmark management for favorite directories
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use dirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub name: String,
    pub path: PathBuf,
    pub created: chrono::DateTime<chrono::Utc>,
}

pub struct BookmarkManager {
    bookmarks: HashMap<String, Bookmark>,
    config_path: PathBuf,
}

impl BookmarkManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
            .unwrap_or_else(|| PathBuf::from("."));
        
        let config_path = config_dir.join("s10-recovery-tool").join("bookmarks.json");
        
        let mut manager = Self {
            bookmarks: HashMap::new(),
            config_path,
        };
        
        manager.load()?;
        Ok(manager)
    }
    
    pub fn add(&mut self, name: String, path: PathBuf) -> Result<()> {
        let bookmark = Bookmark {
            name: name.clone(),
            path,
            created: chrono::Utc::now(),
        };
        
        self.bookmarks.insert(name, bookmark);
        self.save()?;
        Ok(())
    }
    
    pub fn remove(&mut self, name: &str) -> Result<()> {
        self.bookmarks.remove(name);
        self.save()?;
        Ok(())
    }
    
    pub fn get(&self, name: &str) -> Option<&Bookmark> {
        self.bookmarks.get(name)
    }
    
    pub fn get_path(&self, name: &str) -> Option<&PathBuf> {
        self.bookmarks.get(name).map(|b| &b.path)
    }
    
    pub fn list(&self) -> Vec<&Bookmark> {
        let mut bookmarks: Vec<&Bookmark> = self.bookmarks.values().collect();
        bookmarks.sort_by(|a, b| a.name.cmp(&b.name));
        bookmarks
    }
    
    pub fn exists(&self, name: &str) -> bool {
        self.bookmarks.contains_key(name)
    }
    
    fn load(&mut self) -> Result<()> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path)?;
            let bookmarks: HashMap<String, Bookmark> = serde_json::from_str(&content)?;
            self.bookmarks = bookmarks;
        }
        Ok(())
    }
    
    fn save(&self) -> Result<()> {
        // Create config directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(&self.bookmarks)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }
}

impl Default for BookmarkManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            bookmarks: HashMap::new(),
            config_path: PathBuf::from("bookmarks.json"),
        })
    }
}


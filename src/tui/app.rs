use crate::core::recovery::{RecoveryEngine, ScanStats};
use crate::core::FileEntry;
use crate::forensic::ForensicAnalyzer;
use crate::tui::command::{CommandHistory, TabCompleter};
use crate::tui::bookmarks::BookmarkManager;
use crate::forensic::{FileCarver, SignatureDetector, ReportExporter, EncryptionDetector, MemoryDumpAnalyzer};
use crate::core::network::{is_network_path, normalize_network_path, check_network_access};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq)]
pub enum AppMode {
    Browser,
    Search,
    Forensic,
    Command, // Command input mode for cd navigation
}

pub struct App {
    pub target: Option<PathBuf>,
    pub pattern: Option<String>,
    pub deep_scan: bool,
    pub mode: AppMode,
    pub should_quit: bool,
    pub is_scanning: bool,
    pub is_paused: bool,
    
    pub items: Vec<FileEntry>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    
    pub status_message: String,
    pub status_timestamp: Option<Instant>,
    
    pub scan_progress: f64,
    pub scan_stats: ScanStats,
    
    pub recovery_engine: Arc<Mutex<RecoveryEngine>>,
    pub forensic_analyzer: ForensicAnalyzer,
    
    // Shared state for background scanning
    pub items_shared: Arc<Mutex<Vec<FileEntry>>>,
    pub stats_shared: Arc<Mutex<ScanStats>>,
    
    // Navigation state
    pub current_dir: PathBuf,
    pub command_input: String,
    pub directory_items: Vec<PathBuf>, // Current directory listing
    
    // Command history and completion
    pub command_history: CommandHistory,
    pub tab_completer: TabCompleter,
    pub completion_suggestions: Vec<String>,
    pub completion_index: Option<usize>,
    
    // Bookmarks
    pub bookmarks: BookmarkManager,
    
    // Forensic tools
    pub file_carver: FileCarver,
    pub signature_detector: SignatureDetector,
    pub report_exporter: ReportExporter,
    pub encryption_detector: EncryptionDetector,
    pub memory_analyzer: MemoryDumpAnalyzer,
}

impl App {
    pub fn new(target: Option<PathBuf>, pattern: Option<String>, deep_scan: bool) -> Self {
        let recovery_engine = Arc::new(Mutex::new(RecoveryEngine::new()));
        let items_shared = Arc::new(Mutex::new(Vec::new()));
        let stats_shared = Arc::new(Mutex::new(ScanStats::default()));
        
        // Initialize current directory
        let current_dir = target.clone().unwrap_or_else(|| {
            #[cfg(windows)]
            let default = PathBuf::from("C:\\");
            #[cfg(not(windows))]
            let default = PathBuf::from("/");
            std::env::current_dir().unwrap_or(default)
        });
        
        let tab_completer = TabCompleter::new(current_dir.clone());
        let bookmarks = BookmarkManager::new().unwrap_or_else(|_| BookmarkManager::default());
        
        let mut app = Self {
            target,
            pattern,
            deep_scan,
            mode: AppMode::Browser,
            should_quit: false,
            is_scanning: false,
            is_paused: false,
            items: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            status_message: "Press 's' to start scan, 'q' to quit, ':' for command mode".to_string(),
            status_timestamp: None,
            scan_progress: 0.0,
            scan_stats: ScanStats::default(),
            recovery_engine,
            forensic_analyzer: ForensicAnalyzer::new(),
            items_shared,
            stats_shared,
            current_dir: current_dir.clone(),
            command_input: String::new(),
            directory_items: Vec::new(),
            command_history: CommandHistory::new(100),
            tab_completer,
            completion_suggestions: Vec::new(),
            completion_index: None,
            bookmarks,
            file_carver: FileCarver::new(),
            signature_detector: SignatureDetector::new(),
            report_exporter: ReportExporter,
            encryption_detector: EncryptionDetector,
            memory_analyzer: MemoryDumpAnalyzer,
        };
        
        // Load initial directory listing
        app.refresh_directory_listing();
        app
    }
    
    pub fn refresh_directory_listing(&mut self) {
        self.directory_items.clear();
        
        // Check if it's a network path and if we have access
        if is_network_path(&self.current_dir) {
            if !check_network_access(&self.current_dir) {
                self.status_message = format!("Cannot access network path: {}", self.current_dir.display());
                self.status_timestamp = Some(Instant::now());
                return;
            }
            // Normalize network path
            let normalized = normalize_network_path(&self.current_dir);
            self.current_dir = normalized;
        }
        
        if let Ok(entries) = std::fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                self.directory_items.push(entry.path());
            }
        }
        
        // Sort: directories first, then files
        self.directory_items.sort_by(|a, b| {
            let a_is_dir = a.is_dir();
            let b_is_dir = b.is_dir();
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });
        
        // Update tab completer
        self.tab_completer.update_dir(self.current_dir.clone());
    }
    
    pub fn start_command_mode(&mut self) {
        self.mode = AppMode::Command;
        self.command_input.clear();
        self.status_message = "Command mode (type 'cd <path>' to navigate, Esc to cancel)".to_string();
    }
    
    pub fn handle_command(&mut self) {
        let cmd = self.command_input.trim();
        
        if cmd.is_empty() {
            self.mode = AppMode::Browser;
            return;
        }
        
        // Add to history
        self.command_history.add(cmd.to_string());
        
        if cmd.starts_with("cd ") {
            let path_str = &cmd[3..].trim();
            if path_str.is_empty() {
                // cd without arguments - go to home/user directory
                #[cfg(windows)]
                let home = std::env::var("USERPROFILE");
                #[cfg(not(windows))]
                let home = std::env::var("HOME");
                
                if let Ok(home_path) = home {
                    self.change_directory(&PathBuf::from(home_path));
                }
            } else {
                let new_path = if path_str.starts_with("\\") || path_str.starts_with("/") || path_str.contains(':') {
                    // Absolute path
                    PathBuf::from(path_str)
                } else {
                    // Relative path
                    self.current_dir.join(path_str)
                };
                self.change_directory(&new_path);
            }
        } else if cmd == "pwd" {
            self.status_message = format!("Current directory: {}", self.current_dir.display());
            self.status_timestamp = Some(Instant::now());
        } else if cmd.starts_with("bookmark ") {
            let name = &cmd[9..].trim();
            if !name.is_empty() {
                if let Err(e) = self.bookmarks.add(name.to_string(), self.current_dir.clone()) {
                    self.status_message = format!("Failed to add bookmark: {}", e);
                } else {
                    self.status_message = format!("Bookmarked: {} -> {}", name, self.current_dir.display());
                }
                self.status_timestamp = Some(Instant::now());
            }
        } else if cmd.starts_with("goto ") {
            let name = &cmd[5..].trim();
            if let Some(path) = self.bookmarks.get_path(name) {
                let path_clone = path.clone();
                self.change_directory(&path_clone);
            } else {
                self.status_message = format!("Bookmark not found: {}", name);
                self.status_timestamp = Some(Instant::now());
            }
        } else if cmd == "bookmarks" || cmd == "bm" {
            let bm_list: Vec<String> = self.bookmarks.list().iter()
                .map(|b| format!("{} -> {}", b.name, b.path.display()))
                .collect();
            self.status_message = if bm_list.is_empty() {
                "No bookmarks".to_string()
            } else {
                bm_list.join(", ")
            };
            self.status_timestamp = Some(Instant::now());
        } else if cmd.starts_with("export ") {
            let format = cmd[7..].trim().to_string();
            self.export_reports(&format);
        } else if !cmd.is_empty() {
            self.status_message = format!("Unknown command: {}. Type 'help' for commands", cmd);
            self.status_timestamp = Some(Instant::now());
        }
        
        self.mode = AppMode::Browser;
        self.command_input.clear();
        self.completion_suggestions.clear();
        self.completion_index = None;
    }
    
    pub fn export_reports(&mut self, format: &str) {
        match format {
            "json" | "JSON" => {
                let reports: Vec<_> = self.items.iter()
                    .map(|entry| self.forensic_analyzer.analyze_file(entry))
                    .collect();
                
                let output_path = self.current_dir.join("forensic_report.json");
                if let Err(e) = self.report_exporter.export_json(&reports, &output_path) {
                    self.status_message = format!("Export failed: {}", e);
                } else {
                    self.status_message = format!("Exported to: {}", output_path.display());
                }
            }
            "csv" | "CSV" => {
                let output_path = self.current_dir.join("forensic_report.csv");
                if let Err(e) = self.report_exporter.export_files_csv(&self.items, &output_path) {
                    self.status_message = format!("Export failed: {}", e);
                } else {
                    self.status_message = format!("Exported to: {}", output_path.display());
                }
            }
            _ => {
                self.status_message = "Unknown format. Use 'json' or 'csv'".to_string();
            }
        }
        self.status_timestamp = Some(Instant::now());
    }
    
    pub fn change_directory(&mut self, path: &PathBuf) {
        let resolved = if path.is_absolute() {
            path.clone()
        } else {
            self.current_dir.join(path)
        };
        
        // Try to canonicalize, but fall back to the path if it fails
        let final_path = resolved.canonicalize().unwrap_or_else(|_| {
            // If canonicalize fails (e.g., path doesn't exist yet), use as-is
            resolved
        });
        
        if final_path.is_dir() {
            self.current_dir = final_path;
            self.target = Some(self.current_dir.clone());
            self.refresh_directory_listing();
            self.selected_index = 0;
            self.scroll_offset = 0;
            self.status_message = format!("Changed to: {}", self.current_dir.display());
            self.status_timestamp = Some(Instant::now());
        } else {
            self.status_message = format!("Not a directory: {}", path.display());
            self.status_timestamp = Some(Instant::now());
        }
    }
    
    pub fn navigate_back(&mut self) {
        if self.is_scanning {
            return; // Don't navigate back while scanning
        }
        
        // Navigate to parent directory
        if let Some(parent) = self.current_dir.parent() {
            self.change_directory(&parent.to_path_buf());
        } else {
            // Already at root, show message
            self.status_message = "Already at root directory".to_string();
            self.status_timestamp = Some(Instant::now());
        }
    }
    
    pub fn add_char_to_command(&mut self, c: char) {
        self.command_input.push(c);
        // Update completion suggestions
        self.update_completion();
    }
    
    pub fn delete_char_from_command(&mut self) {
        self.command_input.pop();
        // Update completion suggestions
        self.update_completion();
    }
    
    pub fn update_completion(&mut self) {
        self.completion_suggestions = self.tab_completer.complete(&self.command_input);
        self.completion_index = None;
    }
    
    pub fn navigate_history_up(&mut self) {
        if let Some(cmd) = self.command_history.previous() {
            self.command_input = cmd.clone();
            self.update_completion();
        }
    }
    
    pub fn navigate_history_down(&mut self) {
        if let Some(cmd) = self.command_history.next() {
            self.command_input = cmd.clone();
            self.update_completion();
        } else {
            self.command_input.clear();
        }
    }
    
    pub fn complete_command(&mut self) {
        if let Some(completion) = self.tab_completer.complete_command(&self.command_input) {
            self.command_input = completion;
            self.update_completion();
        } else if !self.completion_suggestions.is_empty() {
            // Cycle through suggestions
            let index = self.completion_index.unwrap_or(0);
            let next_index = (index + 1) % self.completion_suggestions.len();
            self.completion_index = Some(next_index);
            if let Some(suggestion) = self.completion_suggestions.get(next_index) {
                // For cd commands, append the suggestion
                if self.command_input.starts_with("cd ") {
                    let base = &self.command_input[..3];
                    self.command_input = format!("{}{}", base, suggestion);
                } else {
                    self.command_input = suggestion.clone();
                }
            }
        }
    }

    pub fn start_scan(&mut self) {
        if self.is_scanning {
            return;
        }

        self.is_scanning = true;
        self.is_paused = false;
        self.scan_progress = 0.0;
        self.scan_stats = ScanStats::default();
        self.items.clear();
        self.status_message = "Scanning...".to_string();
        self.status_timestamp = Some(Instant::now());

        // Start scan in background thread
        let target = self.target.clone();
        let pattern = self.pattern.clone();
        let deep = self.deep_scan;
        let engine = Arc::clone(&self.recovery_engine);
        let items = Arc::clone(&self.items_shared);
        let stats = Arc::clone(&self.stats_shared);

        std::thread::spawn(move || {
            let mut engine = engine.lock().unwrap();
            if let Some(ref path) = target {
                engine.scan_path(path, pattern.as_deref(), deep, |entry, _progress, scan_stats| {
                    items.lock().unwrap().push(entry);
                    *stats.lock().unwrap() = scan_stats;
                });
            }
        });
    }

    pub fn toggle_pause(&mut self) {
        if self.is_scanning {
            self.is_paused = !self.is_paused;
            self.status_message = if self.is_paused {
                "Scan paused (Press 'p' to resume)".to_string()
            } else {
                "Scanning...".to_string()
            };
        }
    }

    pub fn previous_item(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
        }
    }

    pub fn next_item(&mut self) {
        let max_items = if self.is_scanning && !self.items.is_empty() {
            self.items.len()
        } else {
            self.directory_items.len()
        };
        
        if self.selected_index < max_items.saturating_sub(1) {
            self.selected_index += 1;
            let max_visible = 20; // Adjust based on terminal height
            if self.selected_index >= self.scroll_offset + max_visible {
                self.scroll_offset = self.selected_index - max_visible + 1;
            }
        }
    }

    pub fn recover_selected(&mut self) {
        if let Some(entry) = self.items.get(self.selected_index) {
            self.status_message = format!("Recovering: {}", entry.path.display());
            self.status_timestamp = Some(Instant::now());
            
            // Recovery logic
            if let Ok(engine) = self.recovery_engine.lock() {
                if let Some(ref output) = self.target {
                    let output_path = output.join("recovered").join(
                        entry.path.file_name().unwrap_or_default()
                    );
                    if let Err(e) = engine.recover_file(entry, &output_path) {
                        self.status_message = format!("Recovery failed: {}", e);
                    } else {
                        self.status_message = format!("Recovered to: {}", output_path.display());
                    }
                } else {
                    self.status_message = "No output directory specified".to_string();
                }
            }
        }
    }

    pub fn toggle_forensic_mode(&mut self) {
        self.mode = if self.mode == AppMode::Forensic {
            AppMode::Browser
        } else {
            AppMode::Forensic
        };
    }

    pub fn start_search(&mut self) {
        self.mode = AppMode::Search;
    }

    pub fn cancel_action(&mut self) {
        match self.mode {
            AppMode::Search => self.mode = AppMode::Browser,
            AppMode::Command => {
                self.mode = AppMode::Browser;
                self.command_input.clear();
            }
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        // Update scan progress from engine
        if self.is_scanning && !self.is_paused {
            // Sync items and stats from background thread
            if let Ok(items) = self.items_shared.try_lock() {
                self.items = items.clone();
            }
            if let Ok(stats) = self.stats_shared.try_lock() {
                self.scan_stats = stats.clone();
            }
            
            // Update progress based on stats
            if self.scan_stats.files_scanned > 0 {
                self.scan_progress = (self.scan_stats.files_scanned as f64 / 
                    (self.scan_stats.files_scanned + 100) as f64).min(1.0);
            }
        }

        // Clear status messages after 3 seconds
        if let Some(timestamp) = self.status_timestamp {
            if timestamp.elapsed() > Duration::from_secs(3) {
                if !self.is_scanning && self.mode != AppMode::Command {
                    self.status_message = String::new();
                    self.status_timestamp = None;
                }
            }
        }
    }
}


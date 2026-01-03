// Command history and tab completion for command mode
use std::collections::VecDeque;
use std::path::PathBuf;

pub struct CommandHistory {
    history: VecDeque<String>,
    max_size: usize,
    current_index: Option<usize>,
}

impl CommandHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_size),
            max_size,
            current_index: None,
        }
    }
    
    pub fn add(&mut self, command: String) {
        // Don't add if it's the same as the last command
        if let Some(last) = self.history.back() {
            if last == &command {
                return;
            }
        }
        
        if self.history.len() >= self.max_size {
            self.history.pop_front();
        }
        
        self.history.push_back(command);
        self.current_index = None;
    }
    
    pub fn previous(&mut self) -> Option<&String> {
        let len = self.history.len();
        if len == 0 {
            return None;
        }
        
        let index = match self.current_index {
            Some(i) if i > 0 => i - 1,
            Some(_) => return None,
            None => len - 1,
        };
        
        self.current_index = Some(index);
        self.history.get(index)
    }
    
    pub fn next(&mut self) -> Option<&String> {
        let len = self.history.len();
        if len == 0 {
            return None;
        }
        
        let index = match self.current_index {
            Some(i) if i < len - 1 => i + 1,
            Some(_) => {
                self.current_index = None;
                return None;
            }
            None => return None,
        };
        
        self.current_index = Some(index);
        self.history.get(index)
    }
    
    pub fn reset_navigation(&mut self) {
        self.current_index = None;
    }
    
    pub fn get_all(&self) -> &VecDeque<String> {
        &self.history
    }
}

pub struct TabCompleter {
    current_dir: PathBuf,
}

impl TabCompleter {
    pub fn new(current_dir: PathBuf) -> Self {
        Self { current_dir }
    }
    
    pub fn update_dir(&mut self, dir: PathBuf) {
        self.current_dir = dir;
    }
    
    pub fn complete(&self, input: &str) -> Vec<String> {
        let input = input.trim();
        
        // Handle cd command completion
        if input.starts_with("cd ") {
            let path_part = &input[3..].trim();
            return self.complete_path(path_part);
        }
        
        // Handle other commands
        let commands = vec!["cd", "pwd", "ls", "help"];
        commands
            .into_iter()
            .filter(|cmd| cmd.starts_with(input))
            .map(|s| s.to_string())
            .collect()
    }
    
    fn complete_path(&self, path: &str) -> Vec<String> {
        let base_path = if path.starts_with('/') || path.contains(':') {
            // Absolute path
            PathBuf::from(path)
        } else {
            // Relative path
            self.current_dir.join(path)
        };
        
        let parent = base_path.parent().unwrap_or(&self.current_dir);
        let search_name = base_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        let mut matches = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(parent) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                
                if name_str.starts_with(search_name) {
                    let full_path = entry.path();
                    let display = if full_path.is_dir() {
                        format!("{}/", name_str)
                    } else {
                        name_str.to_string()
                    };
                    matches.push(display);
                }
            }
        }
        
        matches.sort();
        matches
    }
    
    pub fn complete_command(&self, input: &str) -> Option<String> {
        let completions = self.complete(input);
        if completions.len() == 1 {
            Some(completions[0].clone())
        } else {
            None
        }
    }
}


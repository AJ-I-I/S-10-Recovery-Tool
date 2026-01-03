# S-10
# Data Recovery Tool

File recovery and forensic analysis tool written in Rust, featuring a terminal-based user interface (TUI).

## Features

### File Recovery
- **Deleted File Recovery**: Scan and recover deleted files from NTFS volumes
- **Scanning**: Comprehensive disk scanning with configurable depth
- **Pattern Matching**: Search for files using regex patterns

### Forensic Capabilities
- **Metadata Extraction**: Extract file metadata including timestamps, attributes
- **Hash Calculation**: SHA256 hashing for file integrity
- **File Type Detection**: Automatic file type identification
- **Encryption Detection**: Identify encrypted files through entropy analysis and signature detection

### Terminal User Interface
- **Interactive TUI**: Rich terminal interface built with Ratatui
- **Real-time Updates**: Live scan progress and statistics
- **File Browser**: Navigate through recovered files and directories

## Requirements
- Rust (2024)
- Windows 10/11
- Administrator privileges

## Building

### Using Cargo

```bash
cargo build --release
```

The optimized release binary will be in `target/release/s10-recovery-tool.exe`

### Using PowerShell Script

A convenient PowerShell script is provided for building and running:

```powershell
# Build and run in debug mode
.\run.ps1 -Target "C:\Users\Documents"

# Build and run in release mode
.\run.ps1 -Target "C:\" -Release

# Get some help
.\run.ps1 -Help
```

The script automatically:
- Checks for Rust/Cargo installation
- Builds the project if needed
- Runs the application with specified arguments
- Provides automated build and packaging pipeline

## Quick Start

1. **Build the project:**
   ```bash
   cargo build --release
   ```

2. **Run the application:**
   ```bash
   .\target\release\s10-recovery-tool.exe
   ```
   Or use the PowerShell script:
   ```powershell
   .\run.ps1
   ```

3. **Navigate directories:**
   - Press `:` to enter command mode
   - Type `cd C:\Users\Documents` and press Enter
   - Use arrow keys to browse files and directories
   - Press Enter on a directory to navigate into it

4. **Start scanning:**
   - Press `s` to start a scan of the current directory
   - Press `p` to pause/resume
   - Press `q` to quit

## Usage

### Command Line Arguments

```bash
# Start with default directory (current working directory)
s10-recovery-tool

# Scan a specific directory
s10-recovery-tool --target C:\Users\Documents

# Search for files matching a pattern
s10-recovery-tool --target C:\ --pattern ".*\.(pdf|doc)$"

# Deep scan (slower but more thorough)
s10-recovery-tool --target C:\ --deep

# Specify output directory for recovered files
s10-recovery-tool --target C:\ --output D:\Recovered

# Combine options
s10-recovery-tool --target C:\Users --pattern "\.txt$" --deep --output D:\Recovered
```

### PowerShell Script Usage

```powershell
# Basic usage - scan current directory
.\run.ps1

# Scan specific directory
.\run.ps1 -Target "C:\Users\Documents"

# Deep scan with pattern matching
.\run.ps1 -Target "C:\" -Pattern "\.pdf$" -Deep

# Build and run in release mode
.\run.ps1 -Target "D:\" -Release

# Show all available options
.\run.ps1 -Help
```

### TUI Controls

   #### Navigation
- `↑/↓` - Navigate file/directory list
- `Enter` - Open directory or recover selected file
- `Ctrl+B` - Navigate back to parent directory
- `:` - Enter command mode for directory navigation
   #### Commands
- `s` - Start scan
- `q` - Quit application
- `f` - Toggle forensic mode
- `p` - Pause/resume scan
- `/` - Start search mode
- `Esc` - Cancel current action or exit command mode

#### Command Mode
Press `:` to enter command mode, then type:

- `cd <path>` - Change directory
  - `cd C:\Users` - Navigate to absolute path
  - `cd Documents` - Navigate to relative path
  - `cd ..` - Go to parent directory
  - `cd` - Go to user home directory
- `pwd` - Show current directory path


#### Directory Navigation Instructions
- The TUI displays the current directory in the title bar
- Directories marked with `[DIR]`
- Files marked with `[FILE]`
- Press `Enter` on a directory to navigate into it
- Press `Ctrl+B` to navigate back to the parent directory
pub mod app;
pub mod ui;
pub mod command;
pub mod bookmarks;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::prelude::*;
use std::io;

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut app::App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.mode {
                    app::AppMode::Command => {
                        match key.code {
                            KeyCode::Enter => app.handle_command(),
                            KeyCode::Esc => app.cancel_action(),
                            KeyCode::Backspace => app.delete_char_from_command(),
                            KeyCode::Tab => app.complete_command(),
                            KeyCode::Up => app.navigate_history_up(),
                            KeyCode::Down => app.navigate_history_down(),
                            KeyCode::Char(c) => app.add_char_to_command(c),
                            _ => {}
                        }
                    }
                    _ => {
                        // Check for Ctrl+B (navigate back)
                        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('b') {
                            app.navigate_back();
                        } else {
                            match key.code {
                                KeyCode::Char('q') => return Ok(()),
                                KeyCode::Char('s') => app.start_scan(),
                                KeyCode::Char('p') => app.toggle_pause(),
                                KeyCode::Up => app.previous_item(),
                                KeyCode::Down => app.next_item(),
                                KeyCode::Enter => {
                                    if app.mode == app::AppMode::Browser && !app.is_scanning {
                                        // If selected item is a directory, navigate into it
                                        let item_path = app.directory_items.get(app.selected_index).cloned();
                                        if let Some(path) = item_path {
                                            if path.is_dir() {
                                                app.change_directory(&path);
                                            } else {
                                                app.recover_selected();
                                            }
                                        }
                                    } else {
                                        app.recover_selected();
                                    }
                                }
                                KeyCode::Char('f') => app.toggle_forensic_mode(),
                                KeyCode::Char('/') => app.start_search(),
                                KeyCode::Char(':') => app.start_command_mode(),
                                KeyCode::Esc => app.cancel_action(),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }

        app.tick();
    }

    Ok(())
}


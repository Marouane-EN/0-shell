pub mod command;
pub mod helper;

use crossterm::{
    cursor::{self, MoveToColumn},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Write, stdout};

use helper::executor::execute;
use helper::parser::{CommandEnum, ParseResult, parse_input};
use helper::print_banner::print_banner;
use helper::state_manager::{RawModeGuard, ShellState};
use helper::ui::{get_byte_index, render_system};
fn main() -> io::Result<()> {
    let _guard = RawModeGuard;
    print_banner();
    enable_raw_mode()?;

    let mut shell = ShellState::new();

    loop {
        // --- 1. SETUP START OF LINE ---
        let current_dir = shell.pwd.get_current_dir();
        let prompt_len = if shell.is_continuation {
            2
        } else {
            current_dir.len() + 2
        };

        try_log!(execute!(stdout(), MoveToColumn(0)), "Cursor reset error");

        let (_, mut start_y) = cursor::position().unwrap_or((0, 0));

        render_system(&shell, prompt_len, &mut start_y, &current_dir);

        // --- 2. INPUT LOOP ---
        loop {
            let event = match event::read() {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("\r\n[Fatal] Input error: {}", e);
                    return Err(e);
                }
            };

            if let Event::Key(key) = event {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char(c) => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'd' {
                            print!("^D\r\n");

                            return Ok(());
                        }
                        if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                            shell.commit_to_history();
                            shell.reset_buffers();
                            shell.is_continuation = false;
                            print!("^C\r\n");
                            break;
                        }

                        let char_count = shell.view_buffer.chars().count();

                        if shell.cursor_idx >= char_count {
                            shell.buffer.push(c);
                            shell.view_buffer.push(c);
                        } else {
                            let view_byte_idx =
                                get_byte_index(&shell.view_buffer, shell.cursor_idx);

                            // Calculate where to insert in the main buffer
                            // (If continuation, buffer includes previous lines)
                            let buffer_base_len = shell.buffer.len() - shell.view_buffer.len();
                            let buffer_byte_idx = buffer_base_len + view_byte_idx;

                            shell.buffer.insert(buffer_byte_idx, c);
                            shell.view_buffer.insert(view_byte_idx, c);
                        }
                        shell.cursor_idx += 1;
                    }

                    KeyCode::Backspace => {
                        if !shell.view_buffer.is_empty() && shell.cursor_idx > 0 {
                            let view_byte_idx =
                                get_byte_index(&shell.view_buffer, shell.cursor_idx - 1);

                            let buffer_base_len = shell.buffer.len() - shell.view_buffer.len();
                            let buffer_byte_idx = buffer_base_len + view_byte_idx;

                            shell.buffer.remove(buffer_byte_idx);
                            shell.view_buffer.remove(view_byte_idx);
                            shell.cursor_idx -= 1;
                        }
                    }

                    KeyCode::Enter => {
                        print!("\r\n");
                        try_log!(stdout().flush(), "Flush error");

                        match parse_input(&shell.buffer) {
                            ParseResult::Ok(cmd) => {
                                shell.commit_to_history();
                                if let CommandEnum::Exit = cmd {
                                    disable_raw_mode()?;
                                    return Ok(());
                                }
                                disable_raw_mode()?;

                                execute(cmd, &mut shell.pwd);
                                enable_raw_mode()?;

                                shell.reset_buffers();
                                shell.is_continuation = false;
                                break;
                            }
                            ParseResult::Incomplete => {
                                shell.buffer.push('\n');
                                shell.view_buffer.clear();
                                shell.is_continuation = true;
                                shell.cursor_idx = 0;
                                break;
                            }
                            ParseResult::Err(e) => {
                                eprintln!("Error: {}", e);
                                shell.reset_buffers();
                                shell.is_continuation = false;
                                break;
                            }
                        }
                    }

                    KeyCode::Up => {
                        if shell.hist_idx > 0 {
                            shell.hist_idx -= 1;
                            shell.buffer = shell.history[shell.hist_idx].clone();
                            shell.view_buffer = shell.history[shell.hist_idx].clone();
                            shell.cursor_idx = shell.view_buffer.chars().count();
                        }
                    }
                    KeyCode::Down => {
                        if shell.hist_idx < shell.history.len() {
                            shell.hist_idx += 1;
                            if shell.hist_idx < shell.history.len() {
                                shell.buffer = shell.history[shell.hist_idx].clone();
                                shell.view_buffer = shell.history[shell.hist_idx].clone();
                                shell.cursor_idx = shell.view_buffer.chars().count();
                            } else {
                                shell.reset_buffers();
                            }
                        }
                    }

                    KeyCode::Left => {
                        if shell.cursor_idx > 0 {
                            shell.cursor_idx -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if shell.cursor_idx < shell.view_buffer.chars().count() {
                            shell.cursor_idx += 1;
                        }
                    }
                    _ => {}
                }

                render_system(&shell, prompt_len, &mut start_y, &current_dir);
            }
        }
    }
}

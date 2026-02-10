pub mod command;
pub mod helper;

use crossterm::{
    cursor::{self, MoveTo, MoveToColumn},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{Clear, ClearType, enable_raw_mode, size},
};
use std::io::{self, Write, stdout};

use helper::parser::{ParseResult, parse_input};
use helper::print_banner::{GREEN, RESET, print_banner};
use helper::state_manager::{RawModeGuard, ShellState};

// 🛠️ HELPER: Convert Character Index -> Byte Index
fn get_byte_index(s: &str, char_idx: usize) -> usize {
    s.chars().take(char_idx).map(|c| c.len_utf8()).sum()
}

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
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(c) => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'd' {
                                print!("\r\nexit\r\n");
                                return Ok(());
                            }
                            if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                                shell.commit_to_history();
                                shell.reset_buffers();
                                shell.is_continuation = false;
                                print!("\r\n");
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
                                ParseResult::Ok(args) => {
                                    if !args.is_empty() {
                                        shell.commit_to_history();
                                        println!("Debug: Executing {:?}", args);
                                    }
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
}

// --- THE RENDER ENGINE ---
fn render_system(shell: &ShellState, prompt_len: usize, start_y: &mut u16, current_dir: &str) {
    // 1. Reset visual state
    try_log!(
        execute!(
            stdout(),
            MoveTo(0, *start_y),
            Clear(ClearType::FromCursorDown)
        ),
        "Clear err"
    );

    // 2. Print Prompt & Buffer
    if !shell.is_continuation {
        print!("{GREEN}{}$ {RESET}", current_dir);
    } else {
        print!("> ");
    }

    let display_text = shell.view_buffer.replace("\n", "\r\n");
    print!("{}", display_text);
    try_log!(stdout().flush(), "Flush err");

    // 3. DETECT SCROLLING
    let (_, term_rows) = size().unwrap_or((80, 24));
    let used_rows = shell.view_buffer.chars().filter(|&c| c == '\n').count() as u16 + 1;
    let expected_end_row = *start_y + used_rows;

    if expected_end_row > term_rows {
        let scroll_amount = expected_end_row - term_rows;
        *start_y = start_y.saturating_sub(scroll_amount);
    }

    // 4. Calculate Visual Cursor Position
    let byte_idx = get_byte_index(&shell.view_buffer, shell.cursor_idx);
    let active_part = &shell.view_buffer[..byte_idx];

    let cursor_rows = active_part.chars().filter(|&c| c == '\n').count() as u16;
    let last_line_len = active_part.split('\n').last().unwrap_or("").chars().count();

    let target_x = if cursor_rows == 0 {
        (prompt_len + last_line_len) as u16
    } else {
        last_line_len as u16
    };

    let target_y = *start_y + cursor_rows;

    try_log!(
        execute!(stdout(), MoveTo(target_x, target_y)),
        "Cursor move err"
    );
}

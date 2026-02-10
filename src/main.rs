pub mod command;
pub mod helper;

use crossterm::{
    cursor::{self, MoveToColumn},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{Clear, ClearType, enable_raw_mode},
};
use std::io::{self, Write, stdout};

use helper::print_banner::{GREEN, RESET, print_banner};
use helper::state_manager::{
    RawModeGuard, ShellState, clear_and_redraw, load_history, redraw_input,
};

fn main() -> io::Result<()> {
    let _guard = RawModeGuard;
    print_banner();
    enable_raw_mode()?;

    let mut shell = ShellState::new();

    loop {
        // Draw Prompt
        let current_dir = shell.pwd.get_current_dir();
        let prompt_len = if shell.is_continuation {
            2
        } else {
            current_dir.len() + 2
        };

        try_log!(
            execute!(stdout(), MoveToColumn(0), Clear(ClearType::CurrentLine)),
            "Cursor reset error"
        );

        if !shell.is_continuation {
            print!("{GREEN}{}$ {RESET}", current_dir);
        } else {
            print!("> ");
        }
        try_log!(stdout().flush(), "Flush error");

        // Input Handling Loop
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
                    let (x, y) = cursor::position().unwrap_or((0, 0));
                    let cursor_idx = (x as usize).saturating_sub(prompt_len);

                    match key.code {
                        KeyCode::Char(c) => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'd' {
                                print!("\r\n");
                                return Ok(());
                            }
                            if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                                shell.commit_to_history();
                                shell.reset_buffers();
                                shell.is_continuation = false;
                                print!("\r\n");
                                break;
                            }

                            if cursor_idx >= shell.view_buffer.len() {
                                shell.buffer.push(c);
                                shell.view_buffer.push(c);
                            } else {
                                shell.buffer.insert(cursor_idx, c);
                                shell.view_buffer.insert(cursor_idx, c);
                            }

                            redraw_input(&shell.view_buffer, prompt_len, y, cursor_idx + 1);
                        }

                        KeyCode::Backspace => {
                            if !shell.view_buffer.is_empty()
                                && cursor_idx > 0
                                && cursor_idx <= shell.view_buffer.len()
                            {
                                shell.buffer.remove(cursor_idx - 1);
                                shell.view_buffer.remove(cursor_idx - 1);
                                redraw_input(&shell.view_buffer, prompt_len, y, cursor_idx - 1);
                            }
                        }

                        KeyCode::Enter => {
                            print!("\r\n");
                            try_log!(stdout().flush(), "Flush error");
                            shell.commit_to_history();
                            shell.reset_buffers();
                            break;
                        }

                        KeyCode::Up => {
                            if shell.hist_idx > 0 {
                                shell.hist_idx -= 1;
                                load_history(&mut shell, prompt_len);
                            }
                        }
                        KeyCode::Down => {
                            if shell.hist_idx < shell.history.len() {
                                shell.hist_idx += 1;
                                if shell.hist_idx < shell.history.len() {
                                    load_history(&mut shell, prompt_len);
                                } else {
                                    shell.reset_buffers();
                                    clear_and_redraw(&shell, prompt_len);
                                }
                            }
                        }

                        KeyCode::Left => {
                            if cursor_idx > 0 {
                                try_log!(execute!(stdout(), cursor::MoveTo(x - 1, y)), "Move Err");
                            }
                        }
                        KeyCode::Right => {
                            if cursor_idx < shell.buffer.len() {
                                try_log!(execute!(stdout(), cursor::MoveTo(x + 1, y)), "Move Err");
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

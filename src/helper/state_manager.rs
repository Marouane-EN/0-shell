use super::print_banner::{GREEN, RESET};
use crate::command::pwd::PwdState;
use crossterm::{
    cursor::{self, MoveToColumn},
    execute,
    terminal::{Clear, ClearType, disable_raw_mode},
};
use std::io::{Write, stdout};
use std::{env, path::PathBuf};
// --- Helper Macro to reduce noise ---
// This replaces all those repetitive `if let Err(e) = ...` blocks
#[macro_export]
macro_rules! try_log {
    ($res:expr, $msg:expr) => {
        if let Err(e) = $res {
            eprintln!("\r\n[Warning] {}: {}", $msg, e);
        }
    };
}
// --- cleanup guard ---
pub struct RawModeGuard;
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

// --- State Management ---
pub struct ShellState {
    pub history: Vec<String>,
    pub hist_idx: usize,
    pub buffer: String,
    pub view_buffer: String,
    pub is_continuation: bool,
    pub pwd: PwdState,
}

impl ShellState {
    pub fn new() -> Self {
        // robust CWD handling
        let cwd = env::current_dir().unwrap_or_else(|_| {
            eprintln!("\r\n[Warning] Failed to get CWD. Defaulting to /");
            PathBuf::from("/")
        });
        let path_str = cwd.to_string_lossy().to_string();

        Self {
            history: Vec::new(),
            hist_idx: 0,
            buffer: String::new(),
            view_buffer: String::new(),
            is_continuation: false,
            pwd: PwdState::new(path_str.clone(), path_str),
        }
    }

    pub fn reset_buffers(&mut self) {
        self.buffer.clear();
        self.view_buffer.clear();
    }

    pub fn commit_to_history(&mut self) {
        if !self.buffer.trim().is_empty() {
            if self.history.last() != Some(&self.buffer) {
                self.history.push(self.buffer.clone());
            }
        }
        self.hist_idx = self.history.len();
    }
}

pub fn redraw_input(text: &str, prompt_len: usize, y: u16, new_cursor_offset: usize) {
    try_log!(
        execute!(
            stdout(),
            cursor::MoveToColumn(prompt_len as u16),
            Clear(ClearType::UntilNewLine)
        ),
        "Clear Err"
    );
    print!("{}", text);
    try_log!(
        execute!(
            stdout(),
            cursor::MoveTo((prompt_len + new_cursor_offset) as u16, y)
        ),
        "Move Err"
    );
    try_log!(stdout().flush(), "Flush Err");
}

pub fn load_history(shell: &mut ShellState, prompt_len: usize) {
    shell.buffer = shell.history[shell.hist_idx].clone();
    shell.view_buffer = shell.history[shell.hist_idx].clone();
    clear_and_redraw(shell, prompt_len);
}

pub fn clear_and_redraw(shell: &ShellState, _prompt_len: usize) {
    try_log!(
        execute!(stdout(), Clear(ClearType::CurrentLine), MoveToColumn(0)),
        "Redraw Err"
    );

    let prompt_txt = if !shell.is_continuation {
        format!("{GREEN}{}$ {RESET}", shell.pwd.get_current_dir())
    } else {
        "> ".to_string()
    };

    print!("{}{}", prompt_txt, shell.view_buffer.replace("\n", "\r\n"));
    try_log!(stdout().flush(), "Flush Err");
}

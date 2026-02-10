use crate::command::pwd::PwdState;
use crossterm::terminal::disable_raw_mode;
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
    pub cursor_idx: usize,
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
            cursor_idx: 0,
        }
    }

    pub fn reset_buffers(&mut self) {
        self.buffer.clear();
        self.view_buffer.clear();
        self.cursor_idx = 0;
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

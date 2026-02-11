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

#[cfg(test)]
mod tests {
    use super::*;

    // 1. Helper to create a dummy state for testing
    fn create_test_state() -> ShellState {
        ShellState {
            history: Vec::new(),
            hist_idx: 0,
            buffer: String::new(),
            view_buffer: String::new(),
            is_continuation: false,
            // Mocking PwdState for the test
            pwd: PwdState::new("/".to_string(), "/".to_string()),
            cursor_idx: 0,
        }
    }

    #[test]
    fn test_reset_buffers() {
        let mut state = create_test_state();

        // Simulate a dirty state (user typed something)
        state.buffer = "echo hello".to_string();
        state.view_buffer = "echo hello".to_string();
        state.cursor_idx = 10;

        // Run the function
        state.reset_buffers();

        // Assert everything is clean
        assert_eq!(state.buffer, "");
        assert_eq!(state.view_buffer, "");
        assert_eq!(state.cursor_idx, 0);
    }

    #[test]
    fn test_commit_to_history_adds_valid_entry() {
        let mut state = create_test_state();
        state.buffer = "ls -la".to_string();

        state.commit_to_history();

        assert_eq!(state.history.len(), 1);
        assert_eq!(state.history[0], "ls -la");
        assert_eq!(state.hist_idx, 1); // Should point to the new end
    }

    #[test]
    fn test_commit_to_history_ignores_empty() {
        let mut state = create_test_state();
        state.buffer = "   ".to_string(); // Whitespace only

        state.commit_to_history();

        assert!(state.history.is_empty());
    }

    #[test]
    fn test_commit_to_history_ignores_duplicates() {
        let mut state = create_test_state();

        // Add first command
        state.buffer = "git status".to_string();
        state.commit_to_history();

        // Try to add the exact same command again
        state.buffer = "git status".to_string();
        state.commit_to_history();

        // History should still only have 1 item
        assert_eq!(state.history.len(), 1);
    }
}

#[derive(Debug, Clone)]
pub struct PwdState {
    current_dir: String,
    old_dir: String,
}

impl PwdState {
    pub fn new(current_dir: String, old_dir: String) -> Self {
        Self {
            current_dir,
            old_dir,
        }
    }
    pub fn set_states(&mut self, new_current: String, new_old: String) {
        self.current_dir = new_current;
        self.old_dir = new_old;
    }

    pub fn get_current_dir(&self) -> String {
        self.current_dir.clone()
    }

    pub fn get_old_dir(&self) -> String {
        self.old_dir.clone()
    }
}

// ... existing code ...

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        // Test that the constructor sets values correctly
        let pwd = PwdState::new("/home/user".to_string(), "/home".to_string());

        assert_eq!(pwd.get_current_dir(), "/home/user");
        assert_eq!(pwd.get_old_dir(), "/home");
    }

    #[test]
    fn test_update_state() {
        let mut pwd = PwdState::new("/var".to_string(), "/".to_string());

        // Simulate 'cd /etc'
        // New Current: /etc, New Old: /var
        pwd.set_states("/etc".to_string(), "/var".to_string());

        assert_eq!(pwd.get_current_dir(), "/etc");
        assert_eq!(pwd.get_old_dir(), "/var");
    }
}

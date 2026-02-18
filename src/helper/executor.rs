use crate::command::{cd::command_cd, cp::cp, ls::ls, mv::mv, pwd::PwdState};
use crate::helper::parser::CommandEnum;
use std::process::Command;

pub fn execute(cmd: CommandEnum, pwd_state: &mut PwdState) {
    match cmd {
        // Built-ins
        CommandEnum::Cd(args, raw) => command_cd(args, raw, pwd_state),

        CommandEnum::Pwd => {
            println!("{}", pwd_state.get_current_dir());
        }
        CommandEnum::Clear => {
            print!("\x1Bc");
        }
        CommandEnum::Exit => { /* Handled in main, but good safety net */ }
        CommandEnum::Unknown(raw_cmd) => {
            eprintln!("command not found: {}", raw_cmd);
        }

        // EXTERNAL COMMANDS (The Fix)
        // We map all these variants to a helper function
        CommandEnum::Ls(args) => ls(args),
        CommandEnum::Cat(args) => run_external("cat", &args),
        CommandEnum::Rm(args) => run_external("rm", &args),
        CommandEnum::Cp(args) => cp(args),
        CommandEnum::Mv(args) => mv(args),
        CommandEnum::Echo(args) => run_external("echo", &args),
        CommandEnum::Mkdir(_raw, args) => run_external("mkdir", &args),
    }
}

// Helper to spawn processes
fn run_external(cmd: &str, args: &[String]) {
    let child = Command::new(cmd).args(args).spawn();

    match child {
        Ok(mut child) => {
            let _ = child.wait();
        }
        Err(e) => eprintln!("Failed to execute {}: {}", cmd, e),
    }
}

// ... (Keep existing code above)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::pwd::PwdState;

    // Helper to create a dummy PwdState
    fn mock_pwd() -> PwdState {
        PwdState::new("/tmp".to_string(), "/".to_string())
    }

    #[test]
    fn test_execute_pwd_runs_without_crash() {
        // This test verifies that the built-in Pwd logic runs.
        // Capturing stdout in Rust tests requires a specific setup,
        // so for now we ensure it simply doesn't panic.
        let mut pwd = mock_pwd();
        execute(CommandEnum::Pwd, &mut pwd);
    }

    #[test]
    fn test_execute_unknown_runs_without_crash() {
        let mut pwd = mock_pwd();
        let cmd = CommandEnum::Unknown("blarg".to_string());
        execute(cmd, &mut pwd); // Should print error to stderr, but not crash
    }

    // Note: Testing 'Ls', 'Echo', etc. requires the actual 'ls' binary to exist
    // on your computer. This is an "Integration Test".
    #[test]
    fn test_execute_echo_integration() {
        let mut pwd = mock_pwd();
        // This tries to actually spawn "echo hello"
        let cmd = CommandEnum::Echo(vec!["hello".to_string()]);
        execute(cmd, &mut pwd);
        // Pass if no panic occurs
    }
}

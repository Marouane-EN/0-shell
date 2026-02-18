#[derive(Debug, PartialEq)]
pub enum ParseResult {
    Ok(CommandEnum),
    Incomplete,
    Err(String),
}

#[derive(Debug, PartialEq)]
pub enum CommandEnum {
    Rm(Vec<String>),
    Cp(Vec<String>),
    Mv(Vec<String>),
    Pwd,
    Cd(Vec<String>, Vec<String>),
    Echo(Vec<String>),
    Mkdir(Vec<String>, Vec<String>),
    Exit,
    Unknown(String),
    Cat(Vec<String>),
    Ls(Vec<String>),
    Clear,
}

pub fn parse_tokens(input: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();

    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if in_single_quote {
            // SINGLE QUOTE MODE: preserve everything literal until closing '
            if c == '\'' {
                in_single_quote = false;
            } else {
                current_arg.push(c);
            }
        } else if in_double_quote {
            // DOUBLE QUOTE MODE: allow escape characters
            if escaped {
                current_arg.push(c);
                escaped = false;
            } else if c == '\\' {
                // If next char is a special one, escape it. Otherwise keep \
                if let Some(&next) = chars.peek() {
                    if next == '"' || next == '\\' {
                        escaped = true;
                    } else {
                        current_arg.push('\\');
                    }
                } else {
                    escaped = true; // Trailing backslash
                }
            } else if c == '"' {
                in_double_quote = false;
            } else {
                current_arg.push(c);
            }
        } else {
            // NORMAL MODE
            if escaped {
                current_arg.push(c);
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '\'' {
                in_single_quote = true;
            } else if c == '"' {
                in_double_quote = true;
            } else if c.is_whitespace() {
                if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }
            } else {
                current_arg.push(c);
            }
        }
    }

    // Check for unclosed quotes
    if in_single_quote || in_double_quote || escaped {
        return Err("Incomplete".to_string());
    }

    // Push the last argument if exists
    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    Ok(args)
}

pub fn parse_input(input: &str) -> ParseResult {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return ParseResult::Ok(CommandEnum::Unknown("".to_string()));
    }

    match parse_tokens(trimmed) {
        Ok(args) => {
            if args.is_empty() {
                return ParseResult::Ok(CommandEnum::Unknown("".to_string()));
            }
            let cmd_name = &args[0];
            let cmd_args = args[1..].to_vec();

            let raw_args = cmd_args.clone();
            let clean_args: Vec<String> = cmd_args
                .iter()
                .map(|ele| ele.replace("\n", "\\n"))
                .collect();

            let parsed = match cmd_name.as_str() {
                "ls" => CommandEnum::Ls(clean_args),
                "cat" => CommandEnum::Cat(clean_args),
                "cp" => CommandEnum::Cp(clean_args),
                "pwd" => CommandEnum::Pwd,
                "cd" => CommandEnum::Cd(clean_args, raw_args),
                "echo" => CommandEnum::Echo(raw_args),
                "rm" => CommandEnum::Rm(clean_args),
                "mkdir" => CommandEnum::Mkdir(raw_args, clean_args),
                "mv" => CommandEnum::Mv(clean_args),
                "exit" => CommandEnum::Exit,
                "clear" => CommandEnum::Clear,
                _ => CommandEnum::Unknown(cmd_name.clone()),
            };

            ParseResult::Ok(parsed)
        }
        Err(_) => ParseResult::Incomplete,
    }
}

// ... (Keep existing code above)

#[cfg(test)]
mod tests {
    use super::*;

    // --- 1. Test Tokenization Logic ---
    #[test]
    fn test_tokenize_simple() {
        let input = "ls -la";
        let expected = Ok(vec!["ls".to_string(), "-la".to_string()]);
        assert_eq!(parse_tokens(input), expected);
    }

    #[test]
    fn test_tokenize_quotes() {
        let input = "echo 'hello world' \"formatted string\"";
        let expected = Ok(vec![
            "echo".to_string(),
            "hello world".to_string(),
            "formatted string".to_string(),
        ]);
        assert_eq!(parse_tokens(input), expected);
    }

    #[test]
    fn test_tokenize_escaped_quotes() {
        // Input: echo "He said \"Hello\""
        let input = r#"echo "He said \"Hello\"""#;
        let expected = Ok(vec![
            "echo".to_string(),
            "He said \"Hello\"".to_string(), // Parser keeps the internal quotes
        ]);
        assert_eq!(parse_tokens(input), expected);
    }

    #[test]
    fn test_tokenize_incomplete_fails() {
        let input = "echo \"missing quote";
        assert_eq!(parse_tokens(input), Err("Incomplete".to_string()));
    }

    // --- 2. Test Enum Mapping Logic ---
    #[test]
    fn test_parse_input_ls() {
        let input = "ls -la /tmp";
        match parse_input(input) {
            ParseResult::Ok(CommandEnum::Ls(args)) => {
                assert_eq!(args, vec!["-la", "/tmp"]);
            }
            _ => panic!("Expected CommandEnum::Ls"),
        }
    }

    #[test]
    fn test_parse_input_cd() {
        let input = "cd /home/user";
        match parse_input(input) {
            ParseResult::Ok(CommandEnum::Cd(clean, raw)) => {
                assert_eq!(clean[0], "/home/user");
                assert_eq!(raw[0], "/home/user");
            }
            _ => panic!("Expected CommandEnum::Cd"),
        }
    }

    #[test]
    fn test_parse_input_unknown() {
        let input = "notacommand arg1";
        match parse_input(input) {
            ParseResult::Ok(CommandEnum::Unknown(cmd)) => {
                assert_eq!(cmd, "notacommand");
            }
            _ => panic!("Expected CommandEnum::Unknown"),
        }
    }
}

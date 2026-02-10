#[derive(Debug, PartialEq)]
pub enum ParseResult {
    Ok(Vec<String>),
    Incomplete,
    Err(String),
}

pub fn parse_input(input: &str) -> ParseResult {
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
        return ParseResult::Incomplete;
    }

    // Push the last argument if exists
    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    ParseResult::Ok(args)
}
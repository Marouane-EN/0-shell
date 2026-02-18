use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::enable_raw_mode,
};
use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

pub fn cat(args: Vec<String>) {
    let stdout = io::stdout();
    if args.is_empty() {
        match enable_raw_mode() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to enable raw mode: {}", e);
                return;
            }
        }

        let mut input_buffer = String::new();

        loop {
            if let Event::Key(key_event) = event::read().unwrap()
                && key_event.kind == KeyEventKind::Press
            {
                match key_event.code {
                    KeyCode::Char(c) if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        if c == 'd' {
                            print!("\r\n");
                            break;
                        } else if c == 'c' {
                            print!("^C\r\n");
                            break;
                        }
                        io::stdout().flush().ok();
                    }

                    KeyCode::Char(c) => {
                        print!("{}", c);
                        input_buffer.push(c);
                        io::stdout().flush().ok();
                    }

                    KeyCode::Backspace => {
                        if !input_buffer.is_empty() {
                            input_buffer.pop();
                            print!("\x08 \x08");
                            io::stdout().flush().ok();
                        }
                    }

                    KeyCode::Enter => {
                        print!("\r\n");
                        print!("{}\r\n", input_buffer);
                        io::stdout().flush().ok();
                        input_buffer.clear();
                    }

                    _ => {}
                }
            }
        }
    } else {
        for file in args {
            let source_path = Path::new(&file);
            let file_open = File::open(source_path);
            match file_open {
                Ok(mut f) => match io::copy(&mut f, &mut stdout.lock()) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("cat: {}: {}", file, e);
                    }
                },
                Err(e) => {
                    eprintln!("cat: {}: {}", file, e);
                }
            }
        }
    }
}

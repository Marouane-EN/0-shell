use crate::helper::print_banner::{GREEN, RESET};
use crate::helper::state_manager::ShellState;
use crate::try_log;
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType, size},
};
use std::io::{Write, stdout};

// 🛠️ HELPER: Convert Character Index -> Byte Index
// (Now public so main can use it if needed, or kept private if only used here)
pub fn get_byte_index(s: &str, char_idx: usize) -> usize {
    s.chars().take(char_idx).map(|c| c.len_utf8()).sum()
}

// --- THE RENDER ENGINE ---
pub fn render_system(shell: &ShellState, prompt_len: usize, start_y: &mut u16, current_dir: &str) {
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
    let last_line_len = active_part
        .split('\n')
        .next_back()
        .unwrap_or("")
        .chars()
        .count();

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

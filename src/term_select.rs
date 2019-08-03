use crossterm::{
    cursor, input, terminal, AlternateScreen, InputEvent, KeyEvent, RawScreen, Terminal,
    TerminalCursor, ClearType
};
use std::{iter::Iterator, thread, time};

/// Use skim to show multiple results, where selections is the files to select
pub fn show_multiple_results(selections: &Vec<String>) -> Vec<usize> {
    //    let options = SkimOptionsBuilder::default()
    //        .ansi(true)
    //        .height(Some("50%"))
    //        .multi(true)
    //        .build()
    //        .unwrap();
    //
    //    let joined = selections
    //        .iter()
    //        .fold(String::new(), |acc, s| acc + s + "\n");

    //    let selected_items = Skim::run_with(&options, Some(Box::new(Cursor::new(joined))))
    //        .map(|out| out.selected_items)
    //        .unwrap_or_else(|| Vec::new());
    //
    //    selected_items.iter().map(|item| item.get_index()).collect()

    let screen = RawScreen::into_raw_mode().unwrap();

    let mut term = terminal();
    let (_, term_height) = term.terminal_size();
    
    let term_cursor = cursor();
    term_cursor.hide();

    let input = input();
    let mut stdin = input.read_async();
    loop {
        match stdin.next() {
            Some(InputEvent::Keyboard(KeyEvent::Char('\n'))) => {
                break;
            }
            _ => {}
        }

        write_results(
            &term,
            &term_cursor,
            term_height,
            selections.iter().map(|s| s.as_str()),
        );

        thread::sleep(time::Duration::from_millis(10));
    }

    term_cursor.show();
    Vec::new()
}

fn write_results<'a>(
    terminal: &Terminal,
    cursor: &TerminalCursor,
    term_height: u16,
    selection: impl Iterator<Item = &'a str>,
) {
    cursor.goto(0, term_height);
    terminal.clear(ClearType::CurrentLine);
    for entry in selection {
        terminal.write(format!("{}\r\n", entry));
    }
}

use crossterm::{cursor, input, terminal, AlternateScreen, InputEvent, KeyEvent, RawScreen, Terminal, TerminalCursor, ClearType, Crossterm};
use std::{iter::Iterator, thread, time};
use std::cmp::max;

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

    let crossterm = Crossterm::new();
    let (_, term_height) = terminal().terminal_size();
    let (_, start_cursor_pos) = crossterm.cursor().pos();
    write_results(
        &crossterm.terminal(),
        selections.iter().map(|s| s.as_str()).take(20),
    );

    let input = crossterm.input();
    let mut stdin = input.read_async();
    loop {
        match stdin.next() {
            Some(InputEvent::Keyboard(KeyEvent::Char('\n'))) => {
                break;
            }
            _ => {}
        }

        let (_, cur_pos) = crossterm.cursor().pos();
        crossterm.cursor().move_up(cur_pos - start_cursor_pos);
        write_results(
            &crossterm.terminal(),
            selections.iter().map(|s| s.as_str()).take(20),
        );

        thread::sleep(time::Duration::from_millis(10));
    }

    Vec::new()
}

fn write_results<'a>(
    terminal: &Terminal,
    selection: impl Iterator<Item = &'a str>,
) {
    for entry in selection {
        terminal.write(format!("{}\r\n", entry));
    }
}

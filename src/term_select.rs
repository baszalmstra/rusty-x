use crossterm::{cursor, input, terminal, AlternateScreen, InputEvent, KeyEvent, RawScreen, Terminal, TerminalCursor, ClearType, Crossterm};
use fuzzy_matcher::skim::fuzzy_indices;
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

    let mut matches = FuzzyMatcher::new(selections);

    let screen = RawScreen::into_raw_mode().unwrap();

    let crossterm = Crossterm::new();
    crossterm.cursor().hide();

    let (_, term_height) = terminal().terminal_size();
    let (_, start_cursor_pos) = crossterm.cursor().pos();
    let max_items = term_height/2;

    write_results(
        &crossterm.terminal(),
        &matches,
        max_items
    );

    let input = crossterm.input();
    let mut stdin = input.read_async();

    loop {
        match stdin.next() {
            Some(InputEvent::Keyboard(KeyEvent::Char('\n'))) => {
                break;
            }
            Some(InputEvent::Keyboard(KeyEvent::Char(c))) => {
                let mut search_term = matches.get_search_term().clone();
                search_term.push(c);
                matches.set_search_term(&search_term);
                rewrite_results(&crossterm, &matches, max_items, start_cursor_pos);
            },
            Some(InputEvent::Keyboard(KeyEvent::Backspace)) => {
                let mut search_term = matches.get_search_term().clone();
                search_term.pop();
                matches.set_search_term(&search_term);
                rewrite_results(&crossterm, &matches, max_items, start_cursor_pos);
            },
            _ => {}
        }

        thread::sleep(time::Duration::from_millis(10));
    }

    crossterm.cursor().show();
    Vec::new()
}

fn rewrite_results<'a>(
    crossterm: &Crossterm,
    matches: &FuzzyMatcher<'a>,
    height: u16,
    start_pos: u16,
) {
    crossterm.cursor().goto(0,0);
    let (_, cur_pos) = crossterm.cursor().pos();
    crossterm.cursor().move_down(start_pos -  cur_pos);
    crossterm.terminal().clear(ClearType::CurrentLine);
    write_results(
        &crossterm.terminal(),
        &matches,
        height
    );
    crossterm.terminal().clear(ClearType::CurrentLine);
    crossterm.terminal().write(format!("> {}", matches.get_search_term()));
}

fn write_results<'a>(
    terminal: &Terminal,
    matches: &FuzzyMatcher<'a>,
    height: u16
) {
    // Write empty lines
    for _ in matches.get_matches().len() ..  height as usize {
        terminal.clear(ClearType::CurrentLine);
        terminal.write(format!("\r\n"));
    }

    for (s, score, indices) in matches.get_matches().iter().take(height as usize) {
        terminal.clear(ClearType::CurrentLine);
        terminal.write(format!("{}\r\n", s));
    }
}

struct FuzzyMatcher<'a> {
    selections: &'a Vec<String>,
    matches: Vec<(&'a String, i64, Vec<usize>)>,
    search_term: String,
}

impl<'a> FuzzyMatcher<'a> {
    pub fn new(selections: &'a Vec<String>) -> Self {
        let mut fuzzy = FuzzyMatcher {
            selections,
            matches: Vec::new(),
            search_term: String::new()
        };
        fuzzy.update_matches();
        fuzzy
    }

    pub fn set_search_term(&mut self, search_term: &str) {
        if search_term != self.search_term {
            self.search_term = search_term.to_string();
            self.update_matches();
        }
    }

    pub fn get_search_term(&self) -> &String {
        &self.search_term
    }

    pub fn get_matches(&self) -> &[(&'a String, i64, Vec<usize>)] {
        &self.matches
    }

    fn update_matches(&mut self) {
        let mut matches = self.selections
            .iter()
            .filter_map(|s| match fuzzy_indices(s.as_str(), &self.search_term) {
                Some((score, indices)) => Some((s, score, indices)),
                None => None
            })
            .collect::<Vec<(&String, i64, Vec<usize>)>>();

        matches.sort_by(|(_, score_a, _), (_, score_b, _)| score_a.cmp(score_b));
        self.matches = matches;
    }
}

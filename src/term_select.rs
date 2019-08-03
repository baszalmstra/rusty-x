use crossterm::{cursor, input, terminal, AlternateScreen, InputEvent, KeyEvent, RawScreen, Terminal, TerminalCursor, ClearType, Crossterm, Colored, Color, Attribute, Styler};
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

    let screen = AlternateScreen::to_alternate(true);

    let mut matches = FuzzyMatcher::new(selections);

//    let screen = RawScreen::into_raw_mode().unwrap();

    let crossterm = Crossterm::new();
    crossterm.cursor().hide();

    let (_, term_height) = terminal().terminal_size();
    let (_, start_cursor_pos) = crossterm.cursor().pos();

    rewrite_results(&crossterm, &matches);
    write_input(&crossterm, matches.get_search_term());

    let input = crossterm.input();
    let mut stdin = input.read_sync();

    let selected_indices = loop {
        match stdin.next() {
            Some(InputEvent::Keyboard(KeyEvent::Char('\n'))) => {
                // Select the top matches
                break matches.get_matches().iter().map(|(i, ..)| *i).take(1).collect();
            }
            Some(InputEvent::Keyboard(KeyEvent::Char(c))) => {
                let mut search_term = matches.get_search_term().clone();
                search_term.push(c);
                matches.set_search_term(&search_term);
                rewrite_results(&crossterm, &matches);
                write_input(&crossterm, matches.get_search_term());
            },
            Some(InputEvent::Keyboard(KeyEvent::Backspace)) => {
                let mut search_term = matches.get_search_term().clone();
                search_term.pop();
                matches.set_search_term(&search_term);
                rewrite_results(&crossterm, &matches);
                write_input(&crossterm, matches.get_search_term());
            }
            _ => {}
        }
    };

    crossterm.cursor().show();
    selected_indices
}

fn rewrite_results<'a>(
    crossterm: &Crossterm,
    matches: &FuzzyMatcher<'a>,
) {
    let (_, height) = crossterm.terminal().terminal_size();
    crossterm.cursor().goto(0,0);
    crossterm.terminal().clear(ClearType::CurrentLine);
    write_results(
        &crossterm.terminal(),
        &matches,
        height
    );
}

fn write_input(crossterm: &Crossterm, search_term: &String) {
    let (_, height) = crossterm.terminal().terminal_size();
    crossterm.cursor().goto(0,height);
    crossterm.terminal().clear(ClearType::CurrentLine);
    crossterm.terminal().write(format!("{}>{}{} {}{}", Colored::Fg(Color::Blue), Colored::Fg(Color::White), Attribute::Bold, search_term, Attribute::Reset));
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

    for (_, s, score, indices) in matches.get_matches().iter().take(height as usize) {
        terminal.clear(ClearType::CurrentLine);
        terminal.write(format!("{}\r\n", s));
    }
}

struct FuzzyMatcher<'a> {
    selections: &'a Vec<String>,
    matches: Vec<(usize, &'a String, i64, Vec<usize>)>,
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

    pub fn get_matches(&self) -> &[(usize, &'a String, i64, Vec<usize>)] {
        &self.matches
    }

    fn update_matches(&mut self) {
        self.matches = self.selections
            .iter()
            .enumerate()
            .filter_map(|(i, s)| match fuzzy_indices(s.as_str(), &self.search_term) {
                Some((score, indices)) => Some((i, s, score, indices)),
                None => None
            })
            .collect();

        self.matches.sort_by(|(_, _, score_a, _), (_, _, score_b, _)| score_a.cmp(score_b));
    }
}

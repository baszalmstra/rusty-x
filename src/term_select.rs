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
    let mut selected_index = 0;

//    let screen = RawScreen::into_raw_mode().unwrap();

    let crossterm = Crossterm::new();
    crossterm.cursor().hide();

    let (_, term_height) = terminal().terminal_size();
    let (_, start_cursor_pos) = crossterm.cursor().pos();

    rewrite_results(&crossterm, &matches);
    write_input(&crossterm, matches.get_search_term());
    write_selected_index(&crossterm, &matches, selected_index);

    let input = crossterm.input();
    let mut stdin = input.read_sync();

    let selected_indices = loop {
        match stdin.next() {
            Some(InputEvent::Keyboard(KeyEvent::Char('\n'))) => {
                // Select the top matches
                break matches.get_matches().iter().map(|(i, ..)| *i).rev().nth(selected_index).map(|e| vec![e]).unwrap_or(Vec::new());
            }
            Some(InputEvent::Keyboard(KeyEvent::Up)) => {
                selected_index = if selected_index + 1 >= matches.get_matches().len() {
                    selected_index
                } else {
                    clear_selected_index(&crossterm, &matches, selected_index);
                    selected_index += 1;
                    write_selected_index(&crossterm, &matches, selected_index);
                    selected_index
                };
            }
            Some(InputEvent::Keyboard(KeyEvent::Down)) => {
                selected_index = if selected_index > 0 {
                    clear_selected_index(&crossterm, &matches, selected_index);
                    selected_index -= 1;
                    write_selected_index(&crossterm, &matches, selected_index);
                    selected_index
                } else {
                    selected_index
                };
            }
            Some(InputEvent::Keyboard(KeyEvent::Char(c))) => {
                let mut search_term = matches.get_search_term().clone();
                search_term.push(c);
                matches.set_search_term(&search_term);
                if selected_index >= matches.get_matches().len() { selected_index = if matches.get_matches().is_empty() { 0 } else { matches.get_matches().len() - 1 }}
                rewrite_results(&crossterm, &matches);
                write_input(&crossterm, matches.get_search_term());
                write_selected_index(&crossterm, &matches, selected_index);
            },
            Some(InputEvent::Keyboard(KeyEvent::Backspace)) => {
                let mut search_term = matches.get_search_term().clone();
                search_term.pop();
                matches.set_search_term(&search_term);
                if selected_index >= matches.get_matches().len() { selected_index = if matches.get_matches().is_empty() { 0 } else { matches.get_matches().len() - 1 }}
                rewrite_results(&crossterm, &matches);
                write_input(&crossterm, matches.get_search_term());
                write_selected_index(&crossterm, &matches, selected_index);
            }
            _ => {}
        }
    };

    crossterm.cursor().show();
    selected_indices
}

fn write_selected_index<'a>(
    crossterm: &Crossterm,
    matches: &FuzzyMatcher<'a>,
    selected_index: usize
) {
    let matches = matches.get_matches();
    if selected_index >= matches.len() {
        return;
    }

    let (_, height) = crossterm.terminal().terminal_size();
    crossterm.cursor().goto(0, height-3-selected_index as u16);
    let terminal = crossterm.terminal();
    terminal.write(format!("{}>{}", Colored::Fg(Color::Red), Attribute::Reset));
}

fn clear_selected_index<'a>(
    crossterm: &Crossterm,
    matches: &FuzzyMatcher<'a>,
    selected_index: usize
) {
    let matches = matches.get_matches();
    if selected_index >= matches.len() {
        return;
    }

    let (_, height) = crossterm.terminal().terminal_size();
    crossterm.cursor().goto(0, height-3-selected_index as u16);
    let terminal = crossterm.terminal();
    terminal.write("  ");
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
    for _ in matches.get_matches().len() ..  height as usize - 1 {
        terminal.clear(ClearType::CurrentLine);
        terminal.write(format!("\r\n"));
    }

    for (_, s, score, indices) in matches.get_matches().iter().take(height as usize - 1) {
        terminal.clear(ClearType::CurrentLine);
        terminal.write(format!("  {}\r\n", s));
    }

    terminal.clear(ClearType::CurrentLine);
    terminal.write(format!("{}/{}\r\n", matches.get_matches().len(), matches.get_selections().len()));
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

    pub fn get_selections(&self) -> &'a Vec<String> {
        self.selections
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

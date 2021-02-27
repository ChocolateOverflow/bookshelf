use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Terminal,
};

use crate::fsio::*;
use crate::module_handler::*;
use crate::shelf::*;
use crate::tui::event::{Event, Events};

struct IndexTable {
    state: TableState,
    items: Vec<Vec<String>>,
}

impl IndexTable {
    fn new(shelf: &Shelf) -> IndexTable {
        IndexTable {
            state: TableState::default(),
            items: index_to_table(
                shelf.get_index(),
                &shelf
                    .search_item(None, None, None, None, None, false, false)
                    .unwrap(),
            ),
        }
    }

    pub fn next(&mut self, count: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + count
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self, count: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - count
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub struct TUI<'lt> {
    config: &'lt HashMap<String, String>,
    shelf: &'lt Shelf,
    module_handler: &'lt ModuleHandler,
}

fn index_to_table<'a>(
    index: &HashMap<(String, String), Item>,
    targets: &HashSet<(String, String)>,
) -> Vec<Vec<String>> {
    let mut result: Vec<Vec<String>> = Vec::new();
    for (module, code) in targets {
        if let Some(item) = index.get(&(module.clone(), code.clone())) {
            let metadata = item.export();
            let title = String::from(metadata.0);
            let mut authors = String::new();
            for author in metadata.1.iter() {
                authors.push_str(author);
                authors.push_str(", ");
            }
            authors.pop();
            authors.pop();
            let mut genres = String::new();
            for genre in metadata.2.iter() {
                genres.push_str(genre);
                genres.push_str(", ");
            }
            genres.pop();
            genres.pop();
            result.push(vec![title, authors, genres, module.clone(), code.clone()]);
        }
    }
    result
}

impl<'lt> TUI<'lt> {
    pub fn new<'a>(
        config: &'lt HashMap<String, String>,
        shelf: &'lt mut Shelf,
        module_handler: &'lt ModuleHandler,
    ) -> TUI<'lt> {
        TUI {
            config,
            shelf,
            module_handler,
        }
    }

    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let stdout = std::io::stdout().into_raw_mode()?;
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let events = Events::new();
        let mut table = IndexTable::new(&self.shelf);

        let mut height: u16 = 1;
        let mut running = true;
        while running {
            terminal.draw(|frame| {
                height = frame.size().height;
                let rects = Layout::default()
                    .constraints([Constraint::Percentage(100)].as_ref())
                    .margin(0)
                    .split(frame.size());
                let style_normal = Style::default();
                let style_selected = Style::default().add_modifier(Modifier::REVERSED);
                let header_cells = ["Title", "Authors", "Genres", "Module", "Code"]
                    .iter()
                    .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
                let header = Row::new(header_cells)
                    .style(style_normal)
                    .height(1)
                    .bottom_margin(1);
                let rows = table.items.iter().map(|item| {
                    let height = item
                        .iter()
                        .map(|content| content.chars().filter(|c| *c == '\n').count())
                        .max()
                        .unwrap_or(0)
                        + 1;
                    let cells = item.iter().map(|c| Cell::from(c.clone()));
                    Row::new(cells).height(height as u16).bottom_margin(0)
                });
                let t = Table::new(rows)
                    .header(header)
                    .block(Block::default().borders(Borders::ALL))
                    .highlight_style(style_selected)
                    .widths(&[
                        Constraint::Percentage(30),
                        Constraint::Percentage(20),
                        Constraint::Percentage(40),
                        Constraint::Percentage(5),
                        Constraint::Percentage(5),
                    ]);
                frame.render_stateful_widget(t, rects[0], &mut table.state);
            })?;

            if let Event::Input(key) = events.next()? {
                match key {
                    Key::Char('q') => {
                        running = false;
                    }
                    Key::Down | Key::Char('j') => {
                        table.next(1);
                    }
                    Key::Up | Key::Char('k') => {
                        table.previous(1);
                    }
                    Key::Ctrl('d') => {
                        // move down 50%
                        table.next(usize::from(height/2));
                    }
                    Key::Ctrl('u') => {
                        // move up 50%
                        table.previous(usize::from(height/2));
                    }
                    Key::Char('f') => {
                        // filter
                    }
                    Key::Char('e') => {
                        // edit
                    }
                    Key::Char('F') => {
                        // toggle favorite
                    }
                    Key::Esc => {
                        // cencel whatever command was being entered
                    }
                    Key::Home => {
                        // go to top
                    }
                    Key::End => {
                        // go to bottom
                    }
                    Key::Char('y') => {
                        // yank item to clipboard
                    }
                    Key::Char('o') => {
                        // open item
                    }
                    Key::Char('w') => {
                        // write
                        save_shelf(
                            &self.shelf,
                            &PathBuf::from(self.config.get("index_file").unwrap()),
                        );
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn open_item(&self, module: &String, code: &String) {}
}

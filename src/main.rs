use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{CrosstermBackend, Terminal},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, List, ListState, Paragraph, Wrap},
};
use yt_cli::backend;

use std::io::{stdout, Result};

struct App {
    active_block: usize,
    search_input: String,
    footer_text: String,
    search_cursor_position: usize,
    results: Vec<backend::OrangeResult>,
    selected_item: usize,
    video_state: ListState,
    navigating_item: usize,
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            active_block: 1,
            search_input: String::new(),
            footer_text: String::new(),
            search_cursor_position: 0,
            results: backend::get_trending().unwrap(),
            selected_item: 0,
            video_state: ListState::default(),
            navigating_item: 0,
        };
        if !app.results.is_empty() {
            app.video_state.select(Some(0));
        }
        app
    }
}

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut app = App::new();

    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(8),
                        Constraint::Percentage(82),
                        Constraint::Percentage(10),
                    ]
                    .as_ref(),
                )
                .split(frame.size());

            for (i, chunk) in chunks.iter().enumerate() {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded);
                match i {
                    0 => {
                        let block = block.title("Search").style(Style::default().fg(
                            if 0 == app.active_block {
                                Color::LightGreen
                            } else {
                                Color::White
                            },
                        ));
                        let mut search_display = if app.active_block == 0 {
                            app.search_input.clone()
                        } else {
                            "What do you want to search?".to_string()
                        };
                        if app.active_block == 0
                            && app.search_cursor_position <= search_display.len()
                            && search_display.is_char_boundary(app.search_cursor_position)
                        {
                            search_display.insert(app.search_cursor_position, '|');
                        }
                        let paragraph = Paragraph::new(search_display.as_ref() as &str)
                            .block(block)
                            .wrap(Wrap { trim: true });

                        frame.render_widget(paragraph, *chunk);
                    }
                    1 => {
                        if app.results.len() > 0 {
                            let block = block
                                .border_style(Style::default().fg(Color::Magenta))
                                .title(format!(
                                    "Videos [{}/{}]",
                                    app.navigating_item + 1,
                                    app.results.len()
                                ));
                            let items: Vec<String> = app
                                .results
                                .iter()
                                .enumerate()
                                .map(|(i, r)| {
                                    format!("{}. {} ===>[󰔛 {}]", i + 1, r.title, r.duration)
                                })
                                .collect::<Vec<String>>();

                            let list = List::new(items).block(block).highlight_style(
                                Style::default().bg(Color::Green).fg(Color::Black),
                            );
                            frame.render_stateful_widget(list, *chunk, &mut app.video_state);
                        } else {
                            let block = block
                                .border_style(Style::default().fg(Color::Magenta))
                                .title("No Videos");
                            let paragraph = Paragraph::new("No results found")
                                .block(block)
                                .wrap(Wrap { trim: true });
                            frame.render_widget(paragraph, *chunk);
                        }
                    }
                    2 => {
                        let block = block.title("Status").style(Style::default().fg(
                            if 2 == app.active_block {
                                Color::Magenta
                            } else {
                                Color::White
                            },
                        ));
                        let footer_text = if app.footer_text.is_empty() {
                            "Press / to search".to_string()
                        } else {
                            app.footer_text.clone()
                        };
                        let paragraph = Paragraph::new(footer_text.as_ref() as &str)
                            .block(block)
                            .wrap(Wrap { trim: true });
                        frame.render_widget(paragraph, *chunk);
                    }
                    _ => {}
                };
            }
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('/') => {
                        app.active_block = 0;
                        app.search_cursor_position = app.search_input.len();
                    }
                    KeyCode::Char(c) if app.active_block == 0 => {
                        app.search_input.insert(app.search_cursor_position, c);
                        app.search_cursor_position += 1;
                    }
                    KeyCode::Backspace
                        if app.active_block == 0 && app.search_cursor_position > 0 =>
                    {
                        app.search_input.remove(app.search_cursor_position - 1);
                        app.search_cursor_position -= 1;
                    }
                    KeyCode::Left if app.active_block == 0 && app.search_cursor_position > 0 => {
                        app.search_cursor_position -= 1;
                    }
                    KeyCode::Right
                        if app.active_block == 0
                            && app.search_cursor_position < app.search_input.len() =>
                    {
                        app.search_cursor_position += 1;
                    }
                    KeyCode::Up if app.active_block == 1 => {
                        if app.results.len() > 0 && app.navigating_item > 0 {
                            app.video_state.select(Some(app.navigating_item - 1));
                            app.navigating_item -= 1;
                        }
                    }
                    KeyCode::Down if app.active_block == 1 => {
                        if app.results.len() > 0 && app.navigating_item < app.results.len() - 1 {
                            app.video_state.select(Some(app.navigating_item + 1));
                            app.navigating_item += 1;
                        }
                    }
                    KeyCode::Enter => match app.active_block {
                        0 => {
                            app.results = backend::get_search(&app.search_input).unwrap();
                            app.active_block = 1;
                            app.selected_item = 0;
                            app.navigating_item = 0;
                            app.video_state.select(Some(0));
                            app.footer_text = format!("Search results for: {}", app.search_input);
                            app.search_input.clear();
                        }
                        1 => {
                            app.selected_item = app.video_state.selected().unwrap();
                            let selection = app.results[app.selected_item].url.clone();
                            let title = app.results[app.selected_item].title.clone();
                            backend::play_selection(&selection, &title);
                            app.footer_text = format!("Playing: {} [{}]", title, selection);
                        }
                        _ => {}
                    },
                    KeyCode::Tab => {
                        app.active_block = (app.active_block + 1) % 3;
                        if app.active_block == 1 {
                            app.selected_item = 0;
                            app.video_state.select(Some(0));
                        }
                    }
                    KeyCode::Char('q') => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

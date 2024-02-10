use clap::{command, Arg};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::{CrosstermBackend, Terminal},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use std::{
    io::{stdout, Result},
    ops::Not,
};
use yt_cli::backend;

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
    let matches = command!()
        .about("A cli to search and play videos from piped API")
        .version("1.6.0")
        .arg(
            Arg::new("search")
                .short('s')
                .long("search")
                .help_heading(Some("Search for a video")),
        )
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .help_heading("Play a video by url"),
        )
        .get_matches();

    let url_is_not_empty = matches.get_one::<String>("url").is_some();
    let search_is_empty = matches.get_one::<String>("search").is_some().not();

    let mut app = App::new();

    if url_is_not_empty && search_is_empty.not() {
        println!("Please provide either a search query or a video url, not both.");
        return Ok(());
    }

    if url_is_not_empty {
        let url = matches.get_one::<String>("url").unwrap();
        println!("Playing from: {}", url);
        backend::play_url(&url);
        return Ok(());
    }

    if !search_is_empty {
        let search = matches
            .get_one::<String>("search")
            .map(|s| s.to_string())
            .unwrap();
        app.results = backend::get_search(&search).unwrap();
    }

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(8),
                        Constraint::Percentage(84),
                        Constraint::Percentage(8),
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
                            let items: Vec<ListItem> = app
                                .results
                                .iter()
                                .enumerate()
                                .map(|(i, r)| {
                                    let index = Span::styled(
                                        format!("{}. ", i + 1),
                                        Style::default().fg(Color::Rgb(198, 160, 246)),
                                    );
                                    let title = Span::styled(
                                        format!("{}", r.title),
                                        Style::default().fg(Color::White).bold(),
                                    );
                                    let verified = Span::styled(
                                        format!(" [ "),
                                        Style::default()
                                            .fg(if r.is_verified {
                                                Color::Rgb(166, 218, 149)
                                            } else {
                                                Color::Rgb(237, 135, 150)
                                            })
                                            .bold(),
                                    );
                                    let uploader = Span::styled(
                                        format!("{}]", r.uploader),
                                        Style::default().fg(Color::Rgb(245, 169, 127)).bold(),
                                    );
                                    let duration = Span::styled(
                                        format!(" [󰔛 {}]", r.duration),
                                        Style::default().fg(Color::Rgb(240, 198, 198)),
                                    );
                                    ListItem::new(Line::from(vec![
                                        index, title, duration, verified, uploader,
                                    ]))
                                })
                                .collect::<Vec<ListItem>>();

                            let list = List::new(items).block(block).highlight_style(
                                Style::default().bg(Color::White).fg(Color::Black),
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
                            Span::styled(
                                format!("Press / to search | Press q to quit"),
                                Style::default().fg(Color::Red),
                            )
                        } else {
                            Span::styled(
                                format!("{}", app.footer_text.clone()),
                                Style::default().fg(Color::Green),
                            )
                        };
                        let paragraph = Paragraph::new(footer_text)
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
                            backend::play_selection(&selection);
                            app.footer_text = format!("Playing: {}", title);
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

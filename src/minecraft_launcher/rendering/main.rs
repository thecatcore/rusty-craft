use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen, event};
use tui::{backend::TermionBackend, Terminal, Frame};
use std::time::Duration;
use crate::minecraft_launcher::rendering::events::{Events, Event};
use crate::minecraft_launcher::manifest::main::MinVersion;
use tui::layout::{Layout, Constraint, Rect, Direction};
use tui::widgets::{Tabs, Borders, Block, Row, Table, Cell, List, ListItem};
use tui::text::{Spans, Span};
use tui::style::{Style, Color, Modifier};
use tui::backend::Backend;
use chrono::Local;
use crate::minecraft_launcher::rendering::utils::StatefulList;

pub fn main(versions: &Vec<(MinVersion, bool)>) -> Result<(), Box<dyn Error>> {

    let events = Events::new();

    let items: Vec<Span> = versions
        .iter()
        .map(|v| {
            // let cells = vec![
            //     Cell::from(Span::raw(format!("{}", v.0.id))),
            //     Cell::from(Span::raw(format!("{}", v.0._type.to_string()))),
            //     Cell::from(Span::raw(format!("{}", match v.1 {
            //         true => {"Yes"}
            //         false => {"No"}
            //     }))),
            //     Cell::from(Span::raw(format!("{:?}", v.0.release_time))),
            // ];
            // Row::new(cells);
            Span::raw(format!("{} {} {} {:?}", v.0.id, v.0._type.to_string(), match v.1 {
                true => {"Yes"}
                false => {"No"}
            }, v.0.release_time))
        })
        .collect();

    let mut list = StatefulList::with_items(items);

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;

    loop {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.size());

            let mut ve: Vec<Spans> = Vec::new();
            ve.append(&mut vec![Spans::from("Test")]);

            let tabs = Tabs::new(ve)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Yellow))
                .select(0);
            f.render_widget(tabs, chunks[0]);
            draw_tab(f, chunks[1], &mut list)
        })?;

        match events.next()? {
            Event::Input(key) => match key {
                // Key::Backspace => {}
                Key::Left => {}
                Key::Right => {}
                Key::Up => {
                    list.previous();
                }
                Key::Down => {
                    list.next();
                }
                // Key::Home => {}
                // Key::End => {}
                // Key::PageUp => {}
                // Key::PageDown => {}
                // Key::BackTab => {}
                // Key::Delete => {}
                // Key::Insert => {}
                // Key::F(_) => {}
                Key::Char(c) => {}
                // Key::Alt(_) => {}
                // Key::Ctrl(_) => {}
                // Key::Null => {}
                Key::Esc => {
                    break
                }
                // Key::__IsNotComplete => {}
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}

fn draw_tab<B: Backend>(f: &mut Frame<B>, area: Rect, versions: &mut StatefulList<Span>) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 1)])
        .split(area);

    let version_list: Vec<ListItem> = versions
        .items
        .iter()
        .map(|i| ListItem::new(vec![Spans::from(i.clone())]))
        .collect();

    let table = List::new(version_list)
        .block(Block::default().borders(Borders::ALL).title("Version List"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    // let table = Table::new(items)
    //     .block(Block::default().title("Versions").borders(Borders::ALL))
    //     .header(Row::new(vec![
    //         Cell::from(Span::raw("Name")),
    //         Cell::from(Span::raw("Type")),
    //         Cell::from(Span::raw("Installed")),
    //         Cell::from(Span::raw("Release Date"))
    //     ]))
    //     .widths(&[
    //         Constraint::Ratio(5, 12),
    //         Constraint::Ratio(2, 12),
    //         Constraint::Ratio(2, 12),
    //         Constraint::Ratio(3, 12),
    //     ])
    //     .highlight_style(Style::default().add_modifier(Modifier::BOLD))
    //     .highlight_symbol("> ");

    f.render_stateful_widget(table, chunks[0], &mut versions.state);
}
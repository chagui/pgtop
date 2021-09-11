use std::io;
use std::iter;
use std::mem;

use termion::event::Key;
use termion::raw::IntoRawMode;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::Color;
use tui::text::Span;
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame, Terminal,
};

use crate::db::{get_activities, get_system_info, PGStatActivity, PGSystemInfo};
use crate::event::Event;
use crate::{CliResult, Context};

const TITLE_STYLE: Style = Style {
    fg: Some(Color::White),
    bg: None,
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
const SELECTED_STYLE: Style = Style {
    fg: None,
    bg: None,
    add_modifier: Modifier::REVERSED,
    sub_modifier: Modifier::empty(),
};

impl<'a> From<PGStatActivity> for Row<'a> {
    fn from(activity: PGStatActivity) -> Row<'a> {
        let mut state_cell_style = Style::default();
        if activity.state == "active" {
            state_cell_style = state_cell_style
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD);
        }

        // todo: only show part of the query that fits
        // let fmt_query: String = activity.query.chars().take(50).collect();

        let cells = vec![
            Cell::from(activity.datname),
            Cell::from(activity.pid.to_string()),
            Cell::from(activity.usename),
            Cell::from(activity.client_addr),
            Cell::from(activity.client_port.to_string()),
            // Cell::from(activity.xact_start.unwrap_or(String::from(""))),
            Cell::from(activity.backend_duration),
            Cell::from(activity.query_duration),
            Cell::from(activity.state).style(state_cell_style),
            Cell::from(activity.query),
        ];

        let height = 1u16;
        Row::new(cells).height(height)
    }
}

impl<'a> From<&PGSystemInfo> for Row<'a> {
    fn from(system_info: &PGSystemInfo) -> Row<'a> {
        let cells = vec![
            // todo spans with different styles
            Cell::from(format!("version: {}", system_info.version)),
            Cell::from(format!("uptime: {}", system_info.uptime)),
            Cell::from(format!("active connections: {}", system_info.nb_of_conn)),
        ];

        let height = 1u16;
        Row::new(cells).height(height)
    }
}

struct StatActivityView {
    state: TableState,
    activities: Vec<PGStatActivity>,
}

impl StatActivityView {
    fn new() -> StatActivityView {
        StatActivityView {
            state: TableState::default(),
            activities: vec![],
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.activities.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.activities.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn get_header_row<'a>() -> Row<'a> {
        // style
        let header_cols = [
            "database",
            "pid",
            "user",
            "client_addr",
            "client_port",
            // todo: xact only set for transaction
            // "xact start",
            "backend duration",
            "query duration",
            "state",
            "query",
        ];
        let header_cells = header_cols
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
        Row::new(header_cells.clone()).height(1)
    }
}

fn draw_system_info<B>(frame: &mut Frame<B>, system_info: &PGSystemInfo, layout_chunk: Rect)
where
    B: Backend,
{
    let rows = iter::once(system_info).map(Row::from);
    let system_info_table = Table::new(rows)
        .widths(&[
            Constraint::Percentage(60),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(String::from("System"), TITLE_STYLE)),
        );
    frame.render_widget(system_info_table, layout_chunk);
}

fn draw_activities<B>(
    frame: &mut Frame<B>,
    activities: Vec<PGStatActivity>,
    state: &mut TableState,
    layout_chunk: Rect,
) where
    B: Backend,
{
    let header = StatActivityView::get_header_row();
    let rows = activities.into_iter().map(Row::from);
    let stat_activity_table = Table::new(rows)
        .header(header)
        .widths(&[
            Constraint::Min(10),
            Constraint::Length(5),
            Constraint::Min(10),
            Constraint::Min(10),
            Constraint::Length(11),
            // Constraint::Min(30),
            Constraint::Min(30),
            Constraint::Min(30),
            Constraint::Min(10),
            Constraint::Min(50),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(String::from("Activities"), TITLE_STYLE)),
        )
        .highlight_style(SELECTED_STYLE);
    frame.render_stateful_widget(stat_activity_table, layout_chunk, state);
}

pub async fn start_ui(ctx: Context) -> CliResult<()> {
    // data initial fetch (refreshed at each tick)
    let mut stat_activity_view = StatActivityView::new();
    let mut system_info = get_system_info(&ctx.client).await?;

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;
    terminal.clear()?;
    loop {
        terminal.draw(|mut frame| {
            // UI layout, each rectangle is a section
            let main_layout = Layout::default()
                .constraints([Constraint::Length(3), Constraint::Min(20)].as_ref())
                .margin(1)
                .split(frame.size());

            draw_system_info(&mut frame, &system_info, main_layout[0]);
            draw_activities(
                &mut frame,
                mem::take(&mut stat_activity_view.activities),
                &mut stat_activity_view.state,
                main_layout[1],
            );
        })?;

        match ctx.events.next()? {
            Event::Input(key) => match key {
                Key::Char('q') | Key::Ctrl('c') => {
                    break;
                }
                Key::Down => {
                    stat_activity_view.next();
                }
                Key::Up => {
                    stat_activity_view.previous();
                }
                Key::Ctrl('r') => {
                    stat_activity_view.activities = get_activities(&ctx.client).await?;
                    system_info = get_system_info(&ctx.client).await?;
                }
                _ => {}
            },
            Event::Tick => {
                stat_activity_view.activities = get_activities(&ctx.client).await?;
                system_info = get_system_info(&ctx.client).await?;
            }
        }
    }
    terminal.clear()?;
    Ok(())
}

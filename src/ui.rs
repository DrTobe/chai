use std::{io, time};
use tui::backend::CrosstermBackend;
use tui::Terminal;
//use tui::widgets::{Widget, Block, Borders};
//use tui::layout::{Layout, Constraint, Direction, Rect};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::layout::{Constraint, Direction, Layout, Rect};

pub struct CTui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl CTui {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(CTui { terminal })
    }

    pub fn terminal(&mut self) -> &mut Terminal<CrosstermBackend<io::Stdout>> {
        &mut self.terminal
    }
}

impl Drop for CTui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

pub fn show_abortable(_ctui: &mut CTui, duration: time::Duration) -> bool {
    let start = time::Instant::now();
    loop {
        let remaining = duration
            .checked_sub(start.elapsed())
            .unwrap_or(crate::ms(0));
        if event::poll(remaining).unwrap() {
            if let Event::Key(key_event) = event::read().unwrap() {
                if key_event.code == KeyCode::Char('q') {
                    return true;
                }
            }
        }
        if start.elapsed() >= duration {
            break;
        }
    }
    false
}

/*
pub fn reshow_board(siv: &mut Cursive, board: BoardState, duration: time::Duration) -> bool {
    siv.clear();
    siv.add_layer(
        Dialog::new()
            .title("ChaiChess")
            .content(LinearLayout::horizontal().child(Panel::new(BoardView::new(board)))),
    );
    show_abortable(siv, duration)
}

pub fn show_board(board: BoardState, duration: time::Duration) -> bool {
    let mut siv = cursive::default();
    reshow_board(&mut siv, board, duration)
}
*/

pub fn layout_vertical<C>(r: Rect, constraints: C) -> Vec<Rect>
where
    C: Into<Vec<Constraint>>,
{
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(r)
}

pub fn layout_horizontal<C>(r: Rect, constraints: C) -> Vec<Rect>
where
    C: Into<Vec<Constraint>>,
{
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(r)
}

pub fn center(outer: Rect, desired_width: u16, desired_height: u16) -> Rect {
    let naive = Rect {
        x: u16::saturating_sub(outer.x + outer.width / 2, desired_width / 2),
        y: u16::saturating_sub(outer.y + outer.height / 2, desired_height / 2),
        width: desired_width,
        height: desired_height,
    };
    outer.intersection(naive)
}

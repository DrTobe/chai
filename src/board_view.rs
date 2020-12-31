use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::Widget;

use std::collections::HashSet;

use crate::game::*;

pub struct BoardView {
    pub board: BoardState,
    pub highlights: HashSet<usize>,
}

impl BoardView {
    pub fn new(board: BoardState) -> Self {
        Self {
            board,
            highlights: HashSet::new(),
        }
    }
    pub fn newh(board: BoardState, highlights: HashSet<usize>) -> Self {
        Self { board, highlights }
    }

    pub fn get_field_index_from_pos(area: Rect, x: u16, y: u16) -> Option<usize> {
        if x >= area.x && x < area.x + area.width && y >= area.y && y < area.y + area.height {
            let x = x - area.x;
            let y = y - area.y;
            let row = y as usize;
            let col = (x / 3) as usize;
            let field = (7 - row) * 8 + col;
            Some(field)
        } else {
            None
        }
    }
}

impl Widget for BoardView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for i in 0..64 {
            let row = 7 - (i / 8);
            let col = i % 8;
            let x = (col * 3) as u16;
            let y = row as u16;
            if x >= area.width || y >= area.height {
                continue;
            }

            let text = if let Some(piece_and_player) = self.board.fields[i] {
                match piece_and_player {
                    (PieceType::InitKing, Player::Black) => " ♚ ",
                    (PieceType::King, Player::Black) => " ♚ ",
                    (PieceType::Queen, Player::Black) => " ♛ ",
                    (PieceType::InitRook, Player::Black) => " ♜ ",
                    (PieceType::Rook, Player::Black) => " ♜ ",
                    (PieceType::Bishop, Player::Black) => " ♝ ",
                    (PieceType::Knight, Player::Black) => " ♞ ",
                    (PieceType::InitPawn, Player::Black) => " ♟ ",
                    (PieceType::Pawn, Player::Black) => " ♟ ",
                    (PieceType::InitKing, Player::White) => " ♔ ",
                    (PieceType::King, Player::White) => " ♔ ",
                    (PieceType::Queen, Player::White) => " ♕ ",
                    (PieceType::InitRook, Player::White) => " ♖ ",
                    (PieceType::Rook, Player::White) => " ♖ ",
                    (PieceType::Bishop, Player::White) => " ♗ ",
                    (PieceType::Knight, Player::White) => " ♘ ",
                    (PieceType::InitPawn, Player::White) => " ♙ ",
                    (PieceType::Pawn, Player::White) => " ♙ ",
                }
            } else {
                "   "
            };

            let base = Style::default().fg(Color::Black);
            let color_style = match (is_dark_field(row, col), self.highlights.contains(&i)) {
                (true, false) => base.bg(Color::Rgb(100, 100, 100)),
                (false, false) => base.bg(Color::Rgb(250, 250, 250)),
                (true, true) => base.bg(Color::Rgb(100, 100, 0)),
                (false, true) => base.bg(Color::Rgb(200, 200, 0)),
            };

            buf.set_string(area.left() + x, area.top() + y, text, color_style);
        }
    }
}

fn is_dark_field(row: usize, col: usize) -> bool {
    (row + col + 1) % 2 == 0
}

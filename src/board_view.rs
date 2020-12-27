//use cursive::direction::Direction;
//use cursive::event::{Event, EventResult, MouseButton, MouseEvent};
use cursive::theme::{BaseColor, Color, ColorStyle};
use cursive::views::{/*Button,*/ Dialog, LinearLayout, Panel /*SelectView*/};
use cursive::Cursive;
use cursive::Printer;
use cursive::Vec2;

use std::sync::atomic;
use std::thread;
use std::time;

use crate::game::*;

static CANCELED: atomic::AtomicBool = atomic::AtomicBool::new(false);

pub fn show_abortable(siv: &mut Cursive, duration: time::Duration) -> bool {
    CANCELED.store(false, atomic::Ordering::Relaxed);
    siv.set_global_callback(cursive::event::Event::Char('q'), |_| {
        CANCELED.store(true, atomic::Ordering::Relaxed)
    });
    siv.refresh();
    let mut passed = time::Duration::new(0, 0);
    while passed < duration {
        thread::sleep(crate::ms(50));
        passed += crate::ms(50);
        loop {
            if siv.step() == false {
                break;
            }
        }
        if CANCELED.load(atomic::Ordering::Relaxed) == true {
            return true;
        }
    }
    false
}

pub fn reshow_board(siv: &mut Cursive, board: BoardState, duration: time::Duration) -> bool {
    siv.clear();
    siv.add_layer(
        Dialog::new()
            .title("ChaiChess")
            .content(LinearLayout::horizontal().child(Panel::new(BoardView { board }))),
    );
    show_abortable(siv, duration)
}

pub fn show_board(board: BoardState, duration: time::Duration) -> bool {
    let mut siv = cursive::default();
    reshow_board(&mut siv, board, duration)
}

pub struct BoardView {
    pub board: BoardState,
}

impl cursive::view::View for BoardView {
    fn draw(&self, printer: &Printer) {
        for i in 0..64 {
            let row = 7 - (i / 8);
            let col = i % 8;
            let x = col * 3;
            let y = row;

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

            let color_style = if (row + col + 1) % 2 == 0 {
                ColorStyle::new(Color::Dark(BaseColor::Black), Color::RgbLowRes(2, 2, 2))
            } else {
                ColorStyle::new(Color::Dark(BaseColor::Black), Color::RgbLowRes(5, 5, 5))
            };

            printer.with_color(color_style, |printer| printer.print((x, y), text));
        }
    }

    fn required_size(&mut self, _constraint: Vec2) -> Vec2 {
        Vec2::new(24, 8)
    }
}

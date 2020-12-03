use cursive::direction::Direction;
use cursive::event::{Event, EventResult, MouseButton, MouseEvent};
use cursive::theme::{BaseColor, Color, ColorStyle};
use cursive::views::{Button, Dialog, LinearLayout, Panel, SelectView};
use cursive::Cursive;
use cursive::Printer;
use cursive::Vec2;

fn main() {
    let mut siv = cursive::default();

    siv.add_layer(
        Dialog::new()
            .title("ChaiChess")
            .content(
                LinearLayout::horizontal()
                    .child(Panel::new(BoardView::new())),
            )
            .button("Quit game", |s| {
                s.quit();
            }),
    );

    siv.run();
}

#[derive(Clone, Copy, PartialEq)]
enum PieceType {
    King,
    Queen,
    InitRook,
    Rook,
    Bishop,
    Knight,
    Pawn
}

#[derive(Clone, Copy, PartialEq)]
enum Player {
    Black,
    White
}

struct BoardView {
    // Actual board, unknown to the player.
    //board: game::Board,

    board_state: [Option<(PieceType, Player)>; 64],
}

impl BoardView {
    pub fn new() -> Self {
        let mut board = [None; 64];
        let init_row = [
            PieceType::InitRook,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Queen,
            PieceType::King,
            PieceType::Bishop,
            PieceType::Knight,
            PieceType::InitRook
        ];
        for i in 0..8 {
            board[i] = Some((init_row[i], Player::White));
            board[1*8+i] = Some((PieceType::Pawn, Player::White));
            board[7*8+i] = Some((init_row[i], Player::Black));
            board[6*8+i] = Some((PieceType::Pawn, Player::Black));
        }
        BoardView {
            board_state: board
        }
    }
}

impl cursive::view::View for BoardView {
    fn draw(&self, printer: &Printer) {
        for i in 0..64 {
            let row = 7 - (i/8);
            let col = i%8;
            let x = col * 3;
            let y = row;

            let text = if let Some(piece_and_player) = self.board_state[i] {
                match piece_and_player {
                    (PieceType::King, Player::Black) => " ♚ ",
                    (PieceType::Queen, Player::Black) => " ♛ ",
                    (PieceType::InitRook, Player::Black) => " ♜ ",
                    (PieceType::Rook, Player::Black) => " ♜ ",
                    (PieceType::Bishop, Player::Black) => " ♝ ",
                    (PieceType::Knight, Player::Black) => " ♞ ",
                    (PieceType::Pawn, Player::Black) => " ♟ ",
                    (PieceType::King, Player::White) => " ♔ ",
                    (PieceType::Queen, Player::White) => " ♕ ",
                    (PieceType::InitRook, Player::White) => " ♖ ",
                    (PieceType::Rook, Player::White) => " ♖ ",
                    (PieceType::Bishop, Player::White) => " ♗ ",
                    (PieceType::Knight, Player::White) => " ♘ ",
                    (PieceType::Pawn, Player::White) => " ♙ ",
                }
            } else {
                "   "
            };

            let color_style = if (row+col+1) % 2 == 0 {
                ColorStyle::new(Color::Dark(BaseColor::Black), Color::RgbLowRes(2, 2, 2))
            } else {
                ColorStyle::new(Color::Dark(BaseColor::Black), Color::RgbLowRes(5, 5, 5))
            };

            printer.with_color(
                color_style,
                |printer| printer.print((x, y), text),
            );
        }
    }

    fn required_size(&mut self, _constraint: Vec2) -> Vec2 {
        Vec2::new(24,8)
    }
}

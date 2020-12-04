use std::time;

use rand::seq::SliceRandom;

mod board_view;
mod game;

fn main() {
    let board = game::BoardState::new();

    /*
    board_view::show_board(board, ms(300));
    board.fields[25] = Some((game::PieceType::Bishop, game::Player::White));
    board_view::show_board(board, ms(300));
    */

    let mut game = game::GameState { board, ply: 0 };

    loop {
        let new_state = match game
            .get_pseudo_legal_moves()
            .choose(&mut rand::thread_rng()) {
                Some(s) => *s,
                None => break,
            };
        game = new_state;
        if board_view::show_board(game.board, ms(100)) == true
        //|| board_view::show_board(game.board, ms(500)) == true
        {
            break;
        }
    }
}

pub fn ms(millis: u64) -> time::Duration {
    time::Duration::from_millis(millis)
}

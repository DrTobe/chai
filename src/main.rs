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

    loop {
        let mut game = game::GameState { board, ply: 0 };

        let mut siv = cursive::default();
        loop {
            let new_state = match game.get_legal_moves().choose(&mut rand::thread_rng()) {
                Some(s) => *s,
                None => break,
            };
            if board_view::reshow_board(&mut siv, new_state.board, ms(500)) == true
            //|| board_view::reshow_board(&mut siv, game.board, ms(100)) == true
            //|| board_view::reshow_board(&mut siv, new_state.board, ms(100)) == true
            {
                return;
            }
            game = new_state;
        }
        drop(siv);
        println!("===========");
        println!("FINAL STATE");
        println!("===========");
        println!("{:?}", game);
    }
}

pub fn ms(millis: u64) -> time::Duration {
    time::Duration::from_millis(millis)
}

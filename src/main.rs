use std::time;

use rand::seq::SliceRandom;

mod board_view;
mod game;
mod minimax;

fn main() {
    let board = game::BoardState::new();

    /*
    board_view::show_board(board, ms(300));
    board.fields[25] = Some((game::PieceType::Bishop, game::Player::White));
    board_view::show_board(board, ms(300));
    */

    loop {
        let mut game = game::GameState {
            board,
            ply: 0,
            fifty_move_rule_last_event: 0,
        };

        let mut siv = cursive::default();
        let game_result = loop {
            if game.fifty_move_rule_draw() {
                break "DRAW: 75 moves without event.".to_string();
            }
            //let new_state = match game.get_legal_moves().choose(&mut rand::thread_rng()) {
            //Some(s) => *s,
            //None => {
            let new_states = game.get_legal_moves();
            if new_states.len() == 0 {
                if game.board.king_in_check(game.turn()) {
                    break format!("{:?} WIN.", game.turn().opponent());
                } else {
                    break format!("DRAW: {:?} can not move.", game.turn());
                }
            }

            let mut values = vec![];
            for new_state in &new_states {
                values.push(minimax::minimax(
                    *new_state,
                    2,
                    &minimax::weighted_piececount,
                ));
            }
            let best = match game.turn() {
                game::Player::White => values.iter().max().unwrap(),
                game::Player::Black => values.iter().min().unwrap(),
            };
            let best_states: Vec<(&i64, game::GameState)> = values
                .iter()
                .zip(new_states)
                .filter(|(val, _)| *val == best)
                .collect();
            let new_state = best_states.choose(&mut rand::thread_rng()).unwrap().1;

            if board_view::reshow_board(&mut siv, new_state.board, ms(500)) == true
            //|| board_view::reshow_board(&mut siv, game.board, ms(100)) == true
            //|| board_view::reshow_board(&mut siv, new_state.board, ms(100)) == true
            {
                return;
            }
            game = new_state;
        };
        drop(siv);
        println!("===========");
        println!("FINAL STATE");
        println!("===========");
        println!("{:?}", game);
        println!("===========");
        println!("--RESULT---");
        println!("{}", game_result);
    }
}

pub fn ms(millis: u64) -> time::Duration {
    time::Duration::from_millis(millis)
}

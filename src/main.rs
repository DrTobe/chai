use cursive::views::{LinearLayout, TextView};
use rand::seq::SliceRandom;
use std::time;

pub mod board_view;
pub mod game;
pub mod minimax;

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
            let minimax_res = minimax::minimax(game, 3, &minimax::weighted_piececount);
            let new_states = minimax_res.1;
            if new_states.len() > 0 {
                let new_state = choose(new_states).unwrap();
                siv.clear();
                siv.add_layer(
                    LinearLayout::vertical()
                        .child(board_view::BoardView {
                            board: new_state.board,
                        })
                        .child(TextView::new(format!("Num Nodes: {}", minimax_res.2))),
                );
                //if board_view::reshow_board(&mut siv, new_state.board, ms(500)) == true {
                if board_view::show_abortable(&mut siv, ms(500)) == true {
                    return;
                }
                game = new_state;
            } else {
                if game.fifty_move_rule_draw() {
                    break "DRAW: 75 moves without event.".to_string();
                }
                let new_states = game.get_legal_moves();
                if new_states.len() == 0 {
                    if game.board.king_in_check(game.turn()) {
                        break format!("{:?} WIN.", game.turn().opponent());
                    } else {
                        break format!("DRAW: {:?} can not move.", game.turn());
                    }
                }
            }
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

pub fn choose<T: Copy>(s: Vec<T>) -> Option<T> {
    s.choose(&mut rand::thread_rng()).map(|x| *x)
}

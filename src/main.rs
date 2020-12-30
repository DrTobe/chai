use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::time;

use tui::layout::Constraint;
use tui::widgets::Paragraph;

pub mod board_view;
pub mod game;
pub mod minimax;
pub mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //tui_test()?;
    autoplay()?;

    Ok(())
}

fn tui_test() -> Result<(), Box<dyn std::error::Error>> {
    let mut ctui = ui::CTui::new()?;

    let board = game::BoardState::new();

    ctui.terminal().draw(|f| {
        let chunks = ui::layout_vertical(
            f.size(),
            [
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        );
        let board = board_view::BoardView::new(board);
        f.render_widget(board, ui::center(chunks[1], 24, 8));
    })?;
    Ok(())
}

pub fn autoplay() -> Result<(), Box<dyn std::error::Error>> {
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

        let mut ctui = ui::CTui::new()?;
        let game_result = loop {
            let minimax_res = minimax::minimax(game, 3, &minimax::weighted_piececount);
            let alphabeta_res = minimax::alphabeta_init(game, 3, &minimax::weighted_piececount);
            assert_eq!(alphabeta_res.0, minimax_res.0);
            let minimax_num_actions = minimax_res.1.len();
            let alphabeta_num_actions = alphabeta_res.1.len();
            //assert_eq!(alphabeta_num_actions, minimax_num_actions);
            //assert_eq!(minimax_res.1, alphabeta_res.1);
            let new_states = minimax_res.1;
            let minimax_nodes = minimax_res.2;
            if new_states.len() > 0 {
                game = choose(new_states).unwrap();
                ctui.terminal().draw(|f| {
                    let size = ui::center(f.size(), 30, 12);
                    let chunks = ui::layout_vertical(
                        size,
                        [
                            Constraint::Length(8),
                            Constraint::Length(1),
                            Constraint::Length(1),
                            Constraint::Length(1),
                            Constraint::Length(1),
                        ]
                        .as_ref(),
                    );
                    let board = board_view::BoardView::new(game.board);
                    let (t1, t2, t3, t4) = (
                        format!("Minimax Nodes: {}", minimax_nodes),
                        format!("Alphabeta Nodes: {}", alphabeta_res.2),
                        format!("Minimax num actions: {}", minimax_num_actions),
                        format!("Alphabeta num actions: {}", alphabeta_num_actions),
                    );
                    f.render_widget(board, chunks[0]);
                    f.render_widget(Paragraph::new(t1), chunks[1]);
                    f.render_widget(Paragraph::new(t2), chunks[2]);
                    f.render_widget(Paragraph::new(t3), chunks[3]);
                    f.render_widget(Paragraph::new(t4), chunks[4]);
                })?;
                if ui::show_abortable(&mut ctui, ms(0)) == true {
                    return Ok(());
                }
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
                } else {
                    panic!("No possible actions and no final game state reached.");
                }
            }
        };
        let game_result2 = game_result.clone();
        ctui.terminal().draw(|f| {
            let size = ui::center(f.size(), 30, 9);
            let chunks = ui::layout_vertical(
                size,
                [
                    Constraint::Length(8),
                    Constraint::Length(1),
                ]
                .as_ref(),
            );
            let board = board_view::BoardView::new(game.board);
            f.render_widget(board, chunks[0]);
            f.render_widget(Paragraph::new(game_result2), chunks[1]);
        })?;
        ui::show_abortable(&mut ctui, ms(5000));
        drop(ctui);
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

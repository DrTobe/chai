use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::time;

use crossterm::event;
use tui::layout::Constraint;
use tui::widgets::Paragraph;

pub mod board_view;
pub mod game;
pub mod minimax;
pub mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    serde_test();
    //tui_test()?;
    //autoplay()?;
    //play_as(game::Player::White)?;

    Ok(())
}

pub fn serde_test() {
    let board = game::BoardState::new();
    let game = game::GameState {
        board,
        ply: 0,
        fifty_move_rule_last_event: 0
    };
    let json_board = serde_json::to_string(&game).unwrap();
    println!("{}", json_board);
    // remove 11 .. 31 (first field)
    //let json_board = format!("{}{}", &json_board[0..11], &json_board[32..]);
    //println!("{}", json_board);
    let deserd: game::GameState = serde_json::from_str(&json_board).unwrap();
    println!("{:?}", deserd);
}

pub fn tui_test() -> Result<(), Box<dyn std::error::Error>> {
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
    //std::thread::sleep(ms(10000));
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
                [Constraint::Length(8), Constraint::Length(1)].as_ref(),
            );
            let board = board_view::BoardView::new(game.board);
            f.render_widget(board, chunks[0]);
            f.render_widget(Paragraph::new(game_result2), chunks[1]);
        })?;
        ui::show_abortable(&mut ctui, ms(30000));
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

pub fn play_as(human: game::Player) -> Result<(), Box<dyn std::error::Error>> {
    let board = game::BoardState::new();

    let mut game = game::GameState {
        board,
        ply: 0,
        fifty_move_rule_last_event: 0,
    };

    let mut ctui = ui::CTui::new()?;
    let game_result = loop {
        if game.get_legal_moves().len() > 0 {
            // TODO fifty move rules draw??
            game = if game.turn() == human {
                match get_new_state_from_user(&mut ctui, game)? {
                    Some(g) => g,
                    None => return Ok(()),
                }
            } else {
                ctui.terminal().draw(|f| {
                    let size = ui::center(f.size(), 24, 8);
                    let chunks = ui::layout_vertical(size, [Constraint::Length(8)].as_ref());
                    let board = board_view::BoardView::new(game.board);
                    f.render_widget(board, chunks[0]);
                })?;
                let start = time::Instant::now();
                let alphabeta_res = minimax::alphabeta_init(game, 3, &minimax::weighted_piececount);
                let new_states = alphabeta_res.1;
                while start.elapsed() < ms(1000) {
                    std::thread::sleep(ms(100));
                }
                choose(new_states).unwrap()
            };
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
            [Constraint::Length(8), Constraint::Length(1)].as_ref(),
        );
        let board = board_view::BoardView::new(game.board);
        f.render_widget(board, chunks[0]);
        f.render_widget(Paragraph::new(game_result2), chunks[1]);
    })?;
    ui::show_abortable(&mut ctui, ms(30000));
    Ok(())
}

fn get_new_state_from_user(
    ctui: &mut ui::CTui,
    game: game::GameState,
) -> Result<Option<game::GameState>, Box<dyn std::error::Error>> {
    let mut selected_field: Option<usize> = None;
    loop {
        let (valid_targets, highlights, new_pos_and_states) = if let Some(field) = selected_field {
            let mut targets = HashSet::new();
            let piece = game.board.fields[field].unwrap().0;
            let new_pos_and_states = game.get_legal_moves_for_single_piece(piece, field);
            for &(new_pos, _) in &new_pos_and_states {
                targets.insert(new_pos);
            }
            let mut h = targets.clone();
            h.insert(field);
            (targets, h, new_pos_and_states)
        } else {
            (HashSet::new(), HashSet::new(), Vec::new())
        };
        let mut board_pos = tui::layout::Rect::new(0, 0, 24, 8);
        ctui.terminal().draw(|f| {
            let size = ui::center(f.size(), 24, 8);
            let chunks = ui::layout_vertical(size, [Constraint::Length(8)].as_ref());
            let board = board_view::BoardView::newh(game.board, highlights);
            board_pos = chunks[0];
            f.render_widget(board, board_pos);
        })?;
        loop {
            match event::read()? {
                event::Event::Key(key_event) => {
                    if key_event.code == event::KeyCode::Char('q') {
                        return Ok(None);
                    }
                }
                event::Event::Mouse(mouse_event) => {
                    if mouse_event.kind == event::MouseEventKind::Down(event::MouseButton::Left) {
                        selected_field = None;
                        if let Some(clicked_field) = board_view::BoardView::get_field_index_from_pos(
                            board_pos,
                            mouse_event.column,
                            mouse_event.row,
                        ) {
                            if valid_targets.contains(&clicked_field) { // TODO seems like this can be omitted using the conditions below
                                let new_states: Vec<game::GameState> = new_pos_and_states.into_iter().filter(|&(pos, _state)| pos==clicked_field).map(|(_pos, state)| state).collect();
                                if new_states.len() == 1 {
                                    return Ok(Some(new_states[0]));
                                } else if new_states.len() > 1 {
                                    // TODO promotions?
                                    return Ok(Some(new_states[0]));
                                } else {
                                    panic!("Internal error: Field was clickable but no new state found.");
                                }
                            } else if let Some(piece) = game.board.fields[clicked_field] {
                                if piece.1 == game.turn() {
                                    selected_field = Some(clicked_field);
                                    break;
                                }
                            }
                        }
                        break;
                    }
                }
                event::Event::Resize(_,_) => {
                    break; // trigger redraw
                }
            }
        }
    }
}

pub fn ms(millis: u64) -> time::Duration {
    time::Duration::from_millis(millis)
}

pub fn choose<T: Copy>(s: Vec<T>) -> Option<T> {
    s.choose(&mut rand::thread_rng()).map(|x| *x)
}

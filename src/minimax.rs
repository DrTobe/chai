/*
function minimax(node, depth, maximizingPlayer) is
    if depth = 0 or node is a terminal node then
        return the heuristic value of node
    if maximizingPlayer then
        value := −∞
        for each child of node do
            value := max(value, minimax(child, depth − 1, FALSE))
        return value
    else (* minimizing player *)
        value := +∞
        for each child of node do
            value := min(value, minimax(child, depth − 1, TRUE))
        return value
*/

use crate::game::*;

use rand::seq::SliceRandom;

// Returns tuple (best value, chosen action, number of nodes evaluated)
pub fn minimax(
    game_state: GameState,
    depth: usize,
    heuristic: &impl Fn(GameState) -> i32,
) -> (i32, Option<GameState>, u64) {
    if depth == 0 {
        return (heuristic(game_state), None, 1);
    }
    if game_state.fifty_move_rule_draw() {
        return (0, None, 1);
    }
    let new_states = game_state.get_legal_moves();
    if new_states.len() == 0 {
        if game_state.board.king_in_check(game_state.turn()) {
            match game_state.turn() {
                Player::White => return (i32::MIN, None, 1),
                Player::Black => return (i32::MAX, None, 1),
            }
        } else {
            return (0, None, 1);
        }
    }
    let (better, fold_init_val) = match game_state.turn() {
        Player::White => (std::cmp::Ordering::Greater, i32::MIN),
        Player::Black => (std::cmp::Ordering::Less, i32::MAX),
    };
    new_states
        .into_iter()
        .fold((fold_init_val, None, 0), |acc, new_state| {
            let minimax_res = minimax(new_state, depth - 1, heuristic);
            let num_nodes_evald = acc.2 + minimax_res.2;
            if minimax_res.0.cmp(&acc.0) == better {
                (minimax_res.0, Some(new_state), num_nodes_evald)
            } else if minimax_res.0 == acc.0 {
                (acc.0, one_of(acc.1, Some(new_state)), num_nodes_evald)
            } else {
                (acc.0, acc.1, num_nodes_evald)
            }
        })
}

fn one_of<T: Copy>(a: T, b: T) -> T {
    *[a, b].choose(&mut rand::thread_rng()).unwrap()
}

pub fn weighted_piececount(game: GameState) -> i32 {
    let mut sum = 0;
    for (piece, player) in game.board.get_pieces() {
        let factor = match player {
            Player::White => 1,
            Player::Black => -1,
        };
        let value = match piece {
            PieceType::InitKing => 0,
            PieceType::King => 0,
            PieceType::Queen => 90,
            PieceType::InitRook => 50,
            PieceType::Rook => 50,
            PieceType::Bishop => 30,
            PieceType::Knight => 30,
            PieceType::InitPawn => 10,
            PieceType::Pawn => 10,
        };
        sum += factor * value;
    }
    sum
}

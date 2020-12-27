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

pub fn minimax(game_state: GameState, depth: usize, heuristic: &Fn(GameState) -> i64) -> i64 {
    if depth == 0 {
        return heuristic(game_state);
    }
    if game_state.fifty_move_rule_draw() {
        return 0;
    }
    let new_states = game_state.get_legal_moves();
    if new_states.len() == 0 {
        if game_state.board.king_in_check(game_state.turn()) {
            match game_state.turn() {
                Player::White => return -1000,
                Player::Black => return 1000,
            }
        } else {
            return 0;
        }
    }
    let values = new_states
        .into_iter()
        .map(|s| minimax(s, depth - 1, heuristic));
    match game_state.turn() {
        Player::White => values.max().unwrap(),
        Player::Black => values.min().unwrap(),
    }
}

pub fn weighted_piececount(game: GameState) -> i64 {
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

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

// Returns tuple (best value, best actions, number of nodes evaluated)
pub fn minimax(
    game_state: GameState,
    depth: usize,
    heuristic: &impl Fn(GameState) -> i32,
) -> (i32, Vec<GameState>, u64) {
    if depth == 0 {
        return (heuristic(game_state), vec![], 1);
    }
    if game_state.fifty_move_rule_draw() {
        return (0, vec![], 1);
    }
    let new_states = game_state.get_legal_moves();
    if new_states.len() == 0 {
        if game_state.board.king_in_check(game_state.turn()) {
            match game_state.turn() {
                Player::White => return (i32::MIN, vec![], 1),
                Player::Black => return (i32::MAX, vec![], 1),
            }
        } else {
            return (0, vec![], 1);
        }
    }
    let (better, fold_init_val) = match game_state.turn() {
        Player::White => (std::cmp::Ordering::Greater, i32::MIN),
        Player::Black => (std::cmp::Ordering::Less, i32::MAX),
    };
    new_states
        .into_iter()
        .fold((fold_init_val, vec![], 1), |mut acc, new_state| {
            let minimax_res = minimax(new_state, depth - 1, heuristic);
            let num_nodes_evald = acc.2 + minimax_res.2;
            if minimax_res.0.cmp(&acc.0) == better {
                (minimax_res.0, vec![new_state], num_nodes_evald)
            } else if minimax_res.0 == acc.0 {
                acc.1.push(new_state);
                (acc.0, acc.1, num_nodes_evald)
            } else {
                (acc.0, acc.1, num_nodes_evald)
            }
        })
}

/* Alpha-Beta-Pruning as found on Wikipedia
 * https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning#Pseudocode
function alphabeta(node, depth, α, β, maximizingPlayer) is
    if depth = 0 or node is a terminal node then
        return the heuristic value of node
    if maximizingPlayer then
        value := −∞
        for each child of node do
            value := max(value, alphabeta(child, depth − 1, α, β, FALSE))
            α := max(α, value)
            if α ≥ β then
                break (* β cutoff *)
        return value
    else
        value := +∞
        for each child of node do
            value := min(value, alphabeta(child, depth − 1, α, β, TRUE))
            β := min(β, value)
            if β ≤ α then
                break (* α cutoff *)
        return value

(* Initial call *)
alphabeta(origin, depth, −∞, +∞, TRUE)

Personally, I find it annoying that the two branches for "maximizing player"
and "minimizing player" look so similar. Therefore, I changed the function
to use gamma and delta. gamma describes "the best value for the current player"
and delta "the best value for the other player". Like this, both branches
can be described with the same code. We just switch between Ordering::Greater
and Ordering::Less to describe "better" for the one or the other player. Then,
instead of maximizing, we check if a new value is "better" than an old value.
Instead of α ≥ β or β ≤ α, we just check if gamma is "better" than delta.
*/
pub fn alphabeta(
    game_state: GameState,
    depth: usize,
    mut gamma: i32,
    delta: i32,
    heuristic: &impl Fn(GameState) -> i32,
) -> (i32, Vec<GameState>, u64) {
    if depth == 0 {
        return (heuristic(game_state), vec![], 1);
    }
    if game_state.fifty_move_rule_draw() {
        return (0, vec![], 1);
    }
    let new_states = game_state.get_legal_moves();
    if new_states.len() == 0 {
        if game_state.board.king_in_check(game_state.turn()) {
            match game_state.turn() {
                Player::White => return (i32::MIN, vec![], 1),
                Player::Black => return (i32::MAX, vec![], 1),
            }
        } else {
            return (0, vec![], 1);
        }
    }

    let (better, mut best_val) = match game_state.turn() {
        Player::White => (std::cmp::Ordering::Greater, i32::MIN),
        Player::Black => (std::cmp::Ordering::Less, i32::MAX),
    };

    let mut actions = vec![];
    let mut num_nodes = 1;

    for new_state in new_states.into_iter() {
        let van = alphabeta(new_state, depth - 1, delta, gamma, heuristic);
        num_nodes += van.2;
        // maximize value
        if van.0.cmp(&best_val) == better {
            best_val = van.0;
            actions = vec![new_state];
            // maximize alpha/beta
            if best_val.cmp(&gamma) == better {
                gamma = best_val;
                // alpha/beta cutoff
                // we do not break on equality because this lets us consider
                // sub-optimal actions! breaking on equality is only valid
                // if we are strictly interested in the value of the root
                // node. In fact, we search for the best value of the
                // root node's children.
                if gamma.cmp(&delta) == better {
                    //|| gamma == delta {
                    break;
                }
            }
        } else if van.0 == best_val {
            actions.push(new_state);
        } else {
        }
    }

    (best_val, actions, num_nodes)
}

pub fn alphabeta_init(
    game_state: GameState,
    depth: usize,
    heuristic: &impl Fn(GameState) -> i32,
) -> (i32, Vec<GameState>, u64) {
    let (alpha, beta) = (i32::MIN, i32::MAX);
    match game_state.turn() {
        Player::White => alphabeta(game_state, depth, alpha, beta, heuristic),
        Player::Black => alphabeta(game_state, depth, beta, alpha, heuristic),
    }
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

use wasm_bindgen::prelude::*;

/* currently just left for reference
// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // Manufacture the element we're gonna append
    let val = document.create_element("p")?;
    val.set_inner_html("Hello from Rust!");

    body.append_child(&val)?;

    Ok(())
}
*/

mod game;
mod minimax;
mod util;

#[wasm_bindgen]
pub fn new_game() -> String {
    console_error_panic_hook::set_once();
    let board = game::BoardState::new();
    let game = game::GameState {
        board,
        ply: 0,
        fifty_move_rule_last_event: 0,
    };
    gamestate_to_json(game)
}

fn get_gamestate_from_json(json_game: String) -> game::GameState {
    let agame: game::AnnotatedGameState = serde_json::from_str(&json_game).unwrap();
    agame.game
}

fn gamestate_to_json(game: game::GameState) -> String {
    let agame = game::AnnotatedGameState::from(game);
    serde_json::to_string(&agame).unwrap()
}

#[wasm_bindgen]
pub fn get_legal_moves_for_single_piece(json_game: String, field: usize) -> String {
    let game = get_gamestate_from_json(json_game);
    let piece = game.board.fields[field].unwrap().0;
    let new_pos_and_states: Vec<(usize, game::AnnotatedGameState)> = game.get_legal_moves_for_single_piece(piece, field).into_iter().map(|pos_and_state| (pos_and_state.0, game::AnnotatedGameState::from(pos_and_state.1))).collect();
    serde_json::to_string(&new_pos_and_states).unwrap()
}

#[wasm_bindgen]
pub fn get_minimax_move(json_game: String) -> String {
    let game = get_gamestate_from_json(json_game);
    let alphabeta_res = minimax::alphabeta_init(game, 3, &minimax::weighted_piececount);
    let new_states = alphabeta_res.1;
    let new_state = util::choose(new_states).unwrap();
    gamestate_to_json(new_state)
}

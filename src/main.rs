use std::time;

mod board_view;
mod game;

fn main() {

    let mut board = game::BoardState::new();

    /*
    board_view::show_board(board, ms(300));
    board.fields[25] = Some((game::PieceType::Bishop, game::Player::White));
    board_view::show_board(board, ms(300));
    */

    let game = game::GameState {
        board,
        ply: 0
    };

    for new_state in game.get_pseudo_legal_moves() {
        board_view::show_board(new_state.board, ms(1000));
        board_view::show_board(game.board, ms(500));
    }
}

fn ms(millis: u64) -> time::Duration {
    time::Duration::from_millis(millis)
}

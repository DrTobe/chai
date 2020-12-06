#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieceType {
    InitKing,
    King,
    Queen,
    InitRook,
    Rook,
    Bishop,
    Knight,
    InitPawn,
    Pawn,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Player {
    Black,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoardState {
    pub fields: [Option<(PieceType, Player)>; 64],
    pub en_passant_field: EnPassantFieldInfo,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnPassantFieldInfo {
    pub ply: usize,
    pub skipped: usize,
    pub target: usize,
}

impl BoardState {
    pub fn new() -> Self {
        let mut board = [None; 64];
        let init_row = [
            PieceType::InitRook,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Queen,
            PieceType::InitKing,
            PieceType::Bishop,
            PieceType::Knight,
            PieceType::InitRook,
        ];
        for i in 0..8 {
            board[0 * 8 + i] = Some((init_row[i], Player::White));
            board[1 * 8 + i] = Some((PieceType::InitPawn, Player::White));
            board[7 * 8 + i] = Some((init_row[i], Player::Black));
            board[6 * 8 + i] = Some((PieceType::InitPawn, Player::Black));
        }
        let en_passant_field = EnPassantFieldInfo {
            ply: 0,
            skipped: 0xFF, // initialized invalid so that condition never true
            target: 0xFF,
        };
        BoardState {
            fields: board,
            en_passant_field,
        }
    }

    pub fn get_pieces_with_pos(&self, player: Player) -> Vec<(PieceType, usize)> {
        let mut pieces = Vec::with_capacity(16);
        for pos in 0..64 {
            if let Some((piece, curr_player)) = self.fields[pos] {
                if curr_player == player {
                    pieces.push((piece, pos));
                }
            }
        }
        pieces
    }
}

type Direction = (isize, isize);

const DIRECTIONS: [Direction; 16] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
    (2, 1),
    (1, 2),
    (-1, 2),
    (-2, 1),
    (-2, -1),
    (-1, -2),
    (1, -2),
    (2, -1),
];

use std::ops;
const WHITE_PAWN_MOVE: ops::Range<usize> = 0..1;
const BLACK_PAWN_MOVE: ops::Range<usize> = 1..2;
const WHITE_PAWN_CAPTURE: ops::Range<usize> = 4..6;
const BLACK_PAWN_CAPTURE: ops::Range<usize> = 6..8;
const STRAIGHT: ops::Range<usize> = 0..4;
const DIAGONAL: ops::Range<usize> = 4..8;
const STRAIGHT_AND_DIAGONAL: ops::Range<usize> = 0..8;
const KNIGHT: ops::Range<usize> = 8..16;

pub fn pos_from_rowcol(row: isize, col: isize) -> Option<usize> {
    if row >= 0 && row < 8 && col >= 0 && col < 8 {
        Some(row as usize * 8 + col as usize)
    } else {
        None
    }
}

pub fn get_steps(pos: usize, direction: Direction, steps: usize) -> Vec<(usize, usize)> {
    let row = (pos / 8) as isize;
    let col = (pos % 8) as isize;
    let mut positions = Vec::with_capacity(7);
    let steps = steps as isize;
    for step in 1..=steps {
        let (new_row, new_col) = (row + direction.0 * step, col + direction.1 * step);
        if let Some(new_pos) = pos_from_rowcol(new_row, new_col) {
            positions.push((new_pos, step as usize));
        }
    }
    positions
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameState {
    pub ply: usize,
    pub board: BoardState,
}

impl GameState {
    fn turn(&self) -> Player {
        if self.ply % 2 == 0 {
            Player::White
        } else {
            Player::Black
        }
    }

    fn new_state_from_to(&self, piece: PieceType, from: usize, to: usize) -> GameState {
        let mut new_state = *self;
        new_state.board.fields[from] = None;
        new_state.board.fields[to] = Some((piece, self.turn()));
        new_state.ply = self.ply + 1;
        new_state
    }

    // far = pieces which can move far distances ... plus king because it behaves
    // the same :)
    pub fn get_far_moves(
        &self,
        pos: usize,
        directions: &[Direction],
        max_steps: usize,
        move_to_empty: bool,
        move_to_capture: bool,
    ) -> Vec<(usize, usize)> {
        let mut positions = Vec::with_capacity(28); // queen: 4 dirs, 7 steps
        for direction in directions {
            for (new_pos, distance) in get_steps(pos, *direction, max_steps) {
                match self.board.fields[new_pos] {
                    None => {
                        if move_to_empty {
                            positions.push((new_pos, distance));
                        }
                    }
                    Some((_, player)) => {
                        if move_to_capture && player != self.turn() {
                            positions.push((new_pos, distance));
                        }
                        break;
                    }
                }
            }
        }
        positions
    }

    fn store_en_passant_info(&self, state: &mut GameState, pos: usize, new_pos: usize) {
        // Handling en passant movements. Here: remember that
        // a double step occured.
        state.board.en_passant_field = EnPassantFieldInfo {
            ply: self.ply,
            skipped: (pos + new_pos) / 2,
            target: new_pos,
        }
    }

    fn generate_pawn_promotions(&self, state: GameState, new_pos: usize) -> [GameState; 4] {
        let mut states = [state; 4];
        states[0].board.fields[new_pos] = Some((PieceType::Queen, self.turn()));
        states[1].board.fields[new_pos] = Some((PieceType::Rook, self.turn()));
        states[2].board.fields[new_pos] = Some((PieceType::Bishop, self.turn()));
        states[3].board.fields[new_pos] = Some((PieceType::Knight, self.turn()));
        states
    }

    fn get_pawn_moves(&self, player: Player) -> (usize, &[Direction], &[Direction]) {
        match player {
            Player::White => (
                7,
                &DIRECTIONS[WHITE_PAWN_MOVE],
                &DIRECTIONS[WHITE_PAWN_CAPTURE],
            ),
            Player::Black => (
                0,
                &DIRECTIONS[BLACK_PAWN_MOVE],
                &DIRECTIONS[BLACK_PAWN_CAPTURE],
            ),
        }
    }

    fn field_under_attack(&self, pos: usize) -> bool {
        for (other_pos, distance) in self.get_far_moves(pos, &DIRECTIONS[STRAIGHT], 7, false, true) {
            match self.board.fields[other_pos].expect("Only requested occupied fields.") {
                (PieceType::InitKing,_) | (PieceType::King,_) if distance == 1 => return true,
                (PieceType::InitKing,_) | (PieceType::King,_) => {},
                (PieceType::Queen, _) => return true,
                (PieceType::InitRook, _) | (PieceType::Rook, _) => return true,
                (PieceType::Bishop, _) => {},
                (PieceType::Knight, _) => {},
                (PieceType::InitPawn, _) | (PieceType::Pawn, _) => {},
            }
        }
        for (other_pos, distance) in self.get_far_moves(pos, &DIRECTIONS[DIAGONAL], 7, false, true) {
            match self.board.fields[other_pos].expect("Only requested occupied fields.") {
                (PieceType::InitKing,_) | (PieceType::King,_) if distance == 1 => return true,
                (PieceType::InitKing,_) | (PieceType::King,_) => {},
                (PieceType::Queen, _) => return true,
                (PieceType::InitRook, _) | (PieceType::Rook, _) => {},
                (PieceType::Bishop, _) => return true,
                (PieceType::Knight, _) => {},
                (PieceType::InitPawn, _) | (PieceType::Pawn, _) => {},
            }
        }
        for (other_pos, distance) in self.get_far_moves(pos, &DIRECTIONS[KNIGHT], 1, false, true) {
            match self.board.fields[other_pos].expect("Only requested occupied fields.") {
                (PieceType::Knight, _) => return true,
                _ => {}
            }
        }
        // For pawns, we need the movement direction of our pawns because that's
        // where attacking pawns (looking from our position) are located. It's
        // a little bit counter-intuitive.
        let (_, _, capture_moves) = self.get_pawn_moves(self.turn());
        for (other_pos, _) in self.get_far_moves(pos, capture_moves, 1, false, true) {
            match self.board.fields[other_pos].expect("Only requested occupied fields.") {
                (PieceType::InitPawn, _) | (PieceType::Pawn, _) => return true,
                _ => {}
            }
        }
        false
    }

    pub fn get_pseudo_legal_moves(&self) -> Vec<GameState> {
        let mut new_states = Vec::new();
        for (piece, pos) in self.board.get_pieces_with_pos(self.turn()) {
            new_states.extend(self.get_pseudo_legal_moves_for_single_piece(piece, pos));
        }
        new_states
    }
    pub fn get_pseudo_legal_moves_for_single_piece(&self, piece: PieceType, pos: usize) -> Vec<GameState> {
        let mut new_states = Vec::new();
        match piece {
            PieceType::InitKing | PieceType::King => {
                for (new_pos, _) in
                    self.get_far_moves(pos, &DIRECTIONS[STRAIGHT_AND_DIAGONAL], 1, true, true)
                {
                    new_states.push(self.new_state_from_to(PieceType::King, pos, new_pos));
                }
                // TODO castlings!
            }
            PieceType::Queen => {
                for (new_pos, _) in
                    self.get_far_moves(pos, &DIRECTIONS[STRAIGHT_AND_DIAGONAL], 7, true, true)
                {
                    new_states.push(self.new_state_from_to(PieceType::Queen, pos, new_pos));
                }
            }
            PieceType::InitRook | PieceType::Rook => {
                for (new_pos, _) in self.get_far_moves(pos, &DIRECTIONS[STRAIGHT], 7, true, true) {
                    new_states.push(self.new_state_from_to(PieceType::Rook, pos, new_pos));
                }
            }
            PieceType::Bishop => {
                for (new_pos, _) in self.get_far_moves(pos, &DIRECTIONS[DIAGONAL], 7, true, true) {
                    new_states.push(self.new_state_from_to(PieceType::Bishop, pos, new_pos));
                }
            }
            PieceType::Knight => {
                for (new_pos, _) in self.get_far_moves(pos, &DIRECTIONS[KNIGHT], 1, true, true) {
                    new_states.push(self.new_state_from_to(PieceType::Knight, pos, new_pos));
                }
            }
            PieceType::InitPawn => {
                let (_, move_moves, capture_moves) = self.get_pawn_moves(self.turn());
                for (i, (new_pos, _)) in self
                    .get_far_moves(pos, move_moves, 2, true, false)
                    .iter()
                    .enumerate()
                {
                    let mut new_state = self.new_state_from_to(PieceType::Pawn, pos, *new_pos);
                    if i == 1 {
                        self.store_en_passant_info(&mut new_state, pos, *new_pos);
                    }
                    new_states.push(new_state);
                }
                for (new_pos, _) in self.get_far_moves(pos, capture_moves, 1, false, true) {
                    new_states.push(self.new_state_from_to(PieceType::Pawn, pos, new_pos));
                }
            }
            PieceType::Pawn => {
                let (final_row, move_moves, capture_moves) = self.get_pawn_moves(self.turn());
                for (new_pos, _) in self.get_far_moves(pos, move_moves, 1, true, false) {
                    let new_state = self.new_state_from_to(piece, pos, new_pos);
                    if new_pos / 8 == final_row {
                        for promoted_state in
                            self.generate_pawn_promotions(new_state, new_pos).iter()
                        {
                            new_states.push(*promoted_state);
                        }
                    } else {
                        new_states.push(new_state);
                    }
                }
                for (new_pos, _) in self.get_far_moves(pos, capture_moves, 1, false, true) {
                    let new_state = self.new_state_from_to(piece, pos, new_pos);
                    if new_pos / 8 == final_row {
                        for promoted_state in
                            self.generate_pawn_promotions(new_state, new_pos).iter()
                        {
                            new_states.push(*promoted_state);
                        }
                    } else {
                        new_states.push(new_state);
                    }
                }
                if self.ply == self.board.en_passant_field.ply + 1 {
                    for (new_pos, _) in self.get_far_moves(pos, capture_moves, 1, true, false) {
                        // No promotions while capturing en-passant possible
                        if self.board.en_passant_field.skipped == new_pos {
                            let mut new_state = self.new_state_from_to(piece, pos, new_pos);
                            new_state.board.fields[self.board.en_passant_field.target] = None;
                            new_states.push(new_state);
                        }
                    }
                }
            }
        }
        new_states
    }

    fn _unused_placeholder(&self) {
    }
}

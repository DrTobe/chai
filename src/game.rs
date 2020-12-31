use std::ops;

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

impl Player {
    pub fn opponent(&self) -> Player {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
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

    pub fn get_pieces(&self) -> Vec<(PieceType, Player)> {
        let mut pieces = Vec::with_capacity(32);
        for pos in 0..64 {
            if let Some(pp) = self.fields[pos] {
                pieces.push(pp);
            }
        }
        pieces
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
        player: Player,
    ) -> Vec<(usize, usize)> {
        let mut positions = Vec::with_capacity(28); // queen: 4 dirs, 7 steps
        for direction in directions {
            for (new_pos, distance) in get_steps(pos, *direction, max_steps) {
                match self.fields[new_pos] {
                    None => {
                        if move_to_empty {
                            positions.push((new_pos, distance));
                        }
                    }
                    Some((_, p)) => {
                        if move_to_capture && p != player {
                            positions.push((new_pos, distance));
                        }
                        break;
                    }
                }
            }
        }
        positions
    }

    pub fn find_king(&self, player: Player) -> usize {
        for pos in 0..64 {
            if self.fields[pos] == Some((PieceType::King, player))
                || self.fields[pos] == Some((PieceType::InitKing, player))
            {
                return pos;
            }
        }
        panic!("Where is your King?");
    }

    fn field_under_attack(&self, pos: usize, player: Player) -> bool {
        for (other_pos, distance) in
            self.get_far_moves(pos, &DIRECTIONS[STRAIGHT], 7, false, true, player)
        {
            match self.fields[other_pos].expect("Only requested occupied fields.") {
                (PieceType::InitKing, p) | (PieceType::King, p) if distance == 1 => {
                    if p != player {
                        return true;
                    }
                }
                (PieceType::Queen, p) | (PieceType::InitRook, p) | (PieceType::Rook, p) => {
                    if p != player {
                        return true;
                    }
                }
                _ => {}
            }
        }
        for (other_pos, distance) in
            self.get_far_moves(pos, &DIRECTIONS[DIAGONAL], 7, false, true, player)
        {
            match self.fields[other_pos].expect("Only requested occupied fields.") {
                (PieceType::InitKing, p) | (PieceType::King, p) if distance == 1 => {
                    if p != player {
                        return true;
                    }
                }
                (PieceType::Queen, p) | (PieceType::Bishop, p) => {
                    if p != player {
                        return true;
                    }
                }
                _ => {}
            }
        }
        for (other_pos, _) in self.get_far_moves(pos, &DIRECTIONS[KNIGHT], 1, false, true, player) {
            match self.fields[other_pos].expect("Only requested occupied fields.") {
                (PieceType::Knight, p) => {
                    if p != player {
                        return true;
                    }
                }
                _ => {}
            }
        }
        // For pawns, we need the movement direction of our own pawns because that's
        // where attacking pawns (looking from our position) are located. It's
        // a little bit counter-intuitive.
        let (_, _, capture_moves) = get_pawn_moves(player);
        for (other_pos, _) in self.get_far_moves(pos, capture_moves, 1, false, true, player) {
            match self.fields[other_pos].expect("Only requested occupied fields.") {
                (PieceType::InitPawn, p) | (PieceType::Pawn, p) => {
                    if p != player {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    pub fn king_in_check(&self, player: Player) -> bool {
        let king_pos = self.find_king(player);
        self.field_under_attack(king_pos, player)
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

fn get_pawn_moves(player: Player) -> (usize, &'static [Direction], &'static [Direction]) {
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameState {
    pub ply: usize,
    pub fifty_move_rule_last_event: usize,
    pub board: BoardState,
}

impl GameState {
    pub fn turn(&self) -> Player {
        if self.ply % 2 == 0 {
            Player::White
        } else {
            Player::Black
        }
    }

    fn new_state_from_to(&self, piece: PieceType, from: usize, to: usize) -> (usize, GameState) {
        let mut new_state = *self;
        new_state.ply = self.ply + 1;
        new_state.board.fields[from] = None;
        let old = new_state.board.fields[to].replace((piece, self.turn()));
        if old != None || piece == PieceType::Pawn {
            new_state.fifty_move_rule_last_event = new_state.ply;
        }
        (to, new_state)
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

    fn generate_pawn_promotions(
        &self,
        pos_and_state: (usize, GameState),
    ) -> [(usize, GameState); 4] {
        let new_pos = pos_and_state.0;
        let mut states = [pos_and_state; 4];
        states[0].1.board.fields[new_pos] = Some((PieceType::Queen, self.turn()));
        states[1].1.board.fields[new_pos] = Some((PieceType::Rook, self.turn()));
        states[2].1.board.fields[new_pos] = Some((PieceType::Bishop, self.turn()));
        states[3].1.board.fields[new_pos] = Some((PieceType::Knight, self.turn()));
        states
    }

    /*
    pub fn get_legal_moves(&self) -> Vec<GameState> {
        //self.get_pseudo_legal_moves().drain_filter(...) // currently nightly only
        self.get_pseudo_legal_moves()
            .drain(..)
            .filter(|&new_state| !new_state.board.king_in_check(self.turn()))
            .collect()
    }
    */

    pub fn get_legal_moves(&self) -> Vec<GameState> {
        let mut new_states = Vec::new();
        for (piece, pos) in self.board.get_pieces_with_pos(self.turn()) {
            new_states.extend(
                self.get_legal_moves_for_single_piece(piece, pos)
                    .into_iter()
                    .map(|(_new_pos, new_state)| new_state),
            );
        }
        new_states
    }

    /*
    pub fn get_pseudo_legal_moves(&self) -> Vec<GameState> {
        let mut new_states = Vec::new();
        for (piece, pos) in self.board.get_pieces_with_pos(self.turn()) {
            new_states.extend(self.get_pseudo_legal_moves_for_single_piece(piece, pos));
        }
        new_states
    }
    */

    pub fn get_legal_moves_for_single_piece(
        &self,
        piece: PieceType,
        pos: usize,
    ) -> Vec<(usize, GameState)> {
        //self.get_pseudo_legal_moves_for_single_piece().drain_filter(...) // currently nightly only
        self.get_pseudo_legal_moves_for_single_piece(piece, pos)
            .drain(..)
            .filter(|&(_new_pos, new_state)| !new_state.board.king_in_check(self.turn()))
            .collect()
    }

    pub fn get_pseudo_legal_moves_for_single_piece(
        &self,
        piece: PieceType,
        pos: usize,
    ) -> Vec<(usize, GameState)> {
        let mut new_states = Vec::new();
        match piece {
            PieceType::InitKing | PieceType::King => {
                for (new_pos, _) in self.board.get_far_moves(
                    pos,
                    &DIRECTIONS[STRAIGHT_AND_DIAGONAL],
                    1,
                    true,
                    true,
                    self.turn(),
                ) {
                    new_states.push(self.new_state_from_to(PieceType::King, pos, new_pos));
                }
                if PieceType::InitKing == piece && !self.board.field_under_attack(pos, self.turn())
                {
                    let castling_options: [(isize, isize); 2] = [(-1, 4), (1, 3)];
                    for &(step, rook_distance) in castling_options.iter() {
                        let castling_pos = |distance| ((pos as isize) + step * distance) as usize;
                        if self.board.fields[castling_pos(rook_distance)]
                            != Some((PieceType::InitRook, self.turn()))
                        {
                            continue;
                        }
                        let occupied_field = (1..rook_distance)
                            .find(|&distance| self.board.fields[castling_pos(distance)] != None);
                        if let Some(_) = occupied_field {
                            continue;
                        }
                        if self.board.field_under_attack(castling_pos(1), self.turn())
                            || self.board.field_under_attack(castling_pos(2), self.turn())
                        {
                            continue;
                        }
                        let mut new_pos_and_state =
                            self.new_state_from_to(PieceType::King, pos, castling_pos(2));
                        new_pos_and_state.1.board.fields[castling_pos(rook_distance)] = None;
                        new_pos_and_state.1.board.fields[castling_pos(1)] =
                            Some((PieceType::Rook, self.turn()));
                        new_states.push(new_pos_and_state);
                    }
                }
            }
            PieceType::Queen => {
                for (new_pos, _) in self.board.get_far_moves(
                    pos,
                    &DIRECTIONS[STRAIGHT_AND_DIAGONAL],
                    7,
                    true,
                    true,
                    self.turn(),
                ) {
                    new_states.push(self.new_state_from_to(PieceType::Queen, pos, new_pos));
                }
            }
            PieceType::InitRook | PieceType::Rook => {
                for (new_pos, _) in
                    self.board
                        .get_far_moves(pos, &DIRECTIONS[STRAIGHT], 7, true, true, self.turn())
                {
                    new_states.push(self.new_state_from_to(PieceType::Rook, pos, new_pos));
                }
            }
            PieceType::Bishop => {
                for (new_pos, _) in
                    self.board
                        .get_far_moves(pos, &DIRECTIONS[DIAGONAL], 7, true, true, self.turn())
                {
                    new_states.push(self.new_state_from_to(PieceType::Bishop, pos, new_pos));
                }
            }
            PieceType::Knight => {
                for (new_pos, _) in
                    self.board
                        .get_far_moves(pos, &DIRECTIONS[KNIGHT], 1, true, true, self.turn())
                {
                    new_states.push(self.new_state_from_to(PieceType::Knight, pos, new_pos));
                }
            }
            PieceType::InitPawn => {
                let (_, move_moves, capture_moves) = get_pawn_moves(self.turn());
                for (i, (new_pos, _)) in self
                    .board
                    .get_far_moves(pos, move_moves, 2, true, false, self.turn())
                    .iter()
                    .enumerate()
                {
                    let mut new_pos_and_state =
                        self.new_state_from_to(PieceType::Pawn, pos, *new_pos);
                    if i == 1 {
                        self.store_en_passant_info(&mut new_pos_and_state.1, pos, *new_pos);
                    }
                    new_states.push(new_pos_and_state);
                }
                for (new_pos, _) in
                    self.board
                        .get_far_moves(pos, capture_moves, 1, false, true, self.turn())
                {
                    new_states.push(self.new_state_from_to(PieceType::Pawn, pos, new_pos));
                }
            }
            PieceType::Pawn => {
                let (final_row, move_moves, capture_moves) = get_pawn_moves(self.turn());
                for (new_pos, _) in
                    self.board
                        .get_far_moves(pos, move_moves, 1, true, false, self.turn())
                {
                    let new_pos_and_state = self.new_state_from_to(piece, pos, new_pos);
                    if new_pos / 8 == final_row {
                        for promoted_pos_and_state in
                            self.generate_pawn_promotions(new_pos_and_state).iter()
                        {
                            new_states.push(*promoted_pos_and_state);
                        }
                    } else {
                        new_states.push(new_pos_and_state);
                    }
                }
                for (new_pos, _) in
                    self.board
                        .get_far_moves(pos, capture_moves, 1, false, true, self.turn())
                {
                    let new_pos_and_state = self.new_state_from_to(piece, pos, new_pos);
                    if new_pos / 8 == final_row {
                        for promoted_pos_and_state in
                            self.generate_pawn_promotions(new_pos_and_state).iter()
                        {
                            new_states.push(*promoted_pos_and_state);
                        }
                    } else {
                        new_states.push(new_pos_and_state);
                    }
                }
                if self.ply == self.board.en_passant_field.ply + 1 {
                    for (new_pos, _) in
                        self.board
                            .get_far_moves(pos, capture_moves, 1, true, false, self.turn())
                    {
                        // No promotions while capturing en-passant possible
                        if self.board.en_passant_field.skipped == new_pos {
                            let mut new_pos_and_state = self.new_state_from_to(piece, pos, new_pos);
                            new_pos_and_state.1.board.fields[self.board.en_passant_field.target] =
                                None;
                            new_states.push(new_pos_and_state);
                        }
                    }
                }
            }
        }
        new_states
    }

    pub fn fifty_move_rule_draw(&self) -> bool {
        self.ply - self.fifty_move_rule_last_event >= 150
    }
}

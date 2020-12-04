#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieceType {
    InitKing,
    King,
    Queen,
    InitRook,
    Rook,
    Bishop,
    Knight,
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
            board[i] = Some((init_row[i], Player::White));
            board[1 * 8 + i] = Some((PieceType::Pawn, Player::White));
            board[7 * 8 + i] = Some((init_row[i], Player::Black));
            board[6 * 8 + i] = Some((PieceType::Pawn, Player::Black));
        }
        BoardState { fields: board }
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    RI, // row increase
    RICI, // row inc, col inc
    CI,
    RDCI,
    RD,
    RDCD,
    CD,
    RICD,
}

const STRAIGHT_AND_DIAGONAL: [Direction; 8] = [Direction::RI, Direction::CI, Direction::RD, Direction::CD, Direction::RICI, Direction::RDCI, Direction::RDCD, Direction::RICD];

pub fn pos_from_rowcol(row: isize, col: isize) -> Option<usize> {
    if row >= 0 && row < 8 && col >= 0 && col < 8 {
        Some(row as usize * 8 + col as usize)
    } else {
        None
    }
}


pub fn get_steps(pos: usize, direction: Direction, steps: usize) -> Vec<usize> {
    let row = (pos / 8) as isize;
    let col = (pos % 8) as isize;
    let mut positions = Vec::with_capacity(7);
    let steps = steps as isize;
    for step in 0..=steps {
        let (new_row, new_col) = match direction {
            Direction::RI => (row+step, col),
            Direction::RICI => (row+step, col+step),
            Direction::CI => (row, col+step),
            Direction::RDCI => (row-step, col+step),
            Direction::RD => (row-step, col),
            Direction::RDCD => (row-step, col-step),
            Direction::CD => (row, col-step),
            Direction::RICD => (row+step, col-step)
        };
        if let Some(pos) = pos_from_rowcol(new_row, new_col) {
            positions.push(pos);
        }
    }
    positions
}

// far = pieces which can move far distances ... plus king because it behaves
// the same :)
pub fn get_far_moves(board: &BoardState, pos: usize, directions: &[Direction], max_steps: usize, player: Player) -> Vec<usize> {
    let mut positions = Vec::with_capacity(28); // queen: 4 dirs, 7 steps
    for direction in directions {
        println!("{:?}", direction);
        for new_pos in get_steps(pos, *direction, max_steps) {
            println!("new_pos = {:?}", new_pos);
            match board.fields[new_pos] {
                None => {
                    println!("None push!");
                    positions.push(new_pos);
                },
                Some((_, curr_player)) => {
                    if curr_player == player {
                        break;
                    } else {
                        println!("Opponent push!");
                        positions.push(new_pos);
                        break;
                    }
                }
            }
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

    pub fn get_pseudo_legal_moves(&self) -> Vec<GameState> {
        let mut new_states = Vec::new();
        // "Default" movement (all but pawns): Move if
        for (piece, pos) in self.board.get_pieces_with_pos(self.turn()) {
            println!("{:?}", piece);
            match piece {
                PieceType::InitKing | PieceType::King => {
                    for new_pos in get_far_moves(&self.board, pos, &STRAIGHT_AND_DIAGONAL[..], 1, self.turn()) {
                        new_states.push(self.new_state_from_to(PieceType::King, pos, new_pos));
                    }
                    // TODO castlings!
                }
                PieceType::Queen => {
                    for new_pos in get_far_moves(&self.board, pos, &STRAIGHT_AND_DIAGONAL[..], 7, self.turn()) {
                        new_states.push(self.new_state_from_to(PieceType::Queen, pos, new_pos));
                    }
                }
                PieceType::InitRook | PieceType::Rook => {
                    for new_pos in get_far_moves(&self.board, pos, &STRAIGHT_AND_DIAGONAL[0..4], 7, self.turn()) {
                        new_states.push(self.new_state_from_to(PieceType::Rook, pos, new_pos));
                    }
                }
                PieceType::Bishop => {
                    for new_pos in get_far_moves(&self.board, pos, &STRAIGHT_AND_DIAGONAL[4..8], 7, self.turn()) {
                        new_states.push(self.new_state_from_to(PieceType::Bishop, pos, new_pos));
                    }
                }
                PieceType::Knight => {
                    let row = (pos / 8) as isize;
                    let col = (pos % 8) as isize;
                    for (i, j) in &[(1,2), (2,1), (-1,2), (-2,1), (1,-2), (2,-1), (-1,-2), (-2,-1)] {
                        let (new_row, new_col) = (row as isize + i, col as isize + j);
                        if let Some(new_pos) = pos_from_rowcol(new_row, new_col) {
                            match self.board.fields[new_pos] {
                                None => {
                                    new_states.push(self.new_state_from_to(PieceType::Knight, pos, new_pos));
                                },
                                Some((_, curr_player)) if curr_player != self.turn() => {
                                    new_states.push(self.new_state_from_to(PieceType::Knight, pos, new_pos));
                                },
                                _ => {}
                            }
                        }
                    }
                }
                PieceType::Pawn => {
                    // For pawns, we do not return any raw moves because those behave
                    // differently: pawns can not go forward if a field is occupied!
                    // The normal logic is: Move is possible if field is empty or
                    // occupied by opponent.
                    // The panicking behaviour was removed because it just requires
                    // additional checking.
                    //panic!("get_raw_moves() should not be called for Pawns!");
                    
                    // TODO handle pawns!
                }
            }
        }
        new_states
    }
}

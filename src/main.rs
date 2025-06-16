// src/main.rs
use std::io::{self, Write};
//read input
fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn coords_to_square(coord: &str) -> Option<Square> {
    if coord.len() != 2 {
        return None;
    }
    let chars: Vec<char> = coord.chars().collect();
    let file = chars[0].to_ascii_lowercase() as u8;
    let rank = chars[1] as u8;

    if file < b'a' || file > b'h' || rank < b'1' || rank > b'8' {
        return None;
    }

    let file_idx = file - b'a';
    let rank_idx = 7 - (rank - b'1'); // Invert the rank to match 0x88 board representation
    Some(rank_idx * 16 + file_idx)
}

const MAX_DEPTH: i32 = 4;

// Add piece-square tables for positional evaluation
const PAWN_PST: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
    5,  5, 10, 25, 25, 10,  5,  5,
    0,  0,  0, 20, 20,  0,  0,  0,
    5, -5,-10,  0,  0,-10, -5,  5,
    5, 10, 10,-20,-20, 10, 10,  5,
    0,  0,  0,  0,  0,  0,  0,  0
];

const KNIGHT_PST: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50
];

const BISHOP_PST: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20
];

const ROOK_PST: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    0,  0,  0,  5,  5,  0,  0,  0
];

const QUEEN_PST: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -5,  0,  5,  5,  5,  5,  0, -5,
    0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];

const KING_PST: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
    20, 20,  0,  0,  0,  0, 20, 20,
    20, 30, 10,  0,  0, 10, 30, 20
];

// Add piece values for MVV-LVA ordering
const PIECE_VALUES: [i32; 6] = [100, 320, 330, 500, 900, 20000]; // Pawn, Knight, Bishop, Rook, Queen, King

// Add MVV-LVA table for move ordering
const MVV_LVA: [[i32; 6]; 6] = [
    [105, 205, 305, 405, 505, 605], // Pawn captures
    [104, 204, 304, 404, 504, 604], // Knight captures
    [103, 203, 303, 403, 503, 603], // Bishop captures
    [102, 202, 302, 402, 502, 602], // Rook captures
    [101, 201, 301, 401, 501, 601], // Queen captures
    [100, 200, 300, 400, 500, 600], // King captures
];

fn get_piece_value(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => PIECE_VALUES[0],
        PieceType::Knight => PIECE_VALUES[1],
        PieceType::Bishop => PIECE_VALUES[2],
        PieceType::Rook => PIECE_VALUES[3],
        PieceType::Queen => PIECE_VALUES[4],
        PieceType::King => PIECE_VALUES[5],
    }
}

fn get_mvv_lva_score(attacker: PieceType, victim: PieceType) -> i32 {
    let attacker_idx = match attacker {
        PieceType::Pawn => 0,
        PieceType::Knight => 1,
        PieceType::Bishop => 2,
        PieceType::Rook => 3,
        PieceType::Queen => 4,
        PieceType::King => 5,
    };
    let victim_idx = match victim {
        PieceType::Pawn => 0,
        PieceType::Knight => 1,
        PieceType::Bishop => 2,
        PieceType::Rook => 3,
        PieceType::Queen => 4,
        PieceType::King => 5,
    };
    MVV_LVA[attacker_idx][victim_idx]
}

fn get_pst_value(square: Square, piece_type: PieceType, color: Color) -> i32 {
    let file = square & 0x7;
    let rank = square >> 4;
    let idx = if color == Color::White {
        rank * 8 + file
    } else {
        (7 - rank) * 8 + file
    } as usize;

    match piece_type {
        PieceType::Pawn => PAWN_PST[idx],
        PieceType::Knight => KNIGHT_PST[idx],
        PieceType::Bishop => BISHOP_PST[idx],
        PieceType::Rook => ROOK_PST[idx],
        PieceType::Queen => QUEEN_PST[idx],
        PieceType::King => KING_PST[idx],
    }
}

// Move struct for move ordering
#[derive(Clone, Copy)]
struct Move {
    from: Square,
    to: Square,
    score: i32,
}

fn evaluate(board: &Board, color: Color) -> i32 {
    let mut score = 0;

    for sq in 0u8..128 {
        if !Board::is_valid(sq) {
            continue;
        }
        if let Some(piece) = board.get_piece(sq) {
            let value = get_piece_value(piece.kind);
            let piece_value = if piece.color == color { value } else { -value };
            let pst_value = if piece.color == color {
                get_pst_value(sq, piece.kind, color)
            } else {
                -get_pst_value(sq, piece.kind, if color == Color::White { Color::Black } else { Color::White })
            };
            score += piece_value + pst_value;
        }
    }

    score
}

fn get_moves_with_scores(board: &Board, color: Color) -> Vec<Move> {
    let mut moves = Vec::new();
    
    for from in 0u8..128 {
        if !Board::is_valid(from) {
            continue;
        }
        if let Some(piece) = board.get_piece(from) {
            if piece.color == color {
                let legal_moves = board.generate_legal_moves_for_piece(from);
                for &to in &legal_moves {
                    let mut clone = board.clone();
                    clone.make_move(from, to);
                    if !is_in_check(&clone, color) {
                        let mut score = evaluate(&clone, color);
                        
                        // Add MVV-LVA score for captures
                        if let Some(captured_piece) = board.get_piece(to) {
                            if captured_piece.color != color {
                                score += get_mvv_lva_score(piece.kind, captured_piece.kind);
                            }
                        }
                        
                        moves.push(Move { from, to, score });
                    }
                }
            }
        }
    }
    
    // Sort moves by score in descending order
    moves.sort_by(|a, b| b.score.cmp(&a.score));
    moves
}

fn negamax(board: &Board, depth: i32, mut alpha: i32, beta: i32, color: Color) -> i32 {
    if depth == 0 {
        return evaluate(board, color);
    }

    let mut best_score = -i32::MAX;
    let mut has_legal_move = false;

    let moves = get_moves_with_scores(board, color);
    for mv in moves {
        let mut clone = board.clone();
        clone.make_move(mv.from, mv.to);
        if !is_in_check(&clone, color) {
            has_legal_move = true;
            let score = -negamax(&clone, depth - 1, -beta, -alpha, if color == Color::White { Color::Black } else { Color::White });
            best_score = best_score.max(score);
            alpha = alpha.max(score);
            if alpha >= beta {
                return alpha; // Beta cutoff
            }
        }
    }

    if !has_legal_move {
        if is_in_check(board, color) {
            return -i32::MAX + 1; // Checkmate
        }
        return 0; // Stalemate
    }

    best_score
}

fn find_best_move(board: &Board, color: Color) -> Option<(Square, Square)> {
    let mut best_score = -i32::MAX;
    let mut best_move = None;
    let mut alpha = -i32::MAX;
    let beta = i32::MAX;

    // Iterative deepening
    for depth in 1..=MAX_DEPTH {
        let moves = get_moves_with_scores(board, color);
        for mv in moves {
            let mut clone = board.clone();
            clone.make_move(mv.from, mv.to);
            if !is_in_check(&clone, color) {
                let score = -negamax(&clone, depth - 1, -beta, -alpha, if color == Color::White { Color::Black } else { Color::White });
                if score > best_score {
                    best_score = score;
                    best_move = Some((mv.from, mv.to));
                }
                alpha = alpha.max(score);
            }
        }
    }

    best_move
}

fn engine_make_move(board: &mut Board, engine_color: Color) -> bool {
    if let Some((from, to)) = find_best_move(board, engine_color) {
        board.make_move(from, to);
        println!(
            "Engine plays: {} -> {}",
            square_to_coords(from).unwrap(),
            square_to_coords(to).unwrap()
        );
        true
    } else {
        println!("Engine has no legal moves.");
        false
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Color {
    White,
    Black,
}

#[derive(Copy, Clone, PartialEq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Copy, Clone)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceType,
}

pub type Square = u8; // 0..127

#[derive(Clone)]
pub struct Board {
    pub squares: [Option<Piece>; 128], // 0x88 board
}

impl Board {
    pub fn new() -> Self {
        Self {
            squares: [None; 128],
        }
    }

    pub fn is_valid(square: Square) -> bool {
        square & 0x88 == 0
    }

    pub fn set_piece(&mut self, square: Square, piece: Piece) {
        if Self::is_valid(square) {
            self.squares[square as usize] = Some(piece);
        }
    }

    pub fn get_piece(&self, square: Square) -> Option<Piece> {
        if Self::is_valid(square) {
            self.squares[square as usize]
        } else {
            None
        }
    }

    pub fn make_move(&mut self, from: Square, to: Square) -> bool {
        if !Self::is_valid(from) || !Self::is_valid(to) {
            return false;
        }

        if let Some(piece) = self.get_piece(from) {
            self.squares[to as usize] = Some(piece);
            self.squares[from as usize] = None;
            true
        } else {
            false
        }
    }

    pub fn print(&self) {
        for rank in 0..8 {
            print!("{} ", 8 - rank);
            for file in 0..8 {
                let sq = rank * 16 + file;
                match self.get_piece(sq) {
                    Some(p) => print!("{} ", piece_char(p)),
                    None => print!(". "),
                }
            }
            println!();
        }
        println!("  a b c d e f g h");
    }

    pub fn print_with_highlights(&self, highlights: &[Square]) {
        for rank in (0..8).rev() {
            print!("{} ", rank + 1);
            for file in 0..8 {
                let sq = rank * 16 + file;
                if highlights.contains(&sq) {
                    print!("* ");
                } else {
                    match self.get_piece(sq) {
                        Some(p) => print!("{} ", piece_char(p)),
                        None => print!(". "),
                    }
                }
            }
            println!();
        }
        println!("  a b c d e f g h");
    }

    pub fn setup_starting_position(&mut self) {
        use Color::*;
        use PieceType::*;

        let white_back = [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook];
        for (i, &kind) in white_back.iter().enumerate() {
            self.set_piece(0x70 + i as u8, Piece { color: White, kind });
            self.set_piece(
                0x60 + i as u8,
                Piece {
                    color: White,
                    kind: Pawn,
                },
            );
        }

        let black_back = [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook];
        for (i, &kind) in black_back.iter().enumerate() {
            self.set_piece(0x00 + i as u8, Piece { color: Black, kind });
            self.set_piece(
                0x10 + i as u8,
                Piece {
                    color: Black,
                    kind: Pawn,
                },
            );
        }
    }

    pub fn generate_moves_for_square(&self, square: Square) -> Vec<Square> {
        self.generate_legal_moves_for_piece(square)
    }

    pub fn generate_pseudo_moves_for_piece(&self, square: Square) -> Vec<Square> {
        match self.get_piece(square) {
            Some(Piece { color, kind }) => match kind {
                PieceType::Knight => generate_knight_moves(self, square),
                PieceType::Pawn => generate_pawn_moves(self, square, color),
                PieceType::King => generate_king_moves(self, square, color),
                PieceType::Rook => generate_rook_moves(self, square, color),
                PieceType::Bishop => generate_bishop_moves(self, square, color),
                PieceType::Queen => generate_queen_moves(self, square, color),
            },
            None => vec![],
        }
    }

    pub fn generate_legal_moves_for_piece(&self, from: Square) -> Vec<Square> {
        let mut legal_moves = Vec::new();
        let pseudo_moves = self.generate_pseudo_moves_for_piece(from);

        for &to in &pseudo_moves {
            // Check if destination square has a piece of the same color
            if let Some(target_piece) = self.get_piece(to) {
                if let Some(moving_piece) = self.get_piece(from) {
                    if target_piece.color == moving_piece.color {
                        continue; // Skip if trying to capture own piece
                    }
                }
            }

            let mut cloned = self.clone();
            cloned.make_move(from, to);
            if let Some(piece) = self.get_piece(from) {
                if !is_in_check(&cloned, piece.color) {
                    legal_moves.push(to);
                }
            }
        }

        legal_moves
    }
}

pub fn is_in_check(board: &Board, color: Color) -> bool {
    let king_square = board.squares.iter().enumerate().find_map(|(i, &piece)| {
        if let Some(Piece {
            kind: PieceType::King,
            color: c,
        }) = piece
        {
            if c == color { Some(i as u8) } else { None }
        } else {
            None
        }
    });

    let king_sq = match king_square {
        Some(sq) => sq,
        None => return false,
    };

    for sq in 0u8..128 {
        if !Board::is_valid(sq) {
            continue;
        }
        if let Some(Piece { color: c, .. }) = board.get_piece(sq) {
            if c != color {
                let attacks = board.generate_pseudo_moves_for_piece(sq);
                if attacks.contains(&king_sq) {
                    return true;
                }
            }
        }
    }

    false
}

const KNIGHT_OFFSETS: [i8; 8] = [-33, -31, -18, -14, 14, 18, 31, 33];
const KING_OFFSETS: [i8; 8] = [-17, -15, -16, -1, 1, 15, 16, 17];

pub fn generate_knight_moves(board: &Board, from: Square) -> Vec<Square> {
    let mut moves = Vec::new();
    if let Some(piece) = board.get_piece(from) {
        for &offset in &KNIGHT_OFFSETS {
            let to = (from as i16 + offset as i16) as u8;
            if Board::is_valid(to) {
                match board.get_piece(to) {
                    Some(target) if target.color == piece.color => {}
                    _ => moves.push(to),
                }
            }
        }
    }
    moves
}

pub fn generate_king_moves(board: &Board, from: Square, color: Color) -> Vec<Square> {
    let mut moves = Vec::new();
    for &offset in &KING_OFFSETS {
        let to = from.wrapping_add(offset as u8);
        if Board::is_valid(to) {
            match board.get_piece(to) {
                Some(p) if p.color == color => {}
                _ => moves.push(to),
            }
        }
    }
    moves
}

pub fn generate_pawn_moves(board: &Board, from: Square, color: Color) -> Vec<Square> {
    let mut moves = Vec::new();
    let dir = if color == Color::White { -16 } else { 16 }; // Inverted direction
    let start_rank = if color == Color::White { 6 } else { 1 }; // Fixed starting ranks
    let rank = from >> 4;

    // Check diagonal captures first
    for &offset in &[dir - 1, dir + 1] {
        let to = (from as i16 + offset) as u8;
        if Board::is_valid(to) {
            if let Some(target) = board.get_piece(to) {
                if target.color != color {
                    moves.push(to);
                }
            }
        }
    }

    // Then check forward moves
    let one_step = (from as i16 + dir) as u8;
    if Board::is_valid(one_step) && board.get_piece(one_step).is_none() {
        moves.push(one_step);
        if rank == start_rank {
            let two_step = (from as i16 + 2 * dir) as u8;
            if board.get_piece(two_step).is_none() {
                moves.push(two_step);
            }
        }
    }

    moves
}

pub fn generate_rook_moves(board: &Board, from: Square, color: Color) -> Vec<Square> {
    let mut moves = Vec::new();
    let directions = [16, -16, 1, -1]; // up, down, right, left

    for &dir in &directions {
        let mut current = from;
        loop {
            let next = (current as i16 + dir) as u8;
            if !Board::is_valid(next) {
                break;
            }
            match board.get_piece(next) {
                Some(piece) if piece.color == color => break,
                Some(_) => {
                    moves.push(next);
                    break;
                }
                None => moves.push(next),
            }
            current = next;
        }
    }
    moves
}

pub fn generate_bishop_moves(board: &Board, from: Square, color: Color) -> Vec<Square> {
    let mut moves = Vec::new();
    let directions = [17, 15, -17, -15]; // up-right, up-left, down-right, down-left

    for &dir in &directions {
        let mut current = from;
        loop {
            let next = (current as i16 + dir) as u8;
            if !Board::is_valid(next) {
                break;
            }
            match board.get_piece(next) {
                Some(piece) if piece.color == color => break,
                Some(_) => {
                    moves.push(next);
                    break;
                }
                None => moves.push(next),
            }
            current = next;
        }
    }
    moves
}

pub fn generate_queen_moves(board: &Board, from: Square, color: Color) -> Vec<Square> {
    let mut moves = Vec::new();
    // Queen moves like a rook and bishop combined
    moves.extend(generate_rook_moves(board, from, color));
    moves.extend(generate_bishop_moves(board, from, color));
    moves
}

fn piece_char(piece: Piece) -> char {
    use Color::*;
    use PieceType::*;
    match (piece.color, piece.kind) {
        (White, Pawn) => 'P',
        (White, Knight) => 'N',
        (White, Bishop) => 'B',
        (White, Rook) => 'R',
        (White, Queen) => 'Q',
        (White, King) => 'K',
        (Black, Pawn) => 'p',
        (Black, Knight) => 'n',
        (Black, Bishop) => 'b',
        (Black, Rook) => 'r',
        (Black, Queen) => 'q',
        (Black, King) => 'k',
    }
}

fn square_to_coords(square: Square) -> Option<String> {
    if Board::is_valid(square) {
        let file = (square & 0x7) as u8;
        let rank = (square >> 4) + 1;
        Some(format!("{}{}", (b'a' + file) as char, rank))
    } else {
        None
    }
}

fn main() {
    let mut board = Board::new();
    board.setup_starting_position();

    let user_color = Color::White;
    let engine_color = Color::Black;

    loop {
        board.print();

        let input = read_input("\nEnter your move (e.g., e2 e4) or 'exit': ");
        if input == "exit" {
            break;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() != 2 {
            println!("Invalid format. Use: e2 e4");
            continue;
        }

        let from = match coords_to_square(parts[0]) {
            Some(sq) => sq,
            None => {
                println!("Invalid source square.");
                continue;
            }
        };

        let to = match coords_to_square(parts[1]) {
            Some(sq) => sq,
            None => {
                println!("Invalid destination square.");
                continue;
            }
        };

        if let Some(piece) = board.get_piece(from) {
            if piece.color != user_color {
                println!("That's not your piece.");
                continue;
            }

            let legal = board.generate_moves_for_square(from);
            if legal.contains(&to) {
                board.make_move(from, to);
            } else {
                println!("Illegal move.");
                continue;
            }
        } else {
            println!("No piece on source square.");
            continue;
        }

        if !engine_make_move(&mut board, engine_color) {
            println!("Game over. Engine has no legal moves.");
            break;
        }
    }
}


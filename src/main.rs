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
    let rank_idx = 8 - (rank - b'1');
    Some(rank_idx * 16 + file_idx)
}
fn engine_make_move(board: &mut Board, engine_color: Color) -> bool {
    let mut best_score = i32::MIN;
    let mut best_move = None;

    for from in 0u8..128 {
        if !Board::is_valid(from) {
            continue;
        }
        if let Some(piece) = board.get_piece(from) {
            if piece.color == engine_color {
                let moves = board.generate_moves_for_square(from);
                for &to in &moves {
                    let mut clone = board.clone();
                    clone.make_move(from, to);
                    let score = evaluate(&clone, engine_color);
                    if score > best_score {
                        best_score = score;
                        best_move = Some((from, to));
                    }
                }
            }
        }
    }

    if let Some((from, to)) = best_move {
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
        for rank in (0..8).rev() {
            print!("{} ", rank + 1);
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
                _ => vec![], // Add rook/bishop/queen later
            },
            None => vec![],
        }
    }

    pub fn generate_legal_moves_for_piece(&self, from: Square) -> Vec<Square> {
        let mut legal_moves = Vec::new();
        let pseudo_moves = self.generate_pseudo_moves_for_piece(from);

        for &to in &pseudo_moves {
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
            let to = from.wrapping_add(offset as u8);
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
    let dir = if color == Color::White { -16 } else { 16 };
    let start_rank = if color == Color::White { 6 } else { 1 };
    let rank = from >> 4;

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
        let rank = 8 - (square >> 4);
        Some(format!("{}{}", (b'a' + file) as char, rank))
    } else {
        None
    }
}
fn evaluate(board: &Board, color: Color) -> i32 {
    let mut score = 0;

    for sq in 0u8..128 {
        if !Board::is_valid(sq) {
            continue;
        }
        if let Some(piece) = board.get_piece(sq) {
            let value = match piece.kind {
                PieceType::Pawn => 100,
                PieceType::Knight => 320,
                PieceType::Bishop => 330,
                PieceType::Rook => 500,
                PieceType::Queen => 900,
                PieceType::King => 20000, // or 0 — king value only matters in check eval
            };
            score += if piece.color == color { value } else { -value };
        }
    }

    score
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

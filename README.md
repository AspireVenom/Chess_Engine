# Rust Chess Engine

A chess engine written in Rust that implements a negamax search algorithm with alpha-beta pruning and various chess-specific optimizations.

## Features

- Complete chess rules implementation
- Move generation for all piece types
- Legal move validation
- Negamax search with alpha-beta pruning
- Position evaluation using:
  - Material value
  - Piece-square tables
  - MVV-LVA move ordering
- Interactive command-line interface

## Technical Details

### Search Algorithm
- Negamax implementation with alpha-beta pruning
- Iterative deepening search
- Move ordering using MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
- Maximum search depth configurable via `MAX_DEPTH`

### Position Evaluation
- Material evaluation with standard piece values
- Piece-square tables for positional evaluation
- Separate tables for different piece types
- Opening and middlegame considerations

### Board Representation
- Uses 0x88 board representation for efficient move generation
- 128-square board (16x8) with invalid squares marked
- Efficient square validation and move generation

## Building and Running

### Prerequisites
- Rust and Cargo installed on your system

### Building
```bash
cargo build
```

### Running
```bash
cargo run
```

## Usage

1. Start the game by running the executable
2. The board will be displayed with standard chess notation
3. Enter moves in the format "e2 e4" (from square to square)
4. The engine will respond with its move
5. Type 'exit' to quit the game

## Move Input Format
- Use standard algebraic notation
- Format: "from_square to_square"
- Example: "e2 e4" for a pawn move
- Files are a-h, ranks are 1-8

## Future Improvements

Potential areas for enhancement:
- Transposition tables for position caching
- Quiescence search for better tactical play
- Opening book implementation
- UCI protocol support
- Improved evaluation with:
  - Mobility evaluation
  - Pawn structure analysis
  - King safety evaluation
  - Endgame tablebases

## License

This project is open source and available under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

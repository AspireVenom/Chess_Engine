# Rust Chess Engine

A chess engine written in Rust that implements a complete chess game with move generation, validation, and AI opponent.

## Features

- Complete chess move generation for all pieces
- Legal move validation including:
  - Check detection
  - Preventing moves that leave king in check
  - Proper piece capture rules
- Interactive command-line interface
- AI opponent using:
  - Negamax search algorithm with alpha-beta pruning
  - Iterative deepening
  - Move ordering with MVV-LVA (Most Valuable Victim - Least Valuable Attacker)
  - Position evaluation using:
    - Material counting
    - Piece-square tables for positional evaluation
    - Mobility evaluation

## Board Representation

The engine uses the 0x88 board representation, which provides:
- Efficient move generation
- Fast attack detection
- Easy boundary checking

## Move Generation

The engine supports all standard chess moves:
- Pawn moves (including first move two squares and diagonal captures)
- Knight moves in L-shape pattern
- Bishop diagonal moves
- Rook horizontal and vertical moves
- Queen moves (combination of rook and bishop)
- King moves (one square in any direction)

## AI Features

The AI opponent uses several chess-specific optimizations:
1. Negamax search with alpha-beta pruning for efficient tree traversal
2. Iterative deepening for better move selection
3. Move ordering using MVV-LVA to improve alpha-beta pruning efficiency
4. Position evaluation using:
   - Material counting (piece values)
   - Piece-square tables for positional evaluation
   - Mobility evaluation

## Usage

Run the engine with:
```bash
cargo run
```

Enter moves in the format "e2 e4" (from square to square).

## Future Improvements

Potential areas for enhancement:
- Transposition tables for better move caching
- Opening book integration
- Endgame tablebase support
- More sophisticated evaluation function
- UCI protocol support for compatibility with chess GUIs
- Multi-threading support for parallel search

## License

This project is open source and available under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

# rust-chess

`rust-chess` is a Python package that acts as a bridge between the `chess` crate and Python. It aims to provide a high-performance chess library that is largely compatible with `python-chess` syntax.

This repository provides:

- A Python package `rust-chess` created using Maturin.
- A type stub (`rust_chess.pyi`) providing hover documentation and examples in IDEs.
- A micro-benchmark comparison against `python-chess` in the file `tests/benchmark.py`.

## WARNING

This project is almost out of alpha/beta phase (pun intended). Maybe expect some breaking changes, refactoring, and new features.

## Overview

Quick usage example:

```python
import rust_chess as rc

board = rc.Board()  # Create a board
move = rc.Move.from_uci("e2e4")  # Create move from UCI

# Check if the move is legal for the current board
if board.is_legal_move(move):
    # Make a move on the current board
    # Disable the legality check since we already know the move is legal
    board.make_move(move, check_legality=False)

# Make move onto a new board
new_board = board.make_move_new(rc.Move("e7e5"))

# Get the FEN of the current board
print(board.get_fen())

# Generate the next move
move = board.generate_next_move()

# Create a list of all legal moves (exhausts the generator)
moves = list(board.generate_legal_moves())

# The generator saves state
assert move not in moves

# Reset the generator to be able to generate moves again
board.reset_move_generator()

# Generate legal captures
captures = list(board.generate_legal_captures())
```

Use IDE completion or read the generated stub (`rust_chess.pyi`) for detailed function signatures and documentation. Actual documentation coming soon (TM).

## Features

### Data Types and Constants

- `Color`: `WHITE`, `BLACK`, `COLORS`
- `PieceType`: `PAWN`, `KNIGHT`, `BISHOP`, `ROOK`, `QUEEN`, `KING`, `PIECE_TYPES`
- `Piece`: `WHITE_PAWN` ... `BLACK_KING`, `COLORED_PIECES`
- `Square`: `A1` .. `H8`, `SQUARES`
- `Bitboard`: `BB_EMPTY`, `BB_FULL`, `BB_FILE_A` ... `BB_FILE_H`, `BB_RANK_1` ... `BB_RANK_8`, `BB_FILES`, `BB_RANKS`
- `Move`: TODO: Add castling and null moves?
- `PyRepetitionDetectionMode` enum: `.NONE`, `.PARTIAL`, `.FULL`
  - Currently no difference between partial and full for now, but the plan is to have partial have a smaller history list
- `CastleRights` enum: `.NO_RIGHTS`, `.QUEENSIDE`, `.KINGSIDE`, `.BOTH`
- `BoardStatus` enum: `.ONGOING`, `.FIVE_FOLD_REPETITION`, `.SEVENTY_FIVE_MOVES`, `.INSUFFICIENT_MATERIAL`, `.STALEMATE`, `.CHECKMATE`
- `Board`: No constants.
- `BoardBatch`: No constants.

### Basic Features Overview

- Create a `Board` from an optional FEN string with `board = rc.Board()`.
- Get the FEN of a board by calling `get_fen()` on a board object.
- Iterate over every square using the `rc.SQUARES` constant or get an individual square by using the corresponding constant (ex. `rc.E2`).
- Create a `Bitboard` from an integer or square.
  - Supports bitwise operators, shift operators, popcnt, iteration, and conversion to and from a `Square`.
- Get many different bitboards for the current board including `board.get_color_bitboard(rc.WHITE)`, `board.get_piece_type_bitboard(rc.PAWN)`, `board.get_checkers_bitboard()`, and more.
- Create a move from a source and destination square, with an optional promotion piece type using `move = rc.Move(rc.E2, rc.E4)`.
  - Can also create a move from a UCI string using `move = rc.Move("e2e4")` or `move = rc.Move.from_uci("e2e4")`.
- Check if a move is legal with `board.is_legal_move(move)`.
- Generate all legal moves or captures for a board by iterating over `board.generate_legal_moves()` and `board.generate_legal_captures()`.
  - **The generator remembers state; make sure to reset it with `board.reset_move_generator()` if you want to iterate over the moves again.**
- Generate the next move for the generator with `board.generate_next_move()`.
- Generate moves for a specific bitboard mask by setting it with `board.set_move_generator_mask(mask_bitboard)` and then calling `board.generate_moves()`.
- Apply a move to a board with `board.make_move(move, check_legality=[True]|False)`.
  - `check_legality` defaults to `True` (can disable if you already know the move is legal for an extra performance boost).
- Apply a move to a new board with `new_board = board.make_move_new(move)`.
- Check what piece, piece type, or color is on a square with the corresponding `get_piece_on`, `get_piece_type_on`, and `get_color_on` functions.
- Get the `BoardStatus` enum of a board with `board.get_status()`.
  - Can also call individual status check functions like `board.is_checkmate()`, `board.is_insufficient_material()`, `board.is_fifty_moves()`, and more.
- Create a `BoardBatch` to apply functions to multiple boards at once.

## Installation

Requires Python 3.10+.

A pip package is available at: (https://pypi.org/project/rust-chess)[https://pypi.org/project/rust-chess]

1. Set up a virtual environment:

```sh
python -m venv .venv
source .venv/bin/activate
# Or
uv venv
source .venv/bin/activate
```

2. Use the pip package:

```sh
pip install rust-chess
# Or
uv pip install rust-chess
```

### Building From Source

1. Set up a virtual environment:

```sh
python -m venv .venv
source .venv/bin/activate
# Or
uv venv
source .venv/bin/activate
```

2. Clone the repository:

```sh
git clone https://github.com/nemeott/rust-chess.git
cd rust-chess
```

3. Build and install the Python package:

```sh
./scripts/build.sh
pip install target/wheels/rust_chess-0.4.0-cp313-cp313-linux_x86_64.whl
# Or
uv pip install target/wheels/rust_chess-0.4.0-cp313-cp313-linux_x86_64.whl

# Or build and install in current virtual environment
./scripts/develop.sh
```

## Roadmap

- [x] `Color`
  - [x] Color constants
  - [x] Comparison between colors and booleans
- [x] `PieceType`
  - [x] Piece type constants
  - [x] Get internal index representation
  - [x] Printing
    - [x] Basic characters
    - [x] Unicode characters
- [x] `Piece`
  - [x] Piece constants
  - [x] Get internal piece type index representation
  - [x] Printing
    - [x] Basic characters
    - [x] Unicode characters
- [x] `Square`
  - [x] Square constants
  - [x] Square creation and parsing
  - [x] Get the rank and file from a square
  - [x] Create a square from rank, file, or vice versa
  - [x] Get the color of a square
  - [x] Get the index of square
  - [x] Use a square as an index
  - [x] Rich comparison operators
  - [x] Flip a square vertically
  - [x] Bitboard conversion
  - [x] Get adjacent squares
  - [x] Get square forward/backward depending on color
  - [x] Printing
- [ ] `Bitboard`
  - [x] File and rank constants
  - [x] Creation from a square or integer
  - [x] Bitboard operations
    - [x] Between bitboards
    - [x] Between a bitboard and integer
  - [x] Count the number of bits
  - [x] Flip vertically
  - [x] Iterate over the squares in a bitboard
  - [x] Printing
    - [ ] Flip printing direction by default?
- [ ] `Move`
  - [x] Move creation from data types or UCI
  - [ ] Castling move constants
  - [ ] Null move constant?
- [x] `MoveGenerator`
  - [x] Generate the next move, legal move, and legal capture
  - [x] Generate moves, legal moves, and legal captures
  - [x] Support iterating over the generator
  - [x] Set a retain generator mask (bitboard of squares the generator will generate for)
  - [x] Set an exclude generator mask (bitboard of squares the generator will avoid)
  - [x] Remove a move from the generator
  - [x] Reset the generator
- [x] `CastleRights`
  - [x] Get castle rights (No rights, queenside, kingside, both)
  - [ ] Set castle rights? (use cases?)
  - [x] Rich comparison operators
- [x] `BoardStatus`
  - [x] Game-ending conditions
    - [x] Checkmate
    - [x] Stalemate
    - [x] Insufficient material
    - [x] Fivefold repetition
  - [x] Potential draw conditions
    - [x] Threefold repetition
    - [x] Fifty moves
  - [x] Rich comparison operators
- [ ] `Board`
  - [x] FEN parsing and printing
  - [x] SAN move parsing
  - [ ] Human readable printing
    - [x] Basic characters
    - [ ] ASCII with colors?
    - [x] Unicode characters
  - [x] Get the color, piece type, and piece on a square
  - [x] Get the king and en passant squares
  - [x] Get castle rights
  - [x] Check if move is zeroing or legal
  - [x] Quick legality detection for psuedo-legal moves
  - [x] Check if move is a capture or en passant or is castling
  - [x] Make moves on the current or new board
  - [ ] Make null moves (make_null_move)
  - [x] Make null moves on new board
  - [x] Get bitboards
    - [x] Pinned pieces
    - [x] Checking pieces
    - [x] Color pieces
    - [x] Piece type
    - [x] Piece
    - [x] All pieces
  - [x] Zobrist hashing
  - [x] Comparison operators (using Zobrist hash)
  - [x] Move history
    - [x] Repetition detection
  - [ ] Cache default board for faster creation?
  - [ ] Piece-Square Table support?
- [ ] `BoardBatch`
  - [x] Initialization
    - [x] Create a batch of boards from a count
    - [x] Create a batch of boards from a list of FEN strings.
    - [x] Create a batch of boards from a list of boards.
  - [x] FEN parsing and printing
  - [x] SAN move parsing
  - [x] Human readable printing
    - [x] Basic characters
    - [ ] ASCII with colors?
    - [x] Unicode characters
  - [x] Get the color, piece type, and piece on a respective square
  - [x] Get the king and en passant squares for each board
  - [x] Get castle rights for each board
  - [x] Check if a resepective move is zeroing or legal for each board
  - [x] Quick legality detection for psuedo-legal moves
  - [x] Check if a respective move is a capture or en passant or is castling for each board
  - [x] Make moves on the current or new board batch
  - [ ] Make null moves (make_null_move)
  - [x] Make null moves on new board batch
  - [x] Get bitboards for the batch
    - [x] Pinned pieces
    - [x] Checking pieces
    - [x] Color pieces
    - [x] Piece type
    - [x] Piece
    - [x] All pieces
  - [x] Zobrist hashing for each board
  - [x] Comparison operators (using Zobrist hash)
  - [x] Move history
    - [x] Repetition detection for each board
  - [x] Generate the next move, legal move, and legal capture for a batch
  - [x] Generate moves, legal moves, and legal captures for a batch
  - [x] Return a list of the generators
  - [x] Set the retain generator masks
  - [x] Set the exclude generator masks
  - [x] Remove moves from the generators
  - [x] Reset the generators
- [ ] Miscellaneous
  - [ ] PGN support (parsing and writing)
  - [ ] UCI protocol basics
  - [ ] Opening book support
  - [ ] Improved Python ergonomics (e.g., more Pythonic wrappers where appropriate)
  - [ ] Comprehensive test suite
    - [x] Docstring tests
    - [x] Benchmark comparision to `python-chess`
    - [ ] Other tests
  - [ ] Multi-threading
  - [ ] Python thread support?
  - [ ] Working GitHub action (😢)

## Comparison with python-chess

**`python-chess` generates moves in reverse order (H8, H7, ...)\* `rust-chess` generates moves in normal order (A1, A2, ...).**

### Performance

`scripts/benchmark.py` was used as a comprehensive benchmark comparision between similar functions in `rust-chess` and `python-chess`. Benchmarked on my Chromebook (Intel i5-1135G7). `scripts/batchmark.py` was used for a comparison between using methods on batches of boards. `python-chess` doesn't have a batch board class so this is kind of an unfair comparison. However, board batches could be useful for analyzing multiple games at once. The results from `rust-chess` v0.4.0 are as follows:

Benchmark Results (n=100,000)

| Category          | Rust Time | Python Time |    Speedup |
| ----------------- | --------: | ----------: | ---------: |
| Colors            |  0.005623 |    0.004736 |   0.842337 |
| Pieces            |  0.018838 |    0.009041 |   0.479946 |
| Squares           |  0.089639 |    0.045636 |   0.509110 |
| Moves             |  0.086117 |    0.217042 |   2.520321 |
| Board Init        |  0.091657 |    5.062907 |  55.237283 |
| Board Props       |  0.426570 |   11.517537 |  27.000372 |
| Board Ops         |  0.100331 |    0.578784 |   5.768751 |
| Board Ops 2       |  0.105453 |    5.327600 |  50.521126 |
| Make Move         |  0.079674 |    0.555655 |   6.974129 |
| Make Move (New)   |  0.090940 |    0.605637 |   6.659730 |
| Undo Move         |  0.091185 |    0.481893 |   5.284780 |
| Next Move         |  0.067739 |    0.459672 |   6.785930 |
| Generate Moves    |  0.208179 |   10.695765 |  51.377735 |
| SAN Parse         |  0.063843 |    0.651645 |  10.207014 |
| King Square       |  0.044894 |    0.131626 |   2.931930 |
| Zobrist Hash      |  0.045252 |    1.741447 |  38.483714 |
| Checkmate         |  0.048705 |    0.206944 |   4.248894 |
| Insufficient Mat. |  0.040469 |    0.174750 |   4.318096 |
| Bitboard Ops      |  0.036695 |    0.067098 |   1.828515 |
| Board Bitboards   |  0.070030 |    0.136282 |   1.946055 |
| Castle Rights     |  0.060880 |    0.333831 |   5.483449 |
| Repetitions       |  0.046618 |   13.510689 | 289.813933 |
| Board Status      |  0.049362 |    0.682978 |  13.836119 |
| Square/Piece Adv. |  0.033849 |    0.048523 |   1.433507 |
| Null Move         |  0.047808 |    0.322804 |   6.752067 |
| **Total**             |  **2.050350** |   **53.570522** |  **26.127503** |

Benchmark Results (n=10,000), (batch_size=25)

| Category          | Rust Time | Python Time |     Speedup |
| ----------------- | --------: | ----------: | ----------: |
| Board Init        |  0.028494 |    0.028333 |    0.994337 |
| Board Props       |  0.127417 |    3.670269 |   28.805164 |
| Board Ops         |  0.084218 |    1.528579 |   18.150214 |
| Board Ops 2       |  0.035208 |    1.710838 |   48.592270 |
| Make Move         |  0.046525 |    1.126794 |   24.219008 |
| Make Move (New)   |  0.046996 |    1.506249 |   32.050450 |
| Undo Move         |  0.046307 |    1.155118 |   24.944634 |
| Next Move         |  0.063420 |    1.172726 |   18.491501 |
| Generate Moves    |  0.164534 |   13.247108 |   80.513124 |
| SAN Parse         |  0.061047 |    1.690472 |   27.691106 |
| King Square       |  0.019242 |    0.326893 |   16.988751 |
| Zobrist Hash      |  0.017036 |    4.458977 |  261.736400 |
| Checkmate         |  0.024211 |    0.517528 |   21.375914 |
| Insufficient Mat. |  0.014820 |    0.418335 |   28.226859 |
| Board Bitboards   |  0.055708 |    0.339233 |    6.089521 |
| Castle Rights     |  0.018760 |    0.851835 |   45.407465 |
| Repetitions       |  0.015187 |   34.786077 | 2290.488025 |
| Board Status      |  0.024841 |    1.692779 |   68.144579 |
| Null Move         |  0.019370 |    0.787410 |   40.652059 |
| **Total**             |  **0.913341** |   **71.015554** |   **77.753579** |

#### Analysis

- Small/simple operations (e.g., some tiny getters, Python-exposed primitives) can be slightly slower because of Rust<->Python boundary costs.
- Complex and heavy operations are substantially faster in `rust-chess`:
  - Creating moves from UCI.
  - Board initialization.
  - FEN parsing and printing.
  - Getting piece bitboards.
  - Generating moves, legal moves, and legal captures.
  - Batch move generation (WIP).
  - San parsing.
  - Zobrist hashing.
  - Checking move legality and check.
  - Repetition detection.
  - Board status checks.

## Notable Limitations

- **Bridge overhead**: Small functions and data types are slower due to the bridge overhead, however heavy computations are much faster.
- **No board history yet**: Undo/pop are not available currently. Make moves onto a new board and pass it into a function instead for now.
- **Reliability**: The library has not been widely tested yet. It has pretty detailed docstring tests but not every edge case is guaranteed to be covered.

## License

MIT License.

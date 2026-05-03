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
  - [ ] FEN parsing and printing
  - [ ] SAN move parsing
  - [ ] Human readable printing
    - [ ] Basic characters
    - [ ] ASCII with colors?
    - [ ] Unicode characters
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
  - [ ] Generate moves, legal moves, and legal captures for a batch
  - [ ] Support iterating over the generators of a bacth? (how would this work?)
  - [ ] Set the retain generator masks
  - [ ] Set the exclude generator masks
  - [ ] Remove moves from the generators
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

| Category          |    Rust Time |   Python Time |       Speedup |
| ----------------- | -----------: | ------------: | ------------: |
| Colors            |     0.006386 |      0.005022 |      0.786379 |
| Pieces            |     0.019039 |      0.009006 |      0.473018 |
| Squares           |     0.098118 |      0.048066 |      0.489882 |
| Moves             |     0.086400 |      0.216912 |      2.510554 |
| Board Init        |     0.100258 |      5.018663 |     50.057617 |
| Board Props       |     0.437592 |     11.718235 |     26.778905 |
| Board Ops         |     0.102832 |      0.605381 |      5.887091 |
| Board Ops 2       |     0.111255 |      5.544555 |     49.836327 |
| Make Move         |     0.082378 |      0.582370 |      7.069442 |
| Make Move (New)   |     0.095266 |      0.629599 |      6.608846 |
| Undo Move         |     0.095382 |      0.502411 |      5.267335 |
| Next Move         |     0.070281 |      0.490481 |      6.978907 |
| Generate Moves    |     0.227567 |     11.174327 |     49.103443 |
| SAN Parse         |     0.065765 |      0.693099 |     10.539040 |
| King Square       |     0.047155 |      0.137995 |      2.926378 |
| Zobrist Hash      |     0.047038 |      1.827242 |     38.846229 |
| Checkmate         |     0.048880 |      0.215472 |      4.408168 |
| Insufficient Mat. |     0.042502 |      0.181757 |      4.276465 |
| Bitboard Ops      |     0.039632 |      0.071816 |      1.812098 |
| Board Bitboards   |     0.074357 |      0.141309 |      1.900411 |
| Castle Rights     |     0.060595 |      0.354572 |      5.851472 |
| Repetitions       |     0.048764 |     14.049805 |    288.118484 |
| Board Status      |     0.049658 |      0.689587 |     13.886646 |
| Square/Piece Adv. |     0.033687 |      0.049919 |      1.481863 |
| Null Move         |     0.049529 |      0.323132 |      6.524138 |
| **Total**         | **2.140316** | **55.280731** | **25.828308** |

Batchmark Results (n=10,000, batch_size=25)

| Category          |    Rust Time |   Python Time |       Speedup |
| ----------------- | -----------: | ------------: | ------------: |
| Board Init        |     0.026836 |      1.796330 |     66.937239 |
| Board Props       |     0.023323 |      2.377874 |    101.954685 |
| Board Ops         |     0.081179 |      1.418494 |     17.473598 |
| Board Ops 2       |     0.034512 |      1.621122 |     46.972399 |
| Make Move         |     0.044887 |      1.068798 |     23.810677 |
| Make Move (New)   |     0.045701 |      1.435041 |     31.400808 |
| Undo Move         |     0.045475 |      1.118573 |     24.597654 |
| Next Move         |     0.063923 |      1.161658 |     18.172829 |
| King Square       |     0.019241 |      0.322392 |     16.755049 |
| Zobrist Hash      |     0.016278 |      4.448038 |    273.256906 |
| Checkmate         |     0.023832 |      0.509187 |     21.365683 |
| Insufficient Mat. |     0.014700 |      0.408878 |     27.815226 |
| Board Bitboards   |     0.057257 |      0.332265 |      5.803102 |
| Castle Rights     |     0.018641 |      0.838173 |     44.962934 |
| Repetitions       |     0.014739 |     33.940829 |   2302.800619 |
| Board Status      |     0.024530 |      1.627044 |     66.328612 |
| Null Move         |     0.017461 |      0.761773 |     43.628063 |
| **Total**         | **0.572515** | **55.186469** | **96.393096** |

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

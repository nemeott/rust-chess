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
./build.sh
pip install target/wheels/rust_chess-0.3.3-cp313-cp313-linux_x86_64.whl
# Or
uv pip install target/wheels/rust_chess-0.3.3-cp313-cp313-linux_x86_64.whl

# Or build and install in current virtual environment
./develop.sh
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
  - [x] Get color, piece type, and piece on a square
  - [x] Get king square for a color
  - [x] Get the en passant square
  - [x] Check if move is zeroing
  - [x] Check if move is legal
  - [x] Quick legality detection for psuedo-legal moves
  - [x] Check if move is a capture
  - [x] Check if move is en passant
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
- [ ] Miscellaneous
  - [ ] PGN support (parsing and writing)
  - [ ] UCI protocol basics
  - [ ] Opening book support
  - [ ] Improved Python ergonomics (e.g., more Pythonic wrappers where appropriate)
  - [ ] Comprehensive test suite
    - [x] Docstring tests
    - [x] Benchmark comparision to `python-chess`
    - [ ] Other tests
  - [ ] Working GitHub action (😢)

## Comparison with python-chess

**`python-chess` generates moves in reverse order (H8, H7, ...)\* `rust-chess` generates moves in normal order (A1, A2, ...).**

### Performance

`scripts/compare.py` was used for a quick benchmark and comparison between the same operations for `rust-chess` and `python-chess`. The comparison script was run with large iteration counts (n = 100,000) and profiled using PySpy. The key observations from that analysis are as follows:

- Small/simple operations (e.g., some tiny getters, Python-exposed primitives) can be slightly slower because of Rust<->Python boundary costs.
- Complex and heavy operations are substantially faster in `rust-chess`:
  - Creating moves from UCI.
  - Board initialization.
  - FEN parsing and printing.
  - Generating legal moves and legal captures.
  - Checking move legality and check.

More detailed analysis is documented inside the file, including time deltas per function.

`scripts/benchmark.py` was used for a more complete benchmark comparision between similar functions in `rust-chess` and `python-chess`. Benchmarked on my Chromebook (Intel i5-1135G7). The results from `rust-chess` v0.3.3 are as follows:

Benchmark Results (n=100,000)

| Category          |    Rust Time |   Python Time |       Speedup |
| ----------------- | -----------: | ------------: | ------------: |
| Colors            |     0.005998 |      0.004842 |      0.807248 |
| Pieces            |     0.017569 |      0.009143 |      0.520384 |
| Squares           |     0.098369 |      0.047939 |      0.487340 |
| Moves             |     0.075502 |      0.216398 |      2.866129 |
| Board Init        |     0.119149 |      5.198251 |     43.628108 |
| Board Props       |     0.485058 |     11.891324 |     24.515271 |
| Board Ops         |     0.112402 |      0.621973 |      5.533447 |
| Board Ops 2       |     0.117081 |      5.603935 |     47.863628 |
| Make Move         |     0.088191 |      0.589247 |      6.681462 |
| Make Move (New)   |     0.106330 |      0.644731 |      6.063521 |
| Undo Move         |     0.106368 |      0.517714 |      4.867222 |
| Next Move         |     0.102310 |      0.485101 |      4.741503 |
| Generate Moves    |     0.229806 |     11.129185 |     48.428666 |
| SAN Parse         |     0.075681 |      0.693389 |      9.161989 |
| King Square       |     0.058984 |      0.136314 |      2.311033 |
| Zobrist Hash      |     0.058187 |      0.202811 |      3.485509 |
| Checkmate         |     0.062048 |      0.215870 |      3.479102 |
| Insufficient Mat. |     0.055382 |      0.178450 |      3.222183 |
| Bitboard Ops      |     0.039851 |      0.069541 |      1.745032 |
| Board Bitboards   |     0.084836 |      0.141772 |      1.671135 |
| Castle Rights     |     0.071389 |      0.368000 |      5.154844 |
| Repetitions       |     0.060330 |     14.024073 |    232.456000 |
| Board Status      |     0.061715 |      0.678618 |     10.995920 |
| Move Gen Ops      |     0.091647 |      2.726762 |     29.752994 |
| Square/Piece Adv. |     0.032743 |      0.048103 |      1.469109 |
| Null Move         |     0.060172 |      0.325820 |      5.414844 |
| Total             | **2.477097** | **56.769306** | **22.917678** |

## Notable Limitations

- **Bridge overhead**: Small functions and data types are slower due to the bridge overhead, however heavy computations are much faster.
- **No board history yet**: Undo/pop are not available currently. Make moves onto a new board and pass it into a function instead for now.
- **Reliability**: The library has not been widely tested yet. It has pretty detailed docstring tests but not every edge case is guaranteed to be covered.

## License

MIT License.

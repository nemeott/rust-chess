# ruff: noqa: ANN001, ANN201, D103, F841, B018
"""Comparison between python-chess and rust-chess.

Automated benchmarking with timing for each function category.

Notable differences between rust-chess and python-chess:
    - rust-chess does not currently support popping since there is no board history.

Results from rust-chess v0.4.0

Benchmark Results (n=100,000)
============================================================
Category          | Rust Time | Python Time |    Speedup
------------------------------------------------------------
Board Init        |  0.026836 |    1.796330 |  66.937239
Board Props       |  0.023323 |    2.377874 | 101.954685
Board Ops         |  0.081179 |    1.418494 |  17.473598
Board Ops 2       |  0.034512 |    1.621122 |  46.972399
Make Move         |  0.044887 |    1.068798 |  23.810677
Make Move (New)   |  0.045701 |    1.435041 |  31.400808
Undo Move         |  0.045475 |    1.118573 |  24.597654
Next Move         |  0.063923 |    1.161658 |  18.172829
King Square       |  0.019241 |    0.322392 |  16.755049
Zobrist Hash      |  0.016278 |    4.448038 | 273.256906
Checkmate         |  0.023832 |    0.509187 |  21.365683
Insufficient Mat. |  0.014700 |    0.408878 |  27.815226
Board Bitboards   |  0.057257 |    0.332265 |   5.803102
Castle Rights     |  0.018641 |    0.838173 |  44.962934
Repetitions       |  0.014739 |   33.940829 | 2302.800619
Board Status      |  0.024530 |    1.627044 |  66.328612
Null Move         |  0.017461 |    0.761773 |  43.628063
------------------------------------------------------------
Total             |  0.572515 |   55.186469 |  96.393096
"""

import time

import chess
import chess.polyglot

import rust_chess as rc


def batchmark(_name, rust_func, python_func, n=100_000):
    start = time.perf_counter()
    for _ in range(n):
        rust_func()
    rust_time = time.perf_counter() - start

    start = time.perf_counter()
    for _ in range(n):
        python_func()
    python_time = time.perf_counter() - start

    speedup = python_time / rust_time if rust_time > 0 else float("inf")
    return rust_time, python_time, speedup


FENS = [
    "rnbqkbnr/ppp1p1pp/5p2/3p4/4P3/3P4/PPP1KPPP/RNBQ1BNR b kq - 1 3",
    "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
    "rnbqkbnr/ppp1pppp/8/3p4/4PP2/8/PPPP2PP/RNBQKBNR b KQkq - 0 2",
]


def rust_board_init():
    boards = rc.BoardBatch(25)
    boards2 = rc.BoardBatch.from_fens(FENS)


def python_board_init():
    boards = [chess.Board() for _ in range(25)]
    boards2 = [chess.Board(fen) for fen in FENS]


def rust_board_props():
    boards2 = rc.BoardBatch.from_fens(FENS)
    str(boards2)
    # boards2.get_fen() # TODO
    boards2.halfmove_clocks
    boards2.fullmove_numbers
    boards2.turn
    boards2.is_fifty_moves()
    boards2.is_check()


def python_board_props():
    boards2 = [chess.Board(fen) for fen in FENS]
    for board in boards2:
        str(board)
        # board.fen()
        board.halfmove_clock
        board.fullmove_number
        board.turn
        board.is_fifty_moves()
        board.is_check()


def rust_board_ops():
    boards = rc.BoardBatch(25)
    moves = [rc.Move(rc.Square(12), rc.Square(28))] * 25
    squares = [rc.E2] * 25
    boards.is_legal_move(moves)
    boards.is_zeroing(moves)
    boards.get_piece_type_on(squares)
    boards.get_color_on(squares)
    boards.get_piece_on(squares)


def python_board_ops():
    boards = [chess.Board() for _ in range(25)]
    move = chess.Move(chess.Square(12), chess.Square(28))
    for board in boards:
        board.is_legal(move)
        board.is_zeroing(move)
        board.piece_type_at(chess.E2)
        board.color_at(chess.E2)
        board.piece_at(chess.E2)


def rust_board_ops2():
    boards2 = rc.BoardBatch.from_fens(FENS)
    moves2 = [rc.Move.from_uci("e2e3")] * len(FENS)
    squares2 = [rc.E2] * len(FENS)
    boards2.is_legal_move(moves2)
    boards2.is_zeroing(moves2)
    boards2.get_piece_type_on(squares2)
    boards2.get_color_on(squares2)
    boards2.get_piece_on(squares2)


def python_board_ops2():
    boards2 = [chess.Board(fen) for fen in FENS]
    move2 = chess.Move.from_uci("e2e3")
    for board in boards2:
        board.is_legal(move2)
        board.is_zeroing(move2)
        board.piece_type_at(chess.E2)
        board.color_at(chess.E2)
        board.piece_at(chess.E2)


def rust_make_move():
    boards = rc.BoardBatch(25)
    moves = [rc.Move.from_uci("g1f3")] * 25
    boards.make_move(moves, check_legality=True)


def python_make_move():
    boards = [chess.Board() for _ in range(25)]
    move = chess.Move.from_uci("g1f3")
    for board in boards:
        board.push(move)


def rust_make_move_new():
    boards = rc.BoardBatch(25)
    moves = [rc.Move(rc.Square(12), rc.Square(28))] * 25
    boards.make_move_new(moves)


def python_make_move_new():
    boards = [chess.Board() for _ in range(25)]
    move = chess.Move(chess.Square(12), chess.Square(28))
    for board in boards:
        board.copy().push(move)


def rust_undo_move():
    boards = rc.BoardBatch(25)
    moves = [rc.Move(rc.Square(12), rc.Square(28))] * 25
    boards.make_move_new(moves)


def python_undo_move():
    boards = [chess.Board() for _ in range(25)]
    move = chess.Move(chess.Square(12), chess.Square(28))
    for board in boards:
        board.push(move)
        board.pop()


def rust_next_move():
    boards = rc.BoardBatch(25)
    boards.generate_next_move()
    boards.reset_move_generator()


def python_next_move():
    boards = [chess.Board() for _ in range(25)]
    for board in boards:
        next(iter(board.legal_moves))


# def rust_generate(fen):
#     boards = rc.BoardBatch.from_fens([fen] * 25)
#     list(boards.generate_legal_captures())
#     list(boards.generate_legal_moves())


# def python_generate(fen):
#     boards = [chess.Board(fen) * 25]
#     for board in boards:
#         list(board.generate_legal_captures())
#         list(board.generate_legal_moves())


# def rust_san_parse():
#     boards = rc.BoardBatch(25)
#     boards.get_move_from_san(["e4"] * 25)


# def python_san_parse():
#     boards = [chess.Board() for _ in range(25)]
#     for board in boards:
#         board.parse_san("e4")


def rust_king_square():
    boards = rc.BoardBatch(25)
    boards.get_king_square(rc.WHITE)


def python_king_square():
    boards = [chess.Board() for _ in range(25)]
    for board in boards:
        board.king(chess.WHITE)


def rust_zobrist():
    boards = rc.BoardBatch(25)
    boards.zobrist_hash


def python_zobrist():
    boards = [chess.Board() for _ in range(25)]
    for board in boards:
        chess.polyglot.zobrist_hash(board)


def rust_checkmate():
    boards = rc.BoardBatch(25)
    boards.is_checkmate()


def python_checkmate():
    boards = [chess.Board() for _ in range(25)]
    for board in boards:
        board.is_checkmate()


def rust_insuff_mat():
    boards = rc.BoardBatch(25)
    boards.is_insufficient_material()


def python_insuff_mat():
    boards = [chess.Board() for _ in range(25)]
    for board in boards:
        board.is_insufficient_material()


def rust_board_bitboards():
    boards = rc.BoardBatch(25)
    boards.get_pinned_bitboard()
    boards.get_checkers_bitboard()
    boards.get_color_bitboard(rc.WHITE)
    boards.get_piece_type_bitboard(rc.PAWN)
    boards.get_piece_bitboard(rc.WHITE_PAWN)
    boards.get_all_bitboard()


def python_board_bitboards():
    boards = [chess.Board() for _ in range(25)]
    for board in boards:
        board.checkers_mask
        board.occupied_co[chess.WHITE]
        board.pieces_mask(chess.PAWN, chess.WHITE)
        board.occupied


def rust_castle_rights():
    boards = rc.BoardBatch(25)
    boards.can_castle(rc.WHITE)
    boards.can_castle_queenside(rc.WHITE)
    boards.can_castle_kingside(rc.WHITE)
    boards.get_castle_rights(rc.BLACK)
    boards.get_my_castle_rights()
    boards.get_their_castle_rights()


def python_castle_rights():
    boards = [chess.Board() for _ in range(25)]
    for board in boards:
        board.has_castling_rights(chess.WHITE)
        board.has_queenside_castling_rights(chess.WHITE)
        board.has_kingside_castling_rights(chess.WHITE)
        board.castling_rights


def rust_repetitions():
    boards = rc.BoardBatch(25)
    boards.is_threefold_repetition()
    boards.is_fivefold_repetition()
    boards.is_n_repetition(4)


def python_repetitions():
    boards = [chess.Board() for _ in range(25)]
    for board in boards:
        board.can_claim_threefold_repetition()
        board.is_fivefold_repetition()
        board.is_repetition(4)


def rust_board_status():
    boards = rc.BoardBatch(25)
    boards.get_status()


def python_board_status():
    boards = [chess.Board() for _ in range(25)]
    for board in boards:
        board.outcome()


def rust_null_move():
    boards = rc.BoardBatch(25)
    boards.make_null_move_new()


def python_null_move():
    boards = [chess.Board() for _ in range(25)]
    for board in boards:
        board.push(chess.Move.null())


if __name__ == "__main__":
    n = 10_000
    fen = "rnbqkbnr/ppp1pppp/8/3p4/2P1P3/8/PP1P1PPP/RNBQKBNR b KQkq - 0 2"

    benchmarks = [
        ("Board Init", rust_board_init, python_board_init),
        ("Board Props", rust_board_props, python_board_props),
        ("Board Ops", rust_board_ops, python_board_ops),
        ("Board Ops 2", rust_board_ops2, python_board_ops2),
        ("Make Move", rust_make_move, python_make_move),
        ("Make Move (New)", rust_make_move_new, python_make_move_new),
        ("Undo Move", rust_undo_move, python_undo_move),
        ("Next Move", rust_next_move, python_next_move),
        # ("Generate Moves", lambda: rust_generate(fen), lambda: python_generate(fen)), # TODO
        # ("SAN Parse", rust_san_parse, python_san_parse),
        ("King Square", rust_king_square, python_king_square),
        ("Zobrist Hash", rust_zobrist, python_zobrist),
        ("Checkmate", rust_checkmate, python_checkmate),
        ("Insufficient Mat.", rust_insuff_mat, python_insuff_mat),
        ("Board Bitboards", rust_board_bitboards, python_board_bitboards),
        ("Castle Rights", rust_castle_rights, python_castle_rights),
        ("Repetitions", rust_repetitions, python_repetitions),
        ("Board Status", rust_board_status, python_board_status),
        # ("Move Gen Ops", rust_move_gen_ops, python_move_gen_ops),
        ("Null Move", rust_null_move, python_null_move),
    ]

    print("Benchmark Results (n=100,000)")
    print("=" * 60)
    print(f"{'Category':<17} | {'Rust Time':>9} | {'Python Time':>11} | {'Speedup':>10}")
    print("-" * 60)

    times = []
    for name, rust_func, python_func in benchmarks:
        rust_time, python_time, speedup = batchmark(name, rust_func, python_func, n)
        times.append((rust_time, python_time, speedup))
        print(f"{name:<17} | {rust_time:>9f} | {python_time:>11f} | {speedup:>10f}")

    print("-" * 60)
    total_rust = sum(r for r, p, s in times)
    total_python = sum(p for r, p, s in times)
    total_speedup = total_python / total_rust if total_rust > 0 else float("inf")
    print(f"{'Total':<17} | {total_rust:>9f} | {total_python:>11f} | {total_speedup:>10f}")

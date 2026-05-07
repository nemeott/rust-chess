# ruff: noqa: ANN001, ANN201, D103, F841, B018
"""Comparison between python-chess and rust-chess.

Automated benchmarking with timing for each function category.

Notable differences between rust-chess and python-chess:
    - rust-chess does not currently support popping since there is no board history.

Results from rust-chess v0.4.2

Benchmark Results (n=10,000), (batch_size=25)
============================================================
Category          | Rust Time | Python Time |     Speedup
------------------------------------------------------------
Board Init        |  0.025174 |    1.978819 |   78.606221
Board Props       |  0.107029 |    2.030983 |   18.975970
Board Ops         |  0.065805 |    1.232720 |   18.732984
Board Ops 2       |  0.013758 |    0.036917 |    2.683418
Make Move         |  0.045168 |    1.175923 |   26.034443
Make Move (New)   |  0.030459 |    1.250573 |   41.058100
Undo Move         |  0.041040 |    1.212199 |   29.537241
Next Move         |  0.054150 |    0.938544 |   17.332415
Generate Moves    |  0.010455 |   13.309741 | 1273.005019
SAN Parse         |  0.049886 |    1.453338 |   29.133345
King Square       |  0.008389 |    0.038343 |    4.570424
Zobrist Hash      |  0.006540 |    4.458676 |  681.801300
Checkmate         |  0.013713 |    0.236573 |   17.251225
Insufficient Mat. |  0.004806 |    0.122893 |   25.573044
Board Bitboards   |  0.048134 |    0.049430 |    1.026933
Castle Rights     |  0.008145 |    0.589525 |   72.381478
Repetitions       |  0.004706 |   35.694660 | 7584.820916
Board Status      |  0.014827 |    1.414834 |   95.425070
Null Move         |  0.009174 |    0.453783 |   49.461692
------------------------------------------------------------
Total             |  0.561356 |   67.678473 |  120.562514
"""

import inspect
import time

import chess
import chess.polyglot

import rust_chess as rc


def batchmark(_name, rust_func, python_func, rust_args=(), python_args=(), n=100_000):
    start = time.perf_counter()
    for _ in range(n):
        rust_func(*rust_args)
    rust_time = time.perf_counter() - start

    start = time.perf_counter()
    for _ in range(n):
        python_func(*python_args)
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


def rust_board_props(boards2):
    str(boards2)
    boards2.get_fens()
    boards2.halfmove_clocks
    boards2.fullmove_numbers
    boards2.turn
    boards2.is_fifty_moves()
    boards2.is_check()


def python_board_props(boards2):
    for board in boards2:
        str(board)
        board.fen()
        board.halfmove_clock
        board.fullmove_number
        board.turn
        board.is_fifty_moves()
        board.is_check()


def rust_board_ops(boards, moves, squares):
    boards.is_legal_move(moves)
    boards.is_zeroing(moves)
    boards.get_piece_type_on(squares)
    boards.get_color_on(squares)
    boards.get_piece_on(squares)


def python_board_ops(boards, move):
    for board in boards:
        board.is_legal(move)
        board.is_zeroing(move)
        board.piece_type_at(chess.E2)
        board.color_at(chess.E2)
        board.piece_at(chess.E2)


def rust_board_ops2(boards2, moves2, squares2):
    boards2.is_legal_move(moves2)
    boards2.is_zeroing(moves2)
    boards2.get_piece_type_on(squares2)
    boards2.get_color_on(squares2)
    boards2.get_piece_on(squares2)


def python_board_ops2(boards2, move2):
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


def rust_make_move_new(boards, moves):
    boards.make_move_new(moves)


def python_make_move_new(boards, move):
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


def rust_next_move(boards):
    boards.generate_next_move()
    boards.reset_move_generator()


def python_next_move(boards):
    for board in boards:
        next(iter(board.legal_moves))


def rust_generate(boards):
    list(boards.generate_legal_captures())
    list(boards.generate_legal_moves())


def python_generate(boards):
    for board in boards:
        list(board.generate_legal_captures())
        list(board.generate_legal_moves())


def rust_san_parse(boards, sans):
    boards.get_move_from_san(sans)


def python_san_parse(boards):
    for board in boards:
        board.parse_san("e4")


def rust_king_square(boards):
    boards.get_king_square(rc.WHITE)


def python_king_square(boards):
    for board in boards:
        board.king(chess.WHITE)


def rust_zobrist(boards):
    boards.zobrist_hashes


def python_zobrist(boards):
    for board in boards:
        chess.polyglot.zobrist_hash(board)


def rust_checkmate(boards):
    boards.is_checkmate()


def python_checkmate(boards):
    for board in boards:
        board.is_checkmate()


def rust_insuff_mat(boards):
    boards.is_insufficient_material()


def python_insuff_mat(boards):
    for board in boards:
        board.is_insufficient_material()


def rust_board_bitboards(boards):
    boards.get_pinned_bitboard()
    boards.get_checkers_bitboard()
    boards.get_color_bitboard(rc.WHITE)
    boards.get_piece_type_bitboard(rc.PAWN)
    boards.get_piece_bitboard(rc.WHITE_PAWN)
    boards.get_all_bitboard()


def python_board_bitboards(boards):
    for board in boards:
        board.checkers_mask
        board.occupied_co[chess.WHITE]
        board.pieces_mask(chess.PAWN, chess.WHITE)
        board.occupied


def rust_castle_rights(boards):
    boards.can_castle(rc.WHITE)
    boards.can_castle_queenside(rc.WHITE)
    boards.can_castle_kingside(rc.WHITE)
    boards.get_castle_rights(rc.BLACK)
    boards.get_my_castle_rights()
    boards.get_their_castle_rights()


def python_castle_rights(boards):
    for board in boards:
        board.has_castling_rights(chess.WHITE)
        board.has_queenside_castling_rights(chess.WHITE)
        board.has_kingside_castling_rights(chess.WHITE)
        board.castling_rights


def rust_repetitions(boards):
    boards.is_threefold_repetition()
    boards.is_fivefold_repetition()
    boards.is_n_repetition(4)


def python_repetitions(boards):
    for board in boards:
        board.can_claim_threefold_repetition()
        board.is_fivefold_repetition()
        board.is_repetition(4)


def rust_board_status(boards):
    boards.get_status()


def python_board_status(boards):
    for board in boards:
        board.outcome()


def rust_null_move(boards):
    boards.make_null_move_new()


def python_null_move(boards):
    for board in boards:
        board.push(chess.Move.null())


if __name__ == "__main__":
    n = 10_000
    fen = "rnbqkbnr/ppp1pppp/8/3p4/2P1P3/8/PP1P1PPP/RNBQKBNR b KQkq - 0 2"

    rc_boards = rc.BoardBatch(25)
    rc_boards_fens = rc.BoardBatch.from_fens(FENS)
    rc_boards_fen = rc.BoardBatch.from_fens([fen] * 25)
    rc_moves = [rc.Move(rc.Square(12), rc.Square(28))] * 25
    rc_squares = [rc.E2] * 25
    rc_moves2 = [rc.Move.from_uci("e2e3")] * len(FENS)
    rc_squares2 = [rc.E2] * len(FENS)
    rc_sans = ["e4"] * 25

    py_boards = [chess.Board() for _ in range(25)]
    py_boards_fens = [chess.Board(f) for f in FENS]
    py_boards_fen = [chess.Board(fen)] * 25
    py_move = chess.Move(chess.Square(12), chess.Square(28))
    py_move2 = chess.Move.from_uci("e2e3")

    benchmarks = [
        ("Board Init", rust_board_init, python_board_init, (), ()),
        ("Board Props", rust_board_props, python_board_props, (rc_boards_fens,), (py_boards_fens,)),
        ("Board Ops", rust_board_ops, python_board_ops, (rc_boards, rc_moves, rc_squares), (py_boards, py_move)),
        (
            "Board Ops 2",
            rust_board_ops2,
            python_board_ops2,
            (rc_boards_fens, rc_moves2, rc_squares2),
            (py_boards_fens, py_move2),
        ),
        ("Make Move", rust_make_move, python_make_move, (), ()),
        ("Make Move (New)", rust_make_move_new, python_make_move_new, (rc_boards, rc_moves), (py_boards, py_move)),
        ("Undo Move", rust_undo_move, python_undo_move, (), ()),
        ("Next Move", rust_next_move, python_next_move, (rc_boards,), (py_boards,)),
        ("Generate Moves", rust_generate, python_generate, (rc_boards_fen,), (py_boards_fen,)),
        ("SAN Parse", rust_san_parse, python_san_parse, (rc_boards, rc_sans), (py_boards,)),
        ("King Square", rust_king_square, python_king_square, (rc_boards,), (py_boards,)),
        ("Zobrist Hash", rust_zobrist, python_zobrist, (rc_boards,), (py_boards,)),
        ("Checkmate", rust_checkmate, python_checkmate, (rc_boards,), (py_boards,)),
        ("Insufficient Mat.", rust_insuff_mat, python_insuff_mat, (rc_boards,), (py_boards,)),
        ("Board Bitboards", rust_board_bitboards, python_board_bitboards, (rc_boards,), (py_boards,)),
        ("Castle Rights", rust_castle_rights, python_castle_rights, (rc_boards,), (py_boards,)),
        ("Repetitions", rust_repetitions, python_repetitions, (rc_boards,), (py_boards,)),
        ("Board Status", rust_board_status, python_board_status, (rc_boards,), (py_boards,)),
        ("Null Move", rust_null_move, python_null_move, (rc_boards,), (py_boards,)),
    ]

    print("Benchmark Results (n=10,000), (batch_size=25)")
    print("=" * 60)
    print(f"{'Category':<17} | {'Rust Time':>9} | {'Python Time':>11} | {'Speedup':>11}")
    print("-" * 60)

    times = []
    for name, rust_func, python_func, r_args, p_args in benchmarks:
        rust_time, python_time, speedup = batchmark(name, rust_func, python_func, r_args, p_args, n)
        times.append((rust_time, python_time, speedup))
        print(f"{name:<17} | {rust_time:>9f} | {python_time:>11f} | {speedup:>11f}")

    print("-" * 60)
    total_rust = sum(r for r, p, s in times)
    total_python = sum(p for r, p, s in times)
    total_speedup = total_python / total_rust if total_rust > 0 else float("inf")
    print(f"{'Total':<17} | {total_rust:>9f} | {total_python:>11f} | {total_speedup:>11f}")

    print()

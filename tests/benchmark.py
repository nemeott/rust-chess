# ruff: noqa: ANN001, ANN201, D103, F841, B018
"""Comparison between python-chess and rust-chess.

Automated benchmarking with timing for each function category.

Notable differences between rust-chess and python-chess:
    - rust-chess does not currently support popping since there is no board history.

Results from rust-chess v0.4.2

Benchmark Results (n=100,000)
============================================================
Category          | Rust Time | Python Time |    Speedup
------------------------------------------------------------
Colors            |  0.006622 |    0.005583 |   0.843214
Pieces            |  0.018807 |    0.010251 |   0.545047
Squares           |  0.090577 |    0.044927 |   0.496016
Moves             |  0.076587 |    0.217583 |   2.840988
Board Init        |  0.066030 |    5.094908 |  77.160674
Board Props       |  0.349438 |    6.110863 |  17.487675
Board Ops         |  0.033388 |    0.468387 |  14.028470
Board Ops 2       |  0.029917 |    0.126599 |   4.231668
Make Move         |  0.052464 |    0.607373 |  11.576909
Make Move (New)   |  0.024611 |    0.492972 |  20.030613
Undo Move         |  0.064030 |    0.528625 |   8.255924
Next Move         |  0.030804 |    0.375795 |  12.199372
Generate Moves    |  0.027474 |    5.230344 | 190.373947
SAN Parse         |  0.025358 |    0.571996 |  22.556816
King Square       |  0.010181 |    0.020101 |   1.974365
Zobrist Hash      |  0.009077 |    1.781595 | 196.277714
Checkmate         |  0.011072 |    0.098640 |   8.909011
Insufficient Mat. |  0.006351 |    0.054583 |   8.593791
Bitboard Ops      |  0.041536 |    0.076017 |   1.830163
Board Bitboards   |  0.034426 |    0.024426 |   0.709532
Castle Rights     |  0.021094 |    0.248873 |  11.798300
Repetitions       |  0.012021 |   14.161510 | 1178.080356
Board Status      |  0.011402 |    0.544159 |  47.722964
Square/Piece Adv. |  0.034012 |    0.048505 |   1.426100
Null Move         |  0.010831 |    0.170929 |  15.782040
------------------------------------------------------------
Total             |  1.098110 |   37.115544 |  33.799470
"""

import time

import chess
import chess.polyglot

import rust_chess as rc


def benchmark(_name, rust_func, python_func, rust_args=(), python_args=(), n=100_000):
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


def rust_colors():
    color = rc.WHITE
    color2 = rc.COLORS[1]
    not color2


def python_colors():
    color = chess.WHITE
    color2 = chess.COLORS[1]
    not color2


def rust_pieces():
    pawn = rc.PAWN
    str(pawn)
    pawn.get_string()
    pawn.get_index()


def python_pieces():
    pawn = chess.PAWN
    str(pawn)


def rust_squares():
    square = rc.Square(12)
    square2 = rc.Square("E2")
    square3 = rc.A3
    str(square2)
    square2.get_name()
    square2.get_index()
    square2.get_file()
    square2.get_rank()
    square2.up()
    square2.down()
    square2.left()
    square2.right()


def python_squares():
    square = chess.Square(12)
    square2 = chess.parse_square("e2")
    square3 = chess.A3
    str(square2)
    chess.square_name(square2)
    square2
    chess.square_file(square2)
    chess.square_rank(square2)
    chess.square_mirror(square2)


def rust_moves():
    move = rc.Move(rc.Square(12), rc.Square(28))
    move2 = rc.Move.from_uci("E2e4")
    str(move2)
    move2.get_uci()
    move2.source
    move2.dest
    move2.promotion


def python_moves():
    move = chess.Move(chess.Square(12), chess.Square(28))
    move2 = chess.Move.from_uci("e2e4")
    str(move2)
    move2.uci()
    move2.from_square
    move2.to_square
    move2.promotion


def rust_board_init():
    board = rc.Board()
    board2 = rc.Board("rnbqkbnr/ppp1p1pp/5p2/3p4/4P3/3P4/PPP1KPPP/RNBQ1BNR b kq - 1 3")


def python_board_init():
    board = chess.Board()
    board2 = chess.Board("rnbqkbnr/ppp1p1pp/5p2/3p4/4P3/3P4/PPP1KPPP/RNBQ1BNR b kq - 1 3")


def rust_board_props(board2):
    str(board2)
    board2.get_fen()
    board2.halfmove_clock
    board2.fullmove_number
    board2.turn
    board2.is_fifty_moves()
    board2.is_check()


def python_board_props(board2):
    str(board2)
    board2.fen()
    board2.halfmove_clock
    board2.fullmove_number
    board2.turn
    board2.is_fifty_moves()
    board2.is_check()


def rust_board_ops(board, move):
    board.is_legal_move(move)
    board.is_zeroing(move)
    board.get_piece_type_on(rc.E2)
    board.get_color_on(rc.E2)
    board.get_piece_on(rc.E4)


def python_board_ops(board, move):
    board.is_legal(move)
    board.is_zeroing(move)
    board.piece_type_at(chess.E2)
    board.color_at(chess.E2)
    board.piece_at(chess.E4)


def rust_board_ops2(board2, move2):
    board2.is_legal_move(move2)
    board2.is_zeroing(move2)
    board2.get_piece_on(rc.E2)


def python_board_ops2(board2, move2):
    board2.is_legal(move2)
    board2.is_zeroing(move2)
    board2.piece_at(chess.E2)


def rust_make_move():
    board = rc.Board()
    move = rc.Move.from_uci("g1f3")
    board.make_move(move, check_legality=True)


def python_make_move():
    board = chess.Board()
    move = chess.Move.from_uci("g1f3")
    board.push(move)


def rust_make_move_new(board, move):
    board.make_move_new(move)


def python_make_move_new(board, move):
    board.copy().push(move)


def rust_undo_move():
    board = rc.Board()
    move = rc.Move(rc.Square(12), rc.Square(28))
    board.make_move_new(move)  # Apply and discard (no undo needed)


def python_undo_move():
    board = chess.Board()
    move = chess.Move(chess.Square(12), chess.Square(28))
    board.push(move)
    board.pop()


def rust_next_move(board):
    board.generate_next_move()
    board.reset_move_generator()


def python_next_move(board):
    next(iter(board.legal_moves))


def rust_generate(board):
    list(board.generate_legal_captures())
    list(board.generate_legal_moves())


def python_generate(board):
    list(board.generate_legal_captures())
    list(board.generate_legal_moves())


def rust_san_parse(board):
    board.get_move_from_san("e4")


def python_san_parse(board):
    board.parse_san("e4")


def rust_king_square(board):
    board.get_king_square(rc.WHITE)


def python_king_square(board):
    board.king(chess.WHITE)


def rust_zobrist(board):
    board.zobrist_hash


def python_zobrist(board):
    chess.polyglot.zobrist_hash(board)


def rust_checkmate(board):
    board.is_checkmate()


def python_checkmate(board):
    board.is_checkmate()


def rust_insuff_mat(board):
    board.is_insufficient_material()


def python_insuff_mat(board):
    board.is_insufficient_material()


def rust_bitboard_ops():
    bb1 = rc.BB_FILE_A | rc.BB_RANK_1
    bb2 = rc.BB_FILE_H & rc.BB_RANK_8
    bb3 = bb1 ^ bb2
    bb3.popcnt()
    bb3.flip_vertical()
    bb3 << 8
    bb3 >> 8


def python_bitboard_ops():
    bb1 = chess.BB_FILE_A | chess.BB_RANK_1
    bb2 = chess.BB_FILE_H & chess.BB_RANK_8
    bb3 = bb1 ^ bb2
    chess.popcount(bb3)
    chess.flip_vertical(bb3)
    bb3 << 8
    bb3 >> 8


def rust_board_bitboards(board):
    board.get_pinned_bitboard()
    board.get_checkers_bitboard()
    board.get_color_bitboard(rc.WHITE)
    board.get_piece_type_bitboard(rc.PAWN)
    board.get_piece_bitboard(rc.WHITE_PAWN)
    board.get_all_bitboard()


def python_board_bitboards(board):
    board.checkers_mask
    board.occupied_co[chess.WHITE]
    board.pieces_mask(chess.PAWN, chess.WHITE)
    board.occupied


def rust_castle_rights(board):
    board.can_castle(rc.WHITE)
    board.can_castle_queenside(rc.WHITE)
    board.can_castle_kingside(rc.WHITE)
    board.get_castle_rights(rc.BLACK)
    board.get_my_castle_rights()
    board.get_their_castle_rights()


def python_castle_rights(board):
    board.has_castling_rights(chess.WHITE)
    board.has_queenside_castling_rights(chess.WHITE)
    board.has_kingside_castling_rights(chess.WHITE)
    board.castling_rights


def rust_repetitions(board):
    board.is_threefold_repetition()
    board.is_fivefold_repetition()
    board.is_n_repetition(4)


def python_repetitions(board):
    board.can_claim_threefold_repetition()
    board.is_fivefold_repetition()
    board.is_repetition(4)


def rust_board_status(board):
    board.get_status()


def python_board_status(board):
    board.outcome()


def rust_square_piece_advanced():
    sq = rc.E4
    sq.get_color()
    sq.to_bitboard()
    sq.flip()
    piece = rc.WHITE_KING
    piece.get_unicode()


def python_square_piece_advanced():
    sq = chess.E4
    chess.square_file(sq)
    chess.square_rank(sq)
    piece = chess.Piece(chess.KING, chess.WHITE)
    piece.unicode_symbol()


def rust_null_move(board):
    board.make_null_move_new()


def python_null_move(board):
    board.push(chess.Move.null())


if __name__ == "__main__":
    n = 100_000
    fen = "rnbqkbnr/ppp1pppp/8/3p4/2P1P3/8/PP1P1PPP/RNBQKBNR b KQkq - 0 2"
    fen_2 = "rnbqkbnr/ppp1p1pp/5p2/3p4/4P3/3P4/PPP1KPPP/RNBQ1BNR b kq - 1 3"

    rc_board = rc.Board()
    rc_board_fen = rc.Board(fen)
    rc_board_fen2 = rc.Board(fen_2)
    rc_move = rc.Move(rc.Square(12), rc.Square(28))
    rc_move2 = rc.Move.from_uci("e2e3")

    py_board = chess.Board()
    py_board_fen = chess.Board(fen)
    py_board_fen2 = chess.Board(fen_2)
    py_move = chess.Move(chess.Square(12), chess.Square(28))
    py_move2 = chess.Move.from_uci("e2e3")

    benchmarks = [
        ("Colors", rust_colors, python_colors, (), ()),
        ("Pieces", rust_pieces, python_pieces, (), ()),
        ("Squares", rust_squares, python_squares, (), ()),
        ("Moves", rust_moves, python_moves, (), ()),
        ("Board Init", rust_board_init, python_board_init, (), ()),
        ("Board Props", rust_board_props, python_board_props, (rc_board_fen2,), (py_board_fen2,)),
        ("Board Ops", rust_board_ops, python_board_ops, (rc_board, rc_move), (py_board, py_move)),
        ("Board Ops 2", rust_board_ops2, python_board_ops2, (rc_board_fen2, rc_move2), (py_board_fen2, py_move2)),
        ("Make Move", rust_make_move, python_make_move, (), ()),
        ("Make Move (New)", rust_make_move_new, python_make_move_new, (rc_board, rc_move), (py_board, py_move)),
        ("Undo Move", rust_undo_move, python_undo_move, (), ()),
        ("Next Move", rust_next_move, python_next_move, (rc_board,), (py_board,)),
        ("Generate Moves", rust_generate, python_generate, (rc_board_fen,), (py_board_fen,)),
        ("SAN Parse", rust_san_parse, python_san_parse, (rc_board,), (py_board,)),
        ("King Square", rust_king_square, python_king_square, (rc_board,), (py_board,)),
        ("Zobrist Hash", rust_zobrist, python_zobrist, (rc_board,), (py_board,)),
        ("Checkmate", rust_checkmate, python_checkmate, (rc_board,), (py_board,)),
        ("Insufficient Mat.", rust_insuff_mat, python_insuff_mat, (rc_board,), (py_board,)),
        ("Bitboard Ops", rust_bitboard_ops, python_bitboard_ops, (), ()),
        ("Board Bitboards", rust_board_bitboards, python_board_bitboards, (rc_board,), (py_board,)),
        ("Castle Rights", rust_castle_rights, python_castle_rights, (rc_board,), (py_board,)),
        ("Repetitions", rust_repetitions, python_repetitions, (rc_board,), (py_board,)),
        ("Board Status", rust_board_status, python_board_status, (rc_board,), (py_board,)),
        ("Square/Piece Adv.", rust_square_piece_advanced, python_square_piece_advanced, (), ()),
        ("Null Move", rust_null_move, python_null_move, (rc_board,), (py_board,)),
    ]

    print("Benchmark Results (n=100,000)")
    print("=" * 60)
    print(f"{'Category':<17} | {'Rust Time':>9} | {'Python Time':>11} | {'Speedup':>10}")
    print("-" * 60)

    times = []
    for name, rust_func, python_func, r_args, p_args in benchmarks:
        rust_time, python_time, speedup = benchmark(name, rust_func, python_func, r_args, p_args, n)
        times.append((rust_time, python_time, speedup))
        print(f"{name:<17} | {rust_time:>9f} | {python_time:>11f} | {speedup:>10f}")

    print("-" * 60)
    total_rust = sum(r for r, p, s in times)
    total_python = sum(p for r, p, s in times)
    total_speedup = total_python / total_rust if total_rust > 0 else float("inf")
    print(f"{'Total':<17} | {total_rust:>9f} | {total_python:>11f} | {total_speedup:>10f}")

    print()

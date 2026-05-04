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
Colors            |  0.005623 |    0.004736 |   0.842337
Pieces            |  0.018838 |    0.009041 |   0.479946
Squares           |  0.089639 |    0.045636 |   0.509110
Moves             |  0.086117 |    0.217042 |   2.520321
Board Init        |  0.091657 |    5.062907 |  55.237283
Board Props       |  0.426570 |   11.517537 |  27.000372
Board Ops         |  0.100331 |    0.578784 |   5.768751
Board Ops 2       |  0.105453 |    5.327600 |  50.521126
Make Move         |  0.079674 |    0.555655 |   6.974129
Make Move (New)   |  0.090940 |    0.605637 |   6.659730
Undo Move         |  0.091185 |    0.481893 |   5.284780
Next Move         |  0.067739 |    0.459672 |   6.785930
Generate Moves    |  0.208179 |   10.695765 |  51.377735
SAN Parse         |  0.063843 |    0.651645 |  10.207014
King Square       |  0.044894 |    0.131626 |   2.931930
Zobrist Hash      |  0.045252 |    1.741447 |  38.483714
Checkmate         |  0.048705 |    0.206944 |   4.248894
Insufficient Mat. |  0.040469 |    0.174750 |   4.318096
Bitboard Ops      |  0.036695 |    0.067098 |   1.828515
Board Bitboards   |  0.070030 |    0.136282 |   1.946055
Castle Rights     |  0.060880 |    0.333831 |   5.483449
Repetitions       |  0.046618 |   13.510689 | 289.813933
Board Status      |  0.049362 |    0.682978 |  13.836119
Square/Piece Adv. |  0.033849 |    0.048523 |   1.433507
Null Move         |  0.047808 |    0.322804 |   6.752067
------------------------------------------------------------
Total             |  2.050350 |   53.570522 |  26.127503
"""

import time

import chess
import chess.polyglot

import rust_chess as rc


def benchmark(_name, rust_func, python_func, n=100_000):
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


def rust_board_props():
    board2 = rc.Board("rnbqkbnr/ppp1p1pp/5p2/3p4/4P3/3P4/PPP1KPPP/RNBQ1BNR b kq - 1 3")
    str(board2)
    board2.get_fen()
    board2.halfmove_clock
    board2.fullmove_number
    board2.turn
    board2.is_fifty_moves()
    board2.is_check()


def python_board_props():
    board2 = chess.Board("rnbqkbnr/ppp1p1pp/5p2/3p4/4P3/3P4/PPP1KPPP/RNBQ1BNR b kq - 1 3")
    str(board2)
    board2.fen()
    board2.halfmove_clock
    board2.fullmove_number
    board2.turn
    board2.is_fifty_moves()
    board2.is_check()


def rust_board_ops():
    board = rc.Board()
    move = rc.Move(rc.Square(12), rc.Square(28))
    board.is_legal_move(move)
    board.is_zeroing(move)
    board.get_piece_type_on(rc.E2)
    board.get_color_on(rc.E2)
    board.get_piece_on(rc.E4)


def python_board_ops():
    board = chess.Board()
    move = chess.Move(chess.Square(12), chess.Square(28))
    board.is_legal(move)
    board.is_zeroing(move)
    board.piece_type_at(chess.E2)
    board.color_at(chess.E2)
    board.piece_at(chess.E4)


def rust_board_ops2():
    board2 = rc.Board("rnbqkbnr/ppp1p1pp/5p2/3p4/4P3/3P4/PPP1KPPP/RNBQ1BNR b kq - 1 3")
    move2 = rc.Move.from_uci("e2e3")
    board2.is_legal_move(move2)
    board2.is_zeroing(move2)
    board2.get_piece_on(rc.E2)


def python_board_ops2():
    board2 = chess.Board("rnbqkbnr/ppp1p1pp/5p2/3p4/4P3/3P4/PPP1KPPP/RNBQ1BNR b kq - 1 3")
    move2 = chess.Move.from_uci("e2e3")
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


def rust_make_move_new():
    board = rc.Board()
    move = rc.Move(rc.Square(12), rc.Square(28))
    board.make_move_new(move)


def python_make_move_new():
    board = chess.Board()
    move = chess.Move(chess.Square(12), chess.Square(28))
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


def rust_next_move():
    board = rc.Board()
    board.generate_next_move()
    board.reset_move_generator()


def python_next_move():
    board = chess.Board()
    next(iter(board.legal_moves))


def rust_generate(fen):
    board = rc.Board(fen)
    list(board.generate_legal_captures())
    list(board.generate_legal_moves())


def python_generate(fen):
    board = chess.Board(fen)
    list(board.generate_legal_captures())
    list(board.generate_legal_moves())


def rust_san_parse():
    board = rc.Board()
    board.get_move_from_san("e4")


def python_san_parse():
    board = chess.Board()
    board.parse_san("e4")


def rust_king_square():
    board = rc.Board()
    board.get_king_square(rc.WHITE)


def python_king_square():
    board = chess.Board()
    board.king(chess.WHITE)


def rust_zobrist():
    board = rc.Board()
    board.zobrist_hash


def python_zobrist():
    board = chess.Board()
    chess.polyglot.zobrist_hash(board)
    # board._transposition_key()


def rust_checkmate():
    board = rc.Board()
    board.is_checkmate()


def python_checkmate():
    board = chess.Board()
    board.is_checkmate()


def rust_insuff_mat():
    board = rc.Board()
    board.is_insufficient_material()


def python_insuff_mat():
    board = chess.Board()
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


def rust_board_bitboards():
    board = rc.Board()
    board.get_pinned_bitboard()
    board.get_checkers_bitboard()
    board.get_color_bitboard(rc.WHITE)
    board.get_piece_type_bitboard(rc.PAWN)
    board.get_piece_bitboard(rc.WHITE_PAWN)
    board.get_all_bitboard()


def python_board_bitboards():
    board = chess.Board()
    board.checkers_mask
    board.occupied_co[chess.WHITE]
    board.pieces_mask(chess.PAWN, chess.WHITE)
    board.occupied


def rust_castle_rights():
    board = rc.Board()
    board.can_castle(rc.WHITE)
    board.can_castle_queenside(rc.WHITE)
    board.can_castle_kingside(rc.WHITE)
    board.get_castle_rights(rc.BLACK)
    board.get_my_castle_rights()
    board.get_their_castle_rights()


def python_castle_rights():
    board = chess.Board()
    board.has_castling_rights(chess.WHITE)
    board.has_queenside_castling_rights(chess.WHITE)
    board.has_kingside_castling_rights(chess.WHITE)
    board.castling_rights


def rust_repetitions():
    board = rc.Board()
    board.is_threefold_repetition()
    board.is_fivefold_repetition()
    board.is_n_repetition(4)


def python_repetitions():
    board = chess.Board()
    board.can_claim_threefold_repetition()
    board.is_fivefold_repetition()
    board.is_repetition(4)


def rust_board_status():
    board = rc.Board()
    board.get_status()


def python_board_status():
    board = chess.Board()
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


def rust_null_move():
    board = rc.Board()
    board.make_null_move_new()


def python_null_move():
    board = chess.Board()
    board.push(chess.Move.null())


if __name__ == "__main__":
    n = 100_000
    fen = "rnbqkbnr/ppp1pppp/8/3p4/2P1P3/8/PP1P1PPP/RNBQKBNR b KQkq - 0 2"

    benchmarks = [
        ("Colors", rust_colors, python_colors),
        ("Pieces", rust_pieces, python_pieces),
        ("Squares", rust_squares, python_squares),
        ("Moves", rust_moves, python_moves),
        ("Board Init", rust_board_init, python_board_init),
        ("Board Props", rust_board_props, python_board_props),
        ("Board Ops", rust_board_ops, python_board_ops),
        ("Board Ops 2", rust_board_ops2, python_board_ops2),
        ("Make Move", rust_make_move, python_make_move),
        ("Make Move (New)", rust_make_move_new, python_make_move_new),
        ("Undo Move", rust_undo_move, python_undo_move),
        ("Next Move", rust_next_move, python_next_move),
        ("Generate Moves", lambda: rust_generate(fen), lambda: python_generate(fen)),
        ("SAN Parse", rust_san_parse, python_san_parse),
        ("King Square", rust_king_square, python_king_square),
        ("Zobrist Hash", rust_zobrist, python_zobrist),
        ("Checkmate", rust_checkmate, python_checkmate),
        ("Insufficient Mat.", rust_insuff_mat, python_insuff_mat),
        ("Bitboard Ops", rust_bitboard_ops, python_bitboard_ops),
        ("Board Bitboards", rust_board_bitboards, python_board_bitboards),
        ("Castle Rights", rust_castle_rights, python_castle_rights),
        ("Repetitions", rust_repetitions, python_repetitions),
        ("Board Status", rust_board_status, python_board_status),
        ("Square/Piece Adv.", rust_square_piece_advanced, python_square_piece_advanced),
        ("Null Move", rust_null_move, python_null_move),
    ]

    print("Benchmark Results (n=100,000)")
    print("=" * 60)
    print(f"{'Category':<17} | {'Rust Time':>9} | {'Python Time':>11} | {'Speedup':>10}")
    print("-" * 60)

    times = []
    for name, rust_func, python_func in benchmarks:
        rust_time, python_time, speedup = benchmark(name, rust_func, python_func, n)
        times.append((rust_time, python_time, speedup))
        print(f"{name:<17} | {rust_time:>9f} | {python_time:>11f} | {speedup:>10f}")

    print("-" * 60)
    total_rust = sum(r for r, p, s in times)
    total_python = sum(p for r, p, s in times)
    total_speedup = total_python / total_rust if total_rust > 0 else float("inf")
    print(f"{'Total':<17} | {total_rust:>9f} | {total_python:>11f} | {total_speedup:>10f}")

    print()

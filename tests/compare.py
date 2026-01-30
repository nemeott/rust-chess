"""Comparison between python-chess and rust-chess.

Notable differences between rust-chess and python-chess:
    - rust-chess does not currently support popping since there is no board history.
    - rust-chess doesn't print a human readable board (grid format) yet, only FEN.

Tests the same features from both for fair comparison.
The tests were run with n = 100,000 and profiled using py-spy (used the VSCode extension to get times in the gutter).
The time delta (rust-chess time - python-chess time) is annotated next to the respective functions.

Conclusions from rust-chess 0.2.0:
    - Small functions and data types take slightly longer (Python-native data types are faster than exporting rust types).
    - Complex functions are much faster; these include:
        - Creating a move from UCI
        - Initializing a board
        - Initializing a board with a FEN string (substantial)
        - Printing the FEN of a board (substantial)
        - Player is in check
        - Is legal move (medium)
        - Generating next move
        - Generating legal captures (substantial)
        - Generating legal moves (substantial)
"""  # noqa: E501

import chess

import rust_chess as rc

# TODO: Auto time and compare functions


def test_rust_chess() -> None:  # noqa: PLR0915
    """Test the rust-chess library."""
    color = rc.WHITE  # +0.03s
    color2 = rc.COLORS[1]  # +0.04s
    print(color)  # +0.18s
    print(color2)  # +0.04s
    print(not color2)  # +0.03s
    print()

    pawn = rc.PAWN  # Same
    print(pawn)  # +0.11s
    print(pawn.get_string())  # Takes 0.72s
    print(pawn.get_index())  # Takes 1.05s
    print()

    square = rc.Square(12)  # +0.11s  # noqa: F841
    square2 = rc.Square("E2")  # +0.33s
    square3 = rc.A3  # -0.01s  # noqa: F841
    print(square2)  # -0.02s
    print(square2.get_name())  # +0.19s
    print(square2.get_index())  # Takes 0.86s
    print(square2.get_file())  # +0.11s
    print(square2.get_rank())  # +0.22s
    print(square2.up())  # Takes 0.89s
    print(square2.down())  # Takes 0.95s
    print(square2.left())  # Takes 0.90s
    print(square2.right())  # Takes 0.71s
    print()

    move = rc.Move(rc.Square(12), rc.Square(28))  # -0.12s
    move2 = rc.Move.from_uci("E2e4")  # -1.12s
    print(move2)  # -0.16s
    print(move2.get_uci())  # -0.06s
    print(move2.source)  # +0.10s
    print(move2.dest)  # +0.25s
    print(move2.promotion)  # +0.21s
    print()

    board = rc.Board()  # -0.54s
    board2 = rc.Board("rnbqkbnr/ppp1p1pp/5p2/3p4/4P3/3P4/PPP1KPPP/RNBQ1BNR b kq - 1 3")  # -23.32s
    print(board2)  # -10.60s (Not completely comparable (FEN vs grid))
    print(board2.get_fen())  # -14.78s
    print(board2.halfmove_clock)  # +0.27s
    print(board2.fullmove_number)  # +0.15s
    print(board2.turn)  # +0.32s
    print(board2.is_fifty_moves())  # +0.04s
    print(board2.is_check())  # -0.53s
    print(board.is_legal_move(move))  # -3.49s
    print(board2.is_legal_move(move2))  # -5.19s

    print(board.is_zeroing(move))  # Pawn move # +0.09s
    print(board2.is_zeroing(rc.Move.from_uci("e2e3")))  # -1.59 (Likely better because UCI conversion is faster)

    print(board.get_piece_type_on(rc.E2))  # +0.06s
    print(board.get_color_on(rc.E2))  # +0.21s
    print(board.get_piece_on(rc.E4))  # -0.05s
    print(board2.get_piece_on(rc.E2))  # -0.42s

    # The rust-chess board does not currently support popping a move (no history stored)
    board3 = board.make_move_new(move)  # Pawn move # Takes 0.17s
    print(board3)  # Takes 2.07s
    move = rc.Move.from_uci("g1f3")  # Horse move
    board.make_move(move, check_legality=True)  # Horse move # -2.81s (including line above)
    print(board)  # -10.44s (Not completely comparable (FEN vs grid))
    # board4 = board2.make_move_new(move2, check_legality=True) # Will panic
    # print(board4)

    print(board.generate_next_move())  # -3.21s
    board.reset_move_generator()  # Takes 0.16s

    board3 = rc.Board(
        "rnbqkbnr/ppp1pppp/8/3p4/2P1P3/8/PP1P1PPP/RNBQKBNR b KQkq - 0 2",
    )  # Black could capture either pawn # -21.89s
    print(list(board3.generate_legal_captures()))  # -7.75s
    print(list(board3.generate_legal_moves()))  # -17.01


def test_chess() -> None:  # noqa: PLR0915
    """Test the python-chess library."""
    color = chess.WHITE
    color2 = chess.COLORS[1]
    print(color)
    print(color2)
    print(not color2)
    print()

    pawn = chess.PAWN
    print(pawn)
    print()

    square = chess.Square(12)  # noqa: F841
    square2 = chess.parse_square("e2")
    square3 = chess.A3  # noqa: F841
    print(square2)
    print(chess.square_name(square2))
    print(chess.square_file(square2))
    print(chess.square_rank(square2))
    print()

    move = chess.Move(chess.Square(12), chess.Square(28))
    move2 = chess.Move.from_uci("e8d7")  # King move
    print(move2)
    print(move2.uci())
    print(move2.from_square)
    print(move2.to_square)
    print(move2.promotion)
    print()

    board = chess.Board()
    board2 = chess.Board("rnbqkbnr/ppp1p1pp/5p2/3p4/4P3/3P4/PPP1KPPP/RNBQ1BNR b kq - 1 3")
    print(board2)
    print(board2.fen())
    print(board2.halfmove_clock)
    print(board2.fullmove_number)
    print(board2.turn)
    print(board2.is_fifty_moves())
    print(board2.is_check())
    print(board.is_legal(move))
    print(board2.is_legal(move2))

    print(board.is_zeroing(move))  # Pawn move
    print(board2.is_zeroing(chess.Move.from_uci("e8d7")))  # King move

    print(board.piece_type_at(chess.E2))
    print(board.color_at(chess.E2))
    print(board.piece_at(chess.E4))
    print(board2.piece_at(chess.E2))

    board.push(move)  # Pawn move
    print(board)
    board.pop()
    board.push(chess.Move.from_uci("g1f3"))  # Horse move
    print(board)

    print(next(iter(board.legal_moves)))

    board3 = chess.Board(
        "rnbqkbnr/ppp1pppp/8/3p4/2P1P3/8/PP1P1PPP/RNBQKBNR b KQkq - 0 2",
    )  # Black could capture either pawn
    print(list(board3.generate_legal_captures()))
    print(list(board3.generate_legal_moves()))


if __name__ == "__main__":
    n = 1
    n = 100_000

    # time (v0.3.0):
    # real	0m39.834s
    # user	0m28.011s
    # sys	0m11.533s
    for _ in range(n):
        # Slower for simple functions and data types, much faster for complex functions
        test_rust_chess()  # Around 3.5 times faster python-chess :) (for this test)

    print("---------------------------------------")

    # # time:
    # # real	2m22.639s
    # # user	2m10.662s
    # # sys	0m10.804s
    # for _ in range(n):
    #     # Biggest slow down is creating with fen, displaying fen, legality, pushing moves, generating moves
    #     test_chess()

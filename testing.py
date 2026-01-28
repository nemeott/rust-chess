import chess

import rust_chess as rc

board = rc.Board()  # Create a board
move = rc.Move.from_uci("e2e4")  # Create move from UCI

print(list(board.generate_legal_moves()))

board.remove_move(move)  # FIXME

print(list(board.generate_legal_moves()))

# help(rc.Square.__doc__)

print(type(rc))

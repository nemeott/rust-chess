import chess

import rust_chess as rc

# TODO: Move to docstring for remove_move (add ... to testing to ignore long list)
board = rc.Board()  # Create a board
move = rc.Move.from_uci("e2e4")  # Create move from UCI

print(len(board.generate_legal_moves()))

rc.MoveGenerator.__next__

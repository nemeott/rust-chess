"""Simple perft test for rust-chess's move generation.

Current results as of rust-chess 0.3.0:
Depth 1:         20 nodes in  0.000s (1.89 Mnps)
Depth 2:        400 nodes in  0.000s (4.05 Mnps)
Depth 3:       8902 nodes in  0.002s (5.23 Mnps)
Depth 4:     197281 nodes in  0.030s (6.47 Mnps)
Depth 5:    4865609 nodes in  0.635s (7.66 Mnps)
Depth 6:  119060324 nodes in 15.406s (7.73 Mnps)
"""

from timeit import default_timer

import rust_chess as rc

DEPTH = 6


def perft_bulk(board: rc.Board, depth: int) -> int:
    """Perft using bulk counting (don't make move at depth 1).

    rust-chess cannot undo moves, so we have to create a new board every time.
    """
    nodes = 0

    moves = list(board.generate_legal_moves())

    if depth == 1:
        return len(moves)

    for move in moves:
        new_board = board.make_move_new(move)
        nodes += perft_bulk(new_board, depth - 1)

    return nodes


for depth in range(1, DEPTH + 1):
    board = rc.Board()

    start_time = default_timer()
    nodes = perft_bulk(board, depth)
    end_time = default_timer()
    duration = end_time - start_time
    mps = (nodes / duration / 1_000_000) if duration > 0 else 0

    print(f"Depth {depth}: {nodes:>10} nodes in {duration:>6.3f}s ({mps:.2f} Mnps)")

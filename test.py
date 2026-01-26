import chess

import rust_chess as rc

print(rc.B5.get_index())

print(rc.A6.get_index())
print(rc.C6.get_index())

print()

print(rc.B5.get_index() - rc.A6.get_index())
print(rc.B5.get_index() - rc.C6.get_index())

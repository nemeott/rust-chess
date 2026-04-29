use chess::{BitBoard, Board, ChessMove, MoveGen, Square};

#[test]
fn test_remove_move() {
    let board = Board::default();
    let mut iterable = MoveGen::new_legal(&board);
    assert_eq!(iterable.len(), 20);

    let move_to_remove = ChessMove::new(Square::D2, Square::D4, None);
    iterable.remove_move(move_to_remove);
    assert_eq!(iterable.len(), 19);

    let _new_move = iterable.next().unwrap();
    assert_eq!(iterable.len(), 18);

    assert!(!iterable.any(|x| x == move_to_remove));
}

#[test]
fn test_remove_first_move() {
    let board = Board::default();
    let mut iterable = MoveGen::new_legal(&board);
    assert_eq!(iterable.len(), 20);

    let move_to_remove = ChessMove::new(Square::A2, Square::A3, None);
    iterable.remove_move(move_to_remove);
    assert_eq!(iterable.len(), 19);

    let _new_move = iterable.next().unwrap();
    assert_eq!(iterable.len(), 18); // Assertion fails with 0 != 18

    assert!(!iterable.any(|x| x == move_to_remove));
}

#[test]
fn test_remove_second_move() {
    let board = Board::default();
    let mut iterable = MoveGen::new_legal(&board);
    assert_eq!(iterable.len(), 20);

    let move_to_remove = ChessMove::new(Square::A2, Square::A4, None);
    iterable.remove_move(move_to_remove);
    assert_eq!(iterable.len(), 19);

    let _new_move = iterable.next().unwrap();
    assert_eq!(iterable.len(), 18); // Assertion fails with 0 != 18

    assert!(!iterable.any(|x| x == move_to_remove));
}

#[test]
fn test_remove_first_second_moves_bitboard() {
    let board = Board::default();
    let mut iterable = MoveGen::new_legal(&board);
    assert_eq!(iterable.len(), 20);

    let mask = BitBoard::from_square(Square::A3);
    iterable.remove_mask(mask); // Removes 2 moves (A2-A3 and A2-A4)
    assert_eq!(iterable.len(), 18);

    let _new_move = iterable.next().unwrap();
    assert_eq!(iterable.len(), 17); // Assertion fails with 0 != 17

    let moves = iterable.collect::<Vec<ChessMove>>();
    assert!(!moves.contains(&ChessMove::new(Square::A2, Square::A3, None)));
    assert!(!moves.contains(&ChessMove::new(Square::A2, Square::A4, None)));
}

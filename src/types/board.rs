use std::str::FromStr;

use pyo3::{exceptions::PyValueError, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyclass_enum, gen_stub_pymethods};

use crate::types::{
    bitboard::PyBitboard, color::PyColor, r#move::{PyMove, PyMoveGenerator}, piece::{PAWN, PyPiece, PyPieceType}, square::PySquare
};

/// Board status enum class.
/// Represents the status of a chess board.
/// The status can be one of the following:
///     Ongoing, five-fold repetition, seventy-five moves, insufficient material, stalemate, or checkmate.
/// Supports comparison and equality.
///
#[gen_stub_pyclass_enum]
#[pyclass(name = "BoardStatus", frozen, eq, ord)]
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub(crate) enum PyBoardStatus {
    #[pyo3(name = "ONGOING")]
    Ongoing,
    #[pyo3(name = "FIVE_FOLD_REPETITION")]
    FiveFoldRepetition,
    #[pyo3(name = "SEVENTY_FIVE_MOVES")]
    SeventyFiveMoves,
    #[pyo3(name = "INSUFFICIENT_MATERIAL")]
    InsufficientMaterial,
    #[pyo3(name = "STALEMATE")]
    Stalemate,
    #[pyo3(name = "CHECKMATE")]
    Checkmate,
}

/// Board class.
/// Represents the state of a chess board.
///
#[gen_stub_pyclass]
#[pyclass(name = "Board")]
pub(crate) struct PyBoard {
    board: chess::Board,
    // move_gen: chess::MoveGen,
    move_gen: Py<PyMoveGenerator>, // Use a Py to be able to share between Python and Rust

    /// Get the halfmove clock.
    ///
    /// ```python
    /// >>> rust_chess.Board().halfmove_clock
    /// 0
    /// ```
    #[pyo3(get)]
    halfmove_clock: u8, // Halfmoves since last pawn move or capture

    /// Get the fullmove number.
    ///
    /// ```python
    /// >>> rust_chess.Board().fullmove_number
    /// 1
    /// ```
    #[pyo3(get)]
    fullmove_number: u8, // Fullmove number (increments after black moves)
}
// TODO: Incremental Zobrist hash

#[gen_stub_pymethods]
#[pymethods]
impl PyBoard {
    /// Create a new board from a FEN string, otherwise default to the starting position.
    ///
    /// ```python
    /// >>> rust_chess.Board()
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    /// >>> rust_chess.Board("rnbqkbnr/ppp1pppp/8/3p4/2P1P3/8/PP1P1PPP/RNBQKBNR b KQkq - 0 2")
    /// rnbqkbnr/ppp1pppp/8/3p4/2P1P3/8/PP1P1PPP/RNBQKBNR b KQkq - 0 2
    /// ```
    #[new]
    #[pyo3(signature = (fen = None))] // Default to None
    fn new(fen: Option<&str>) -> PyResult<Self> {
        match fen {
            // If no FEN string is provided, use the default starting position
            None => {
                let board = chess::Board::default();

                // We can assume the GIL is acquired, since this function is only called from Python
                let py = unsafe { Python::assume_gil_acquired() };

                // Create a new move generator using the chess crate
                let move_gen = Py::new(py, PyMoveGenerator(chess::MoveGen::new_legal(&board)))?;

                Ok(PyBoard {
                    board,
                    move_gen,
                    halfmove_clock: 0,
                    fullmove_number: 1,
                })
            }
            // Otherwise, parse the FEN string using the chess crate
            Some(fen_str) => PyBoard::from_fen(fen_str),
        }
    }

    /// Get the FEN string representation of the board.
    ///
    /// ```python
    /// >>> rust_chess.Board().get_fen()
    /// 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'
    /// ```
    #[inline]
    fn get_fen(&self) -> String {
        let base_fen = self.board.to_string();

        // 0: board, 1: player, 2: castling, 3: en passant, 4: halfmove clock, 5: fullmove number
        let mut parts: Vec<&str> = base_fen.split_whitespace().collect();

        // The chess crate does not track the halfmove clock and fullmove number correctly, so we need to add them manually.
        let halfmove_clock_str: String = self.halfmove_clock.to_string();
        let fullmove_number_str: String = self.fullmove_number.to_string();
        parts[4] = halfmove_clock_str.as_str();
        parts[5] = fullmove_number_str.as_str();

        parts.join(" ")
    }

    /// Get the FEN string representation of the board.
    ///
    /// ```python
    /// >>> print(rust_chess.Board())
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    /// ```
    #[inline]
    fn __str__(&self) -> String {
        self.get_fen()
    }

    /// Get the FEN string representation of the board.
    ///
    /// ```python
    /// >>> print(rust_chess.Board())
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    /// ```
    #[inline]
    fn __repr__(&self) -> String {
        self.get_fen()
    }

    /// Create a new board from a FEN string.
    ///
    /// ```python
    /// >>> rust_chess.Board.from_fen("rnbqkbnr/ppp1pppp/8/3p4/2P1P3/8/PP1P1PPP/RNBQKBNR b KQkq - 0 2")
    /// rnbqkbnr/ppp1pppp/8/3p4/2P1P3/8/PP1P1PPP/RNBQKBNR b KQkq - 0 2
    /// ```
    #[staticmethod]
    fn from_fen(fen: &str) -> PyResult<Self> {
        // Extract the halfmove clock and fullmove number from the FEN string
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() != 6 {
            return Err(PyValueError::new_err(
                "FEN string must have exactly 6 parts",
            ));
        }

        // Parse the halfmove clock and fullmove number
        let halfmove_clock = parts[4]
            .parse::<u8>()
            .map_err(|_| PyValueError::new_err("Invalid halfmove clock"))?;
        let fullmove_number = parts[5]
            .parse::<u8>()
            .map_err(|_| PyValueError::new_err("Invalid fullmove number"))?;

        // Parse the board using the chess crate
        let board = chess::Board::from_str(fen)
            .map_err(|e| PyValueError::new_err(format!("Invalid FEN: {e}")))?;

        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_gil_acquired() };

        // Create a new move generator using the chess crate
        let move_gen = Py::new(py, PyMoveGenerator(chess::MoveGen::new_legal(&board)))?;

        Ok(PyBoard {
            board,
            move_gen,
            halfmove_clock,
            fullmove_number,
        })
    }

    /// Get the current player to move.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.turn
    /// True
    /// >>> print(board.turn)
    /// WHITE
    /// ```
    #[getter]
    #[inline]
    fn get_turn(&self) -> PyColor {
        PyColor(self.board.side_to_move())
    }

    /// Get the en passant square, otherwise None.
    ///
    /// ```python
    /// >>> rust_chess.Board().en_passant
    ///
    /// >>> rust_chess.Board().en_passant == None
    /// True
    /// >>> rust_chess.Board("rnbqkbnr/pp2p1pp/2p5/3pPp2/5P2/8/PPPP2PP/RNBQKBNR w KQkq f6 0 4").en_passant
    /// f5
    /// ```
    #[getter]
    #[inline]
    fn get_en_passant(&self) -> Option<PySquare> {
        self.board.en_passant().map(PySquare)
    }

    /// Get the piece type on a square, otherwise None.
    /// Different than `get_piece_on` because it returns the piece type, which does not include color.
    ///
    /// ```python
    /// >>> rust_chess.Board().get_piece_type_on(rust_chess.A1)
    /// R
    /// >>> rust_chess.Board().get_piece_type_on(rust_chess.E8)
    /// K
    /// ```
    #[inline]
    fn get_piece_type_on(&self, square: PySquare) -> Option<PyPieceType> {
        // Get the piece on the square using the chess crate
        self.board.piece_on(square.0).map(PyPieceType)
    }

    /// Get the color of the piece on a square, otherwise None.
    ///
    /// ```python
    /// >>> rust_chess.Board().get_color_on(rust_chess.A1)
    /// True
    /// >>> print(rust_chess.Board().get_color_on(rust_chess.A1))
    /// WHITE
    /// >>> rust_chess.Board().get_color_on(rust_chess.E8)
    /// False
    #[inline]
    fn get_color_on(&self, square: PySquare) -> Option<PyColor> {
        // Get the color of the piece on the square using the chess crate
        self.board.color_on(square.0).map(PyColor)
    }

    /// Get the piece on a square (color-inclusive), otherwise None.
    /// Different than `get_piece_on` because it returns the piece, which includes color.
    ///
    /// ```python
    /// >>> rust_chess.Board().get_piece_on(rust_chess.A1)
    /// R
    /// >>> rust_chess.Board().get_piece_on(rust_chess.E8)
    /// k
    /// ```
    #[inline]
    fn get_piece_on(&self, square: PySquare) -> Option<PyPiece> {
        self.get_color_on(square).and_then(|color| {
            self.get_piece_type_on(square)
                .map(|piece_type| PyPiece { piece_type, color })
        })
    }

    /// Get the king square of a certain color
    #[inline]
    fn get_king_square(&self, color: PyColor) -> PySquare {
        PySquare(self.board.king_square(color.0))
    }

    /// Check if a move is a capture or a pawn move.
    /// Doesn't check legality.
    ///
    #[inline]
    fn is_zeroing(&self, chess_move: PyMove) -> bool {
        self.get_piece_type_on(chess_move.get_source()) == Some(PAWN) // Pawn move
        || self.get_piece_type_on(chess_move.get_dest()).is_some() // Capture (moving piece onto other piece)
    }

    /// Check if the move is legal (supposedly very slow according to the chess crate).
    /// Use this function for moves not generated by the move generator.
    /// `is_legal_quick` is faster for moves generated by the move generator.
    ///
    /// ```python
    /// >>> move = rust_chess.Move("e2e4")
    /// >>> rust_chess.Board().is_legal_move(move)
    /// True
    /// >>> move2 = rust_chess.Move("e2e5")
    /// >>> rust_chess.Board().is_legal_move(move2)
    /// False
    /// ```
    #[inline]
    fn is_legal_move(&self, chess_move: PyMove) -> bool {
        // Check if the move is legal using the chess crate
        chess::Board::legal(&self.board, chess_move.0)
    }

    // TODO: is_legal_quick

    /// Make a null move onto a new board.
    /// Returns None if the current player is in check.
    ///
    #[inline]
    fn make_null_move_new(&self) -> PyResult<Option<Self>> {
        // Get the new board using the chess crate
        let Some(new_board) = self.board.null_move() else {
            return Ok(None);
        };

        // Increment the halfmove clock
        let halfmove_clock: u8 = self.halfmove_clock + 1;

        // Increment fullmove number if black moves
        let fullmove_number: u8 = if self.board.side_to_move() == chess::Color::Black {
            self.fullmove_number + 1
        } else {
            self.fullmove_number
        };

        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_gil_acquired() };

        // Create a new move generator using the chess crate
        let move_gen = Py::new(py, PyMoveGenerator(chess::MoveGen::new_legal(&new_board)))?;

        Ok(Some(PyBoard {
            board: new_board,
            move_gen,
            halfmove_clock,
            fullmove_number,
        }))
    }

    /// Make a move onto a new board
    ///
    #[pyo3(signature = (chess_move, check_legality = false))]
    fn make_move_new(&self, chess_move: PyMove, check_legality: bool) -> PyResult<Self> {
        // If we are checking legality, check if the move is legal
        if check_legality && !self.is_legal_move(chess_move) {
            return Err(PyValueError::new_err("Illegal move"));
        }

        // Make the move onto a new board using the chess crate
        let new_board: chess::Board = self.board.make_move_new(chess_move.0);

        // Reset the halfmove clock if the move zeroes (is a capture or pawn move and therefore "zeroes" the halfmove clock)
        let halfmove_clock: u8 = if self.is_zeroing(chess_move) {
            0
        } else {
            self.halfmove_clock + 1
        };

        // Increment fullmove number if black moves
        let fullmove_number: u8 = if self.board.side_to_move() == chess::Color::Black {
            self.fullmove_number + 1
        } else {
            self.fullmove_number
        };

        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_gil_acquired() };

        // Create a new move generator using the chess crate
        let move_gen = Py::new(py, PyMoveGenerator(chess::MoveGen::new_legal(&new_board)))?;

        Ok(PyBoard {
            board: new_board,
            move_gen,
            halfmove_clock,
            fullmove_number,
        })
    }

    /// Make a move on the current board
    ///
    #[pyo3(signature = (chess_move, check_legality = false))]
    fn make_move(&mut self, chess_move: PyMove, check_legality: bool) -> PyResult<()> {
        // If we are checking legality, check if the move is legal
        if check_legality && !self.is_legal_move(chess_move) {
            return Err(PyValueError::new_err("Illegal move"));
        }

        // Make the move onto a new board using the chess crate
        let temp_board: chess::Board = self.board.make_move_new(chess_move.0);

        // Reset the halfmove clock if the move zeroes (is a capture or pawn move and therefore "zeroes" the halfmove clock)
        self.halfmove_clock = if self.is_zeroing(chess_move) {
            0
        } else {
            self.halfmove_clock + 1
        };

        // Increment fullmove number if black moves
        if self.board.side_to_move() == chess::Color::Black {
            self.fullmove_number += 1;
        }

        // Update the current board
        self.board = temp_board;

        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_gil_acquired() };

        // Create a new move generator using the chess crate
        self.move_gen = Py::new(py, PyMoveGenerator(chess::MoveGen::new_legal(&temp_board)))?;

        Ok(())
    }

    /// Get the bitboard of the side to move's pinned pieces
    #[inline]
    fn get_pinned_bitboard(&self) -> PyBitboard {
        PyBitboard(*self.board.pinned())
    }

    /// Get the bitboard of the pieces putting the side to move in check
    #[inline]
    fn get_checkers_bitboard(&self) -> PyBitboard {
        PyBitboard(*self.board.checkers())
    }

    /// Get the bitboard of all the pieces
    #[inline]
    fn get_all_bitboard(&self) -> PyBitboard {
        PyBitboard(*self.board.combined())
    }

    /// Get the bitboard of all the pieces of a certain color
    #[inline]
    fn get_color_bitboard(&self, color: PyColor) -> PyBitboard {
        PyBitboard(*self.board.color_combined(color.0))
    }

    /// Get the bitboard of all the pieces of a certain type
    #[inline]
    fn get_piece_type_bitboard(&self, piece_type: PyPieceType) -> PyBitboard {
        PyBitboard(*self.board.pieces(piece_type.0))
    }

    /// Get the bitboard of all the pieces of a certain color and type
    #[inline]
    fn get_piece_bitboard(&self, piece: PyPiece) -> PyBitboard {
        PyBitboard(self.board.pieces(piece.piece_type.0) & self.board.color_combined(piece.color.0))
    }

    // TODO: set_iterator_mask, will have to implement PyBitboard
    // TODO: remove_mask

    // Fixme
    // /// Get the number of moves remaining in the move generator.
    // /// This is the number of remaining moves that can be generated.
    // /// The default mask is all legal moves.
    // ///
    // #[inline]
    // fn get_moves_remaining(&self) -> usize {
    //     // We can assume the GIL is acquired, since this function is only called from Python
    //     let py = unsafe { Python::assume_gil_acquired() };
    //
    //     // Get the length of the move generator
    //     self.move_gen.borrow(py).0.len()
    // }

    /// Remove a move from the move generator.
    /// Prevents the move from being generated.
    /// Useful if you already have a certain move and don't need to generate it again.
    ///
    #[inline]
    fn remove_move(&mut self, chess_move: PyMove) {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_gil_acquired() };

        // Remove the move from the generator
        self.move_gen.borrow_mut(py).0.remove_move(chess_move.0);
    }

    /// Reset the move generator for the current board
    #[inline]
    fn reset_move_generator(&mut self) -> PyResult<()> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_gil_acquired() };

        // Create a new move generator using the chess crate
        self.move_gen = Py::new(py, PyMoveGenerator(chess::MoveGen::new_legal(&self.board)))?;

        Ok(())
    }

    /// Get the next remaining move of the generator.
    /// Updates the move generator to the next move.
    /// Unless the mask is set, this will return the next legal move by default.
    ///
    #[inline]
    fn next_move(&mut self) -> Option<PyMove> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_gil_acquired() };

        // Get the next move from the generator
        self.move_gen.borrow_mut(py).__next__()
    }

    /// Generate the next remaining legal moves for the current board.
    /// Exhausts the move generator if fully iterated over.
    /// Updates the move generator.
    ///
    #[inline]
    fn generate_legal_moves(&mut self) -> Py<PyMoveGenerator> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_gil_acquired() };

        // Set the iterator mask to everything (check all legal moves)
        self.move_gen
            .borrow_mut(py)
            .0
            .set_iterator_mask(!chess::EMPTY);

        // Share ownership with Python
        self.move_gen.clone_ref(py)
    }

    #[inline]
    /// Generate the next remaining legal captures for the current board.
    /// Exhausts the move generator if fully iterated over.
    /// Updates the move generator.
    ///
    fn generate_legal_captures(&mut self) -> Py<PyMoveGenerator> {
        // Get the mask of enemy‐occupied squares
        let targets_mask = self.board.color_combined(!self.board.side_to_move());

        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_gil_acquired() };

        // Set the iterator mask to the targets mask (check all legal captures [moves onto enemy pieces])
        self.move_gen
            .borrow_mut(py)
            .0
            .set_iterator_mask(*targets_mask);

        // Share ownership with Python
        self.move_gen.clone_ref(py)
    }

    /// Checks if the side to move has insufficient material to checkmate the opponent.
    /// The cases where this is true are:
    ///     1. K vs K
    ///     2. K vs K + N
    ///     3. K vs K + B
    ///     4. K + B vs K + B with the bishops on the same color.
    ///
    /// ```python
    /// >>> rust_chess.Board().is_insufficient_material()
    /// False
    /// >>> rust_chess.Board("4k3/8/8/8/8/8/8/4K3 w - - 0 1").is_insufficient_material() # K vs K
    /// True
    /// >>> rust_chess.Board("4k3/8/8/8/5N2/8/8/4K3 w - - 0 1").is_insufficient_material() # K vs K + N
    /// True
    /// >>> rust_chess.Board("4k3/8/8/8/5B2/8/8/4K3 w - - 0 1").is_insufficient_material() # K vs K + B
    /// True
    /// >>> rust_chess.Board("4k3/8/8/5b2/5B2/8/8/4K3 w - - 0 1").is_insufficient_material() # K + B vs K + B different color
    /// False
    /// >>> rust_chess.Board("4k3/8/5b2/8/5B2/8/8/4K3 w - - 0 1").is_insufficient_material() # K + B vs K + B same color
    /// True
    /// ```
    #[inline]
    fn is_insufficient_material(&self) -> bool {
        let kings = self.board.pieces(chess::Piece::King);

        // Get the bitboards of the white and black pieces without the kings
        let white_bb = self.board.color_combined(chess::Color::White) & !kings;
        let black_bb = self.board.color_combined(chess::Color::Black) & !kings;
        let combined_bb = white_bb | black_bb;

        // King vs King: Combined bitboard minus kings is empty
        if combined_bb == chess::EMPTY {
            return true;
        }

        let remaining_num_pieces = combined_bb.popcnt();

        if remaining_num_pieces <= 2 {
            let knights = self.board.pieces(chess::Piece::Knight);
            let bishops = self.board.pieces(chess::Piece::Bishop);

            // King vs King + Knight/Bishop: Combined bitboard minus kings and knight/bishop is empty
            if remaining_num_pieces == 1 && combined_bb & !(knights | bishops) == chess::EMPTY {
                return true;
            } else if *knights == chess::EMPTY {
                // Only bishops left
                let white_bishops = bishops & white_bb;
                let black_bishops = bishops & black_bb;

                if white_bishops != chess::EMPTY && black_bishops != chess::EMPTY // Both sides have a bishop
                    // King + Bishop vs King + Bishop same color: White and black bishops are on the same color square
                    && PySquare(white_bishops.to_square()).get_color() == PySquare(black_bishops.to_square()).get_color()
                {
                    return true;
                }
            }
        }
        false
    }

    /// Checks if the halfmoves since the last pawn move or capture is >= 100
    /// and the game is ongoing (not checkmate or stalemate).
    ///
    /// ```python
    /// >>> rust_chess.Board().is_fifty_moves
    /// False
    /// >>> rust_chess.Board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 50 1").is_fifty_moves()
    /// True
    /// ```
    #[inline]
    fn is_fifty_moves(&self) -> bool {
        self.halfmove_clock >= 100 && self.board.status() == chess::BoardStatus::Ongoing
    }

    /// Checks if the halfmoves since the last pawn move or capture is >= 150
    /// and the game is ongoing (not checkmate or stalemate).
    ///
    #[inline]
    fn is_seventy_five_moves(&self) -> bool {
        self.halfmove_clock >= 150 && self.board.status() == chess::BoardStatus::Ongoing
    }

    // TODO: Check threefold and fivefold repetition

    /// Checks if the game is in a fivefold repetition.
    /// TODO: Currently not implementable due to no storage of past moves
    #[inline]
    fn is_fivefold_repetition(&self) -> bool {
        false
    }

    /// Checks if the side to move is in check.
    ///
    /// ```python
    /// >>> rust_chess.Board().is_check
    /// False
    /// >>> rust_chess.Board("rnb1kbnr/pppp1ppp/4p3/8/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3").is_check()
    /// True
    /// ```
    #[inline]
    fn is_check(&self) -> bool {
        *self.board.checkers() != chess::EMPTY
    }

    /// Checks if the side to move is in stalemate
    #[inline]
    fn is_stalemate(&self) -> bool {
        self.board.status() == chess::BoardStatus::Stalemate
    }

    /// Checks if the side to move is in checkmate
    #[inline]
    fn is_checkmate(&self) -> bool {
        self.board.status() == chess::BoardStatus::Checkmate
    }

    /// Get the status of the board
    #[inline]
    fn get_status(&self) -> PyBoardStatus {
        let status = self.board.status();
        match status {
            chess::BoardStatus::Checkmate => PyBoardStatus::Checkmate,
            chess::BoardStatus::Stalemate => PyBoardStatus::Stalemate,
            chess::BoardStatus::Ongoing => {
                if self.is_insufficient_material() {
                    PyBoardStatus::InsufficientMaterial
                } else if self.is_seventy_five_moves() {
                    PyBoardStatus::SeventyFiveMoves
                } else if self.is_fivefold_repetition() {
                    PyBoardStatus::FiveFoldRepetition
                } else {
                    PyBoardStatus::Ongoing
                }
            }
        }
    }
}

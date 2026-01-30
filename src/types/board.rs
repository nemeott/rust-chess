use std::str::FromStr;

use pyo3::{exceptions::PyValueError, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyclass_enum, gen_stub_pymethods};

use crate::types::{
    bitboard::PyBitboard,
    color::PyColor,
    r#move::{PyMove, PyMoveGenerator},
    piece::{PyPiece, PyPieceType},
    square::PySquare,
};

// TODO: Comparision and partial ord

/// Board status enum class.
/// Represents the status of a chess board.
/// The status can be one of the following:
///     Ongoing, seventy-five moves, five-fold repetition, insufficient material, stalemate, or checkmate.
/// Supports comparison and equality.
///
#[gen_stub_pyclass_enum]
#[pyclass(name = "BoardStatus", frozen, eq, ord)]
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub(crate) enum PyBoardStatus {
    #[pyo3(name = "ONGOING")]
    Ongoing,
    #[pyo3(name = "SEVENTY_FIVE_MOVES")]
    SeventyFiveMoves,
    #[pyo3(name = "FIVE_FOLD_REPETITION")]
    FiveFoldRepetition,
    #[pyo3(name = "INSUFFICIENT_MATERIAL")]
    InsufficientMaterial,
    #[pyo3(name = "STALEMATE")]
    Stalemate,
    #[pyo3(name = "CHECKMATE")]
    Checkmate,
}

// TODO: Comparison and partial ord

/// Board class.
/// Represents the state of a chess board.
///
#[gen_stub_pyclass]
#[pyclass(name = "Board")]
pub(crate) struct PyBoard {
    board: chess::Board,

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
                let py = unsafe { Python::assume_attached() };

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
        let base_parts: Vec<&str> = base_fen.split_whitespace().collect();

        // The chess crate doesn't handle the halfmove and fullmove values so we need to do it ourselves
        format!(
            "{} {} {} {} {} {}",
            base_parts[0],        // board
            base_parts[1],        // player
            base_parts[2],        // castling
            base_parts[3],        // en passant
            self.halfmove_clock,  // halfmove clock
            self.fullmove_number, // fullmove number
        )
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
        let py = unsafe { Python::assume_attached() };

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
    ///
    /// >>> board.make_move(rust_chess.Move("e2e4"))
    /// >>> board.turn
    /// False
    /// >>> print(board.turn)
    /// BLACK
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
    ///
    /// >>> board = rust_chess.Board("rnbqkbnr/pp2p1pp/2p5/3pPp2/5P2/8/PPPP2PP/RNBQKBNR w KQkq f6 0 4")
    /// >>> board.en_passant
    /// f6
    /// ```
    #[getter]
    #[inline]
    fn get_en_passant(&self) -> Option<PySquare> {
        // The Rust chess crate doesn't actually computer this right, it returns the square that the pawn was moved to.
        // The actual en passant square is the one that one can move to that would cause en passant.
        // TLDR: The actual en passant square is one above or below the one returned by the chess crate.
        self.board.en_passant().map(|sq| {
            if self.board.side_to_move() == chess::Color::White {
                PySquare(sq.up().unwrap())
            } else {
                PySquare(sq.down().unwrap())
            }
        })
    }

    /// Check if a move is en passant.
    ///
    /// Assumes the move is legal.
    ///
    /// ```python
    /// >>> rust_chess.Board().is_en_passant(rust_chess.Move("e2e4"))
    /// False
    ///
    /// >>> board = rust_chess.Board("rnbqkbnr/pp2p1pp/2p5/3pPp2/5P2/8/PPPP2PP/RNBQKBNR w KQkq f6 0 4")
    /// >>> board.is_en_passant(rust_chess.Move("e5f6"))
    /// True
    /// ```
    #[inline]
    fn is_en_passant(&self, chess_move: PyMove) -> bool {
        let source = chess_move.0.get_source();
        let dest = chess_move.0.get_dest();

        // The Rust chess crate doesn't actually computer this right, it returns the square that the pawn was moved to.
        // The actual en passant square is the one that one can move to that would cause en passant.
        // TLDR: The actual en passant square is one above or below the one returned by the chess crate.
        let ep_square = self.board.en_passant().and_then(|sq| {
            if self.board.side_to_move() == chess::Color::White {
                sq.up()
            } else {
                sq.down()
            }
        });

        ep_square.is_some_and(|ep_sq| ep_sq == dest) // Use our en passant square function since it is accurate
            && self.board.piece_on(source).is_some_and(|p| p == chess::Piece::Pawn) // Moving pawn
            && {
                // Moving diagonally
                let diff = (dest.to_index() as i8 - source.to_index() as i8).abs();
                diff == 7 || diff == 9
            }
            && self.board.piece_on(dest).is_none() // Target square is empty
    }

    /// Check if a move is a capture.
    ///
    /// Assumes the move is legal.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.is_capture(rust_chess.Move("e2e4"))
    /// False
    /// >>> board.make_move(rust_chess.Move("e2e4"))
    ///
    /// >>> board.make_move(rust_chess.Move("d7d5"))
    /// >>> board.is_capture(rust_chess.Move("e4d5"))
    /// True
    ///
    /// >>> ep_board = rust_chess.Board("rnbqkbnr/pp2p1pp/2p5/3pPp2/5P2/8/PPPP2PP/RNBQKBNR w KQkq f6 0 4")
    /// >>> ep_board.is_capture(rust_chess.Move("e5f6"))
    /// True
    /// ```
    #[inline]
    fn is_capture(&self, chess_move: PyMove) -> bool {
        self.board.piece_on(chess_move.0.get_dest()).is_some() // Capture (moving piece onto other piece)
            || self.is_en_passant(chess_move) // Or the move is en passant (also a capture)
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
    /// >>> print(rust_chess.Board().get_color_on(rust_chess.E8))
    /// BLACK
    /// ```
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

    /// Get the king square of a color
    ///
    /// ```python
    /// >>> rust_chess.Board().get_king_square(rust_chess.WHITE)
    /// e1
    /// >>> rust_chess.Board().get_king_square(rust_chess.BLACK)
    /// e8
    /// ```
    #[inline]
    fn get_king_square(&self, color: PyColor) -> PySquare {
        PySquare(self.board.king_square(color.0))
    }

    /// Check if a move is a capture or a pawn move.
    /// "Zeros" the halfmove clock (sets it to 0).
    ///
    /// Doesn't check legality.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.is_zeroing(rust_chess.Move("e2e4"))
    /// True
    /// >>> board.make_move(rust_chess.Move("e2e4"))
    ///
    /// >>> board.is_zeroing(rust_chess.Move("g8f6"))
    /// False
    /// >>> board.make_move(rust_chess.Move("d7d5"))
    ///
    /// >>> board.is_zeroing(rust_chess.Move("e4d5"))
    /// True
    /// ```
    #[inline]
    fn is_zeroing(&self, chess_move: PyMove) -> bool {
        self.board.piece_on(chess_move.0.get_source()).is_some_and(|p| p == chess::Piece::Pawn) // Pawn move
        || self.board.piece_on(chess_move.0.get_dest()).is_some() // Capture (moving piece onto other piece)
    }

    /// Check if the move is legal (supposedly very slow according to the chess crate).
    /// Use this function for moves not generated by the move generator.
    /// `is_legal_quick` is faster for moves generated by the move generator.
    ///
    /// ```python
    /// >>> move = rust_chess.Move("e2e4")
    /// >>> rust_chess.Board().is_legal_move(move)
    /// True
    /// >>> ill_move = rust_chess.Move("e2e5")
    /// >>> rust_chess.Board().is_legal_move(ill_move)
    /// False
    /// ```
    #[inline]
    fn is_legal_move(&self, chess_move: PyMove) -> bool {
        // Check if the move is legal using the chess crate
        chess::Board::legal(&self.board, chess_move.0)
    }

    /// Check if the move generated by the generator is legal.
    /// Only use this function for moves generated by the move generator.
    /// You would want to use this when you have a psuedo-legal move (guarenteed by the generator).
    /// Slightly faster than using `is_legal_move` since it doesn't have to check as much stuff.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>>
    /// ```
    #[inline]
    fn is_legal_generator_move(&self, chess_move: PyMove) -> bool {
        chess::MoveGen::legal_quick(&self.board, chess_move.0)
    }

    // FIXME

    // TODO: make_null_move (would require move history to undo (probably?))

    /// Make a null move onto a new board.
    /// Returns None if the current player is in check.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> print(board)
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    /// >>> new_board = board.make_null_move_new()
    /// >>> print(new_board)
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 1 1
    ///
    /// >>> board = rust_chess.Board("rnbqkbnr/ppppp1pp/5p2/7Q/8/4P3/PPPP1PPP/RNB1KBNR b KQkq - 1 2")
    /// >>> new_board = board.make_null_move_new()
    /// >>> print(new_board)
    /// None
    /// ```
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
        let py = unsafe { Python::assume_attached() };

        // Create a new move generator using the chess crate
        let move_gen = Py::new(py, PyMoveGenerator(chess::MoveGen::new_legal(&new_board)))?;

        Ok(Some(PyBoard {
            board: new_board,
            move_gen,
            halfmove_clock,
            fullmove_number,
        }))
    }

    /// Make a move onto the current board.
    ///
    /// Defaults to checking move legality, unless the optional legality parameter is `False`.
    /// Not checking move legality will provide a slight performance boost, but crash if the move is invalid.
    /// Checking legality will return an error if the move is illegal.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.make_move(rust_chess.Move("e2e4"))
    /// >>> print(board)
    /// rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1
    /// ```
    #[pyo3(signature = (chess_move, check_legality = true))]
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

        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        // Create a new move generator using the chess crate
        self.move_gen = Py::new(py, PyMoveGenerator(chess::MoveGen::new_legal(&temp_board)))?;

        // Update the current board
        self.board = temp_board;

        Ok(())
    }

    /// Make a move onto a new board.
    ///
    /// Defaults to checking move legality, unless the optional legality parameter is `False`.
    /// Not checking move legality will provide a slight performance boost, but crash if the move is invalid.
    /// Checking legality will return an error if the move is illegal.
    ///
    /// ```python
    /// >>> old_board = rust_chess.Board()
    /// >>> new_board = old_board.make_move_new(rust_chess.Move("e2e4"))
    /// >>> print(new_board)
    /// rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1
    /// >>> print(old_board)
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    /// ```
    #[pyo3(signature = (chess_move, check_legality = true))]
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
        let py = unsafe { Python::assume_attached() };

        // Create a new move generator using the chess crate
        let move_gen = Py::new(py, PyMoveGenerator(chess::MoveGen::new_legal(&new_board)))?;

        Ok(PyBoard {
            board: new_board,
            move_gen,
            halfmove_clock,
            fullmove_number,
        })
    }

    /// Get the bitboard of the side to move's pinned pieces.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.get_pinned_bitboard().popcnt()
    /// 0
    ///
    /// board.make_move(rust_chess.Move("e2e4"))
    /// board.make_move(rust_chess.Move("d7d5"))
    /// board.make_move(rust_chess.Move("d1h5"))
    /// FIXME
    /// >>> board.get_pinned_bitboard().popcnt()
    /// 1
    /// >>> board.get_pinned_bitboard()
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . X . .
    /// . . . . . . . .
    /// ```
    #[inline]
    fn get_pinned_bitboard(&self) -> PyBitboard {
        PyBitboard(*self.board.pinned())
    }

    /// Get the bitboard of the pieces putting the side to move in check.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.get_checkers_bitboard().popcnt()
    /// 0
    ///
    /// board.make_move(rust_chess.Move("e2e4"))
    /// board.make_move(rust_chess.Move("f2f3"))
    /// board.make_move(rust_chess.Move("d1h5"))
    /// FIXME
    /// >>> board.get_checkers_bitboard().popcnt()
    /// 1
    /// >>> board.get_checkers_bitboard()
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . X
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// ```
    #[inline]
    fn get_checkers_bitboard(&self) -> PyBitboard {
        PyBitboard(*self.board.checkers())
    }

    /// Get the bitboard of all the pieces of a certain color.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.make_move(rust_chess.Move("e2e4"))
    /// >>> board.get_color_bitboard(rust_chess.WHITE).popcnt()
    /// 16
    /// >>> board.get_color_bitboard(rust_chess.WHITE)
    /// X X X X X X X X
    /// X X X X . X X X
    /// . . . . . . . .
    /// . . . . X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// ```
    #[inline]
    fn get_color_bitboard(&self, color: PyColor) -> PyBitboard {
        PyBitboard(*self.board.color_combined(color.0))
    }

    /// Get the bitboard of all the pieces of a certain type.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.make_move(rust_chess.Move("e2e4"))
    /// >>> board.get_piece_type_bitboard(rust_chess.PAWN).popcnt()
    /// 16
    /// >>> board.get_piece_type_bitboard(rust_chess.PAWN)
    /// . . . . . . . .
    /// X X X X . X X X
    /// . . . . . . . .
    /// . . . . X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// X X X X X X X X
    /// . . . . . . . .
    /// ```
    #[inline]
    fn get_piece_type_bitboard(&self, piece_type: PyPieceType) -> PyBitboard {
        PyBitboard(*self.board.pieces(piece_type.0))
    }

    /// Get the bitboard of all the pieces of a certain color and type.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.make_move(rust_chess.Move("e2e4"))
    /// >>> board.get_piece_bitboard(rust_chess.WHITE_PAWN).popcnt()
    /// 8
    /// >>> board.get_piece_bitboard(rust_chess.WHITE_PAWN)
    /// . . . . . . . .
    /// X X X X . X X X
    /// . . . . . . . .
    /// . . . . X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// ```
    #[inline]
    fn get_piece_bitboard(&self, piece: PyPiece) -> PyBitboard {
        PyBitboard(self.board.pieces(piece.piece_type.0) & self.board.color_combined(piece.color.0))
    }

    /// Get the bitboard of all the pieces.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.make_move(rust_chess.Move("e2e4"))
    /// >>> board.get_all_bitboard().popcnt()
    /// 32
    /// >>> board.get_all_bitboard()
    /// X X X X X X X X
    /// X X X X . X X X
    /// . . . . . . . .
    /// . . . . X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// X X X X X X X X
    /// X X X X X X X X
    /// ```
    #[inline]
    fn get_all_bitboard(&self) -> PyBitboard {
        PyBitboard(*self.board.combined())
    }

    /// Get the number of moves remaining in the move generator.
    /// This is the number of remaining moves that can be generated.
    /// Does not consume any iterations.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.get_generator_num_remaining()
    /// 20
    /// >>> next(board.generate_legal_moves())
    /// Move(a2, a3, None)
    /// >>> board.get_generator_num_remaining()
    /// 19
    /// ```
    #[inline]
    fn get_generator_num_remaining(&self) -> usize {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };
        self.move_gen.borrow(py).__len__()
    }

    /// Reset the move generator for the current board.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> len(board.generate_legal_moves())
    /// 20
    /// >>> list(board.generate_legal_moves())
    /// [Move(a2, a3, None), Move(a2, a4, None), ..., Move(g1, h3, None)]
    /// >>> len(board.generate_legal_moves())
    /// 0
    /// >>> board.reset_move_generator()
    /// >>> len(board.generate_legal_moves())
    /// 20
    /// ```
    #[inline]
    fn reset_move_generator(&mut self) -> PyResult<()> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        // Create a new move generator using the chess crate
        self.move_gen = Py::new(py, PyMoveGenerator(chess::MoveGen::new_legal(&self.board)))?;

        Ok(())
    }

    /// Remove a move from the move generator.
    /// Prevents the move from being generated.
    /// Updates the generator mask to exclude the move.
    /// Useful if you already have a certain move and don't need to generate it again.
    ///
    /// **WARNING**: using any form of `legal_move` or `legal_capture` generation
    /// will set the generator mask, invalidating any previous removals by this function.
    /// This also applies to setting the generator mask manually.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> len(board.generate_moves())  # Legal moves by default
    /// 20
    /// >>> move = rust_chess.Move("a2a3")
    /// >>> board.remove_generator_move(move)
    /// >>> len(board.generate_moves())
    /// 19
    /// >>> move in board.generate_moves()  # Consumes generator moves
    /// False
    /// >>> len(board.generate_moves())
    /// 0
    /// ```
    #[inline]
    fn remove_generator_move(&mut self, chess_move: PyMove) {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };
        self.move_gen.borrow_mut(py).0.remove_move(chess_move.0);
    }

    /// Sets the generator mask for the move generator.
    /// The mask is a bitboard that indicates what landing squares to generate moves for.
    /// Only squares in the mask will be considered when generating moves.
    /// See `remove_generator_mask` for the inverse (never generate bitboard moves).
    ///
    /// Moves that have already been iterated over will not be generated again, regardless of the mask value.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> len(board.generate_moves())
    /// 20
    /// >>> board.set_generator_mask(rust_chess.E4.to_bitboard())
    /// >>> len(board.generate_moves())
    /// 1
    /// >>> board.generate_next_move()
    /// Move(e2, e4, None)
    /// ```
    #[inline]
    fn set_generator_mask(&mut self, mask: PyBitboard) {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };
        self.move_gen.borrow_mut(py).0.set_iterator_mask(mask.0);
    }

    /// Removes the generator mask from the move generator.
    /// The mask is a bitboard that indicates what landing squares *not* to generate moves for.
    /// Only squares not in the mask will be considered when generating moves.
    /// See `set_generator_mask` for the inverse (only generate bitboard moves).
    ///
    /// You can remove moves, and then generate over all legal moves for example without regenerating the removed moves.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> len(board.generate_moves())
    /// 20
    /// >>> board.remove_generator_mask(rust_chess.E4.to_bitboard())
    /// >>> len(board.generate_moves())
    /// 19
    /// >>> rust_chess.Move("e2e4") in board.generate_moves()
    /// False
    /// >>> len(board.generate_moves())
    /// 0
    /// ```
    #[inline]
    fn remove_generator_mask(&mut self, mask: PyBitboard) {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };
        self.move_gen.borrow_mut(py).0.remove_mask(mask.0);
    }

    /// Get the next remaining move in the generator.
    /// Updates the move generator to the next move.
    ///
    /// Unless the mask has been set, this will return the next legal move by default.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> len(board.generate_moves())
    /// 20
    /// >>> board.remove_generator_move(rust_chess.Move("a2a3"))
    /// >>> len(board.generate_moves())
    /// 19
    /// >>> board.generate_next_move()
    /// Move(a2, a4, None)
    /// >>> len(board.generate_moves())
    /// 18
    /// ```
    #[inline]
    fn generate_next_move(&mut self) -> Option<PyMove> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };
        self.move_gen.borrow_mut(py).__next__()
    }

    /// Get the next remaining legal move in the generator.
    /// Updates the move generator to the next legal move.
    ///
    /// Updates the generator mask to all legal moves.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> len(board.generate_legal_moves())
    /// 20
    /// >>> board.generate_next_legal_move()
    /// Move(a2, a3, None)
    /// >>> len(board.generate_legal_moves())
    /// 19
    /// ```
    #[inline]
    fn generate_next_legal_move(&mut self) -> Option<PyMove> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        // Set the iterator mask to everything (check all legal moves)
        self.move_gen
            .borrow_mut(py)
            .0
            .set_iterator_mask(!chess::EMPTY);

        self.move_gen.borrow_mut(py).__next__()
    }

    /// Get the next remaining legal capture in the generator.
    /// Updates the move generator to the next move.
    ///
    /// Updates the generator mask to the enemy's squares (all legal captures).
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.make_move(rust_chess.Move("e2e4"))
    /// >>> board.make_move(rust_chess.Move("d7d5"))
    /// >>> len(board.generate_legal_moves())
    /// 31
    /// >>> len(board.generate_legal_captures())
    /// 1
    /// >>> board.generate_next_legal_capture()
    /// Move(e4, d5, None)
    /// >>> len(board.generate_legal_captures())
    /// 0
    /// ```
    #[inline]
    fn generate_next_legal_capture(&mut self) -> Option<PyMove> {
        // Get the mask of enemy‐occupied squares
        let targets_mask = self.board.color_combined(!self.board.side_to_move());

        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        // Set the iterator mask to the targets mask (check all legal captures [moves onto enemy pieces])
        self.move_gen
            .borrow_mut(py)
            .0
            .set_iterator_mask(*targets_mask);

        self.move_gen.borrow_mut(py).__next__()
    }
    
    // TODO: Generate moves_list (PyList<PyMove>)

    /// Generate the next remaining moves for the current board.
    /// Exhausts the move generator if fully iterated over.
    /// Updates the move generator.
    ///
    /// Unless the generator mask is set, this will generate the next legal moves by default.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> len(board.generate_moves())
    /// 20
    /// >>> board.set_generator_mask(rust_chess.Bitboard(11063835754496))
    /// >>> len(board.generate_moves())
    /// 3
    /// >>> list(board.generate_moves())
    /// [Move(b2, b3, None), Move(d2, d3, None), Move(e2, e4, None)]
    /// >>> len(board.generate_moves())
    /// 0
    /// ```
    #[inline]
    fn generate_moves(&mut self) -> Py<PyMoveGenerator> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        // Share ownership with Python
        self.move_gen.clone_ref(py)
    }

    /// Generate the next remaining legal moves for the current board.
    /// Exhausts the move generator if fully iterated over.
    /// Updates the move generator.
    ///
    /// Will not iterate over the same moves already generated by `generate_legal_captures`.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> len(board.generate_legal_moves())
    /// 20
    /// >>> list(board.generate_legal_moves())
    /// [Move(a2, a3, None), Move(a2, a4, None), ..., Move(g1, h3, None)]
    /// >>> len(board.generate_legal_moves())
    /// 0
    /// >>> board.reset_move_generator()
    /// >>> len(board.generate_legal_moves())
    /// 20
    /// ```
    #[inline]
    fn generate_legal_moves(&mut self) -> Py<PyMoveGenerator> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        // Set the iterator mask to everything (check all legal moves)
        self.move_gen
            .borrow_mut(py)
            .0
            .set_iterator_mask(!chess::EMPTY);

        // Share ownership with Python
        self.move_gen.clone_ref(py)
    }

    /// Generate the next remaining legal captures for the current board.
    /// Exhausts the move generator if fully iterated over.
    /// Updates the move generator.
    ///
    /// Can iterate over legal captures first and then legal moves without any duplicated moves.
    /// Useful for move ordering, in case you want to check captures first before generating other moves.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.make_move(rust_chess.Move("e2e4"))
    /// >>> board.make_move(rust_chess.Move("d7d5"))
    /// >>> len(board.generate_legal_moves())
    /// 31
    /// >>> len(board.generate_legal_captures())
    /// 1
    /// >>> next(board.generate_legal_captures())
    /// Move(e4, d5, None)
    /// >>> len(board.generate_legal_moves())
    /// 30
    /// >>> len(board.generate_legal_captures())
    /// 0
    /// ```
    #[inline]
    fn generate_legal_captures(&mut self) -> Py<PyMoveGenerator> {
        // Get the mask of enemy‐occupied squares
        let targets_mask = self.board.color_combined(!self.board.side_to_move());

        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        // Set the iterator mask to the targets mask (check all legal captures [moves onto enemy pieces])
        self.move_gen
            .borrow_mut(py)
            .0
            .set_iterator_mask(*targets_mask);

        // Share ownership with Python
        self.move_gen.clone_ref(py)
    }

    /// Checks if the halfmoves since the last pawn move or capture is >= 100
    /// and the game is ongoing (not checkmate or stalemate).
    ///
    /// This is a claimable draw according to FIDE rules.
    ///
    /// ```python
    /// >>> rust_chess.Board().is_fifty_moves()
    /// False
    /// >>> rust_chess.Board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 100 1").is_fifty_moves()
    /// True
    /// ```
    #[inline]
    fn is_fifty_moves(&self) -> bool {
        self.halfmove_clock >= 100 && self.board.status() == chess::BoardStatus::Ongoing
    }

    /// Checks if the halfmoves since the last pawn move or capture is >= 150
    /// and the game is ongoing (not checkmate or stalemate).
    ///
    /// This is an automatic draw according to FIDE rules.
    ///
    /// ```python
    /// >>> rust_chess.Board().is_seventy_five_moves()
    /// False
    /// >>> rust_chess.Board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 150 1").is_seventy_five_moves()
    /// True
    /// ```
    #[inline]
    fn is_seventy_five_moves(&self) -> bool {
        self.halfmove_clock >= 150 && self.board.status() == chess::BoardStatus::Ongoing
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

        let num_remaining_pieces = combined_bb.popcnt();
        if num_remaining_pieces <= 2 {
            let knights = self.board.pieces(chess::Piece::Knight);
            let bishops = self.board.pieces(chess::Piece::Bishop);

            // King vs King + Knight/Bishop: Combined bitboard minus kings and knight/bishop is empty
            if num_remaining_pieces == 1 && combined_bb & !(knights | bishops) == chess::EMPTY {
                return true;
            } else if *knights == chess::EMPTY {
                // Only bishops left
                let white_bishops = bishops & white_bb;
                let black_bishops = bishops & black_bb;

                // Both sides have a bishop
                if white_bishops != chess::EMPTY && black_bishops != chess::EMPTY {
                    let white_bishop_index = white_bishops.to_square().to_index();
                    let black_bishop_index = black_bishops.to_square().to_index();

                    // King + Bishop vs King + Bishop same color: White and black bishops are on the same color square
                    return ((9 * (white_bishop_index ^ black_bishop_index)) & 8) == 0; // Check if square colors are the same
                }
            }
        }
        false
    }

    // TODO: Check threefold and fivefold repetition

    /// Checks if the game is in a threefold repetition.
    ///
    /// This is a claimable draw according to FIDE rules.
    /// TODO: Currently not implementable due to no storage of past moves
    #[inline]
    fn is_threefold_repetition(&self) -> bool {
        false
    }

    /// Checks if the game is in a fivefold repetition.
    ///
    /// This is an automatic draw according to FIDE rules.
    /// TODO: Currently not implementable due to no storage of past moves
    #[inline]
    fn is_fivefold_repetition(&self) -> bool {
        false
    }

    /// Checks if the side to move is in check.
    ///
    /// ```python
    /// >>> rust_chess.Board().is_check()
    /// False
    /// >>> rust_chess.Board("rnb1kbnr/pppp1ppp/4p3/8/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3").is_check()
    /// True
    /// ```
    #[inline]
    fn is_check(&self) -> bool {
        *self.board.checkers() != chess::EMPTY
    }

    // TODO: Docs

    /// Checks if the side to move is in stalemate
    ///
    /// ```python
    /// >>> rust_chess.Board().is_stalemate()
    /// False
    /// ```
    /// TODO
    #[inline]
    fn is_stalemate(&self) -> bool {
        self.board.status() == chess::BoardStatus::Stalemate
    }

    /// Checks if the side to move is in checkmate
    ///
    /// ```python
    /// >>> rust_chess.Board().is_checkmate()
    /// False
    /// ```
    /// TODO
    #[inline]
    fn is_checkmate(&self) -> bool {
        self.board.status() == chess::BoardStatus::Checkmate
    }

    /// Get the status of the board (ongoing, draw, or game-ending).
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.get_status()
    /// BoardStatus.ONGOING
    /// ```
    /// TODO
    #[inline]
    fn get_status(&self) -> PyBoardStatus {
        match self.board.status() {
            chess::BoardStatus::Ongoing => {
                if self.is_seventy_five_moves() {
                    PyBoardStatus::SeventyFiveMoves
                } else if self.is_insufficient_material() {
                    PyBoardStatus::InsufficientMaterial
                } else if self.is_fivefold_repetition() {
                    PyBoardStatus::FiveFoldRepetition
                } else {
                    PyBoardStatus::Ongoing
                }
            }
            chess::BoardStatus::Stalemate => PyBoardStatus::Stalemate,
            chess::BoardStatus::Checkmate => PyBoardStatus::Checkmate,
        }
    }
}

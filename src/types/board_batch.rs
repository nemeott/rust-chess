use std::fmt::Write;
use std::str::FromStr;

use pyo3::{exceptions::PyValueError, prelude::*, types::PyList};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::types::board::{PyBoard, PyBoardStatus, PyCastleRights};
use crate::types::{
    bitboard::PyBitboard,
    board::PyRepetitionDetectionMode,
    color::PyColor,
    r#move::{PyMove, PyMoveGenerator},
    piece::{PyPiece, PyPieceType},
    square::PySquare,
};

/// BoardBatch class.
/// Represents a batch of chess boards.
/// Uses the same method names as `Board`, however they operate on a batch now.
///
/// TODO: docs

// Could remove lots of code duplication from `Board`, but might be less understandable/readable.
// Would have to define Rust-only functions, then make Python wrappers in `Board` and `BoardBatch`.
// Might end up doing this anyway though; would save potential problems of changing a method in one but not the other.

// Uses SoA apprach to improve cache locality.
#[gen_stub_pyclass]
#[pyclass(name = "BoardBatch")]
pub struct PyBoardBatch {
    boards: Vec<chess::Board>,

    // TODO: LazyLock?

    // Lazily initialized per board, reset to None when a move is applied
    move_gens: Vec<std::sync::OnceLock<Py<PyMoveGenerator>>>, // Use a Py to be able to share between Python and Rust

    #[pyo3(get)]
    // FIXME: __repr__ returns raw bytes (u8 converted to bytes in PyO3)
    halfmove_clocks: Vec<u8>, // Halfmoves since last pawn move or capture

    #[pyo3(get)]
    // FIXME: __repr__ returns raw bytes (u8 converted to bytes in PyO3)
    fullmove_numbers: Vec<u8>, // Fullmove number; increments after black moves (theoretical max 218, fits in u8)

    /// The repetition dectection mode the board will use.
    #[pyo3(get)]
    repetition_detection_mode: PyRepetitionDetectionMode,

    /// Store board Zobrist hashes for board history
    #[pyo3(get)]
    board_histories: Vec<Option<Vec<u64>>>,
}

/// Rust only helpers
impl PyBoardBatch {
    /// Helper to lazily initialize and return references to the generators
    #[inline]
    // TODO: Remap
    fn ensure_move_gens(&self, py: Python<'_>) -> Vec<Py<PyMoveGenerator>> {
        self.move_gens
            .iter()
            .zip(self.boards.iter())
            .map(|(once_lock, board)| {
                once_lock
                    .get_or_init(|| Py::new(py, PyMoveGenerator::new(board)).unwrap())
                    .clone_ref(py)
            })
            .collect()
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyBoardBatch {
    // TODO: Reword docs to make more sense
    // TODO: Optimize

    /// Create a new batch of boards.
    ///
    #[new]
    #[pyo3(signature = (count, mode = PyRepetitionDetectionMode::Full))] // Default to full repetition detection
    fn new(count: usize, mode: PyRepetitionDetectionMode) -> PyResult<Self> {
        let boards = vec![chess::Board::default(); count];

        // TODO: One loop?

        let board_histories = match mode {
            PyRepetitionDetectionMode::None => vec![None; boards.len()],
            PyRepetitionDetectionMode::Full => boards
                .iter()
                .map(|board| {
                    let mut history = Vec::with_capacity(256);
                    history.push(board.get_hash());
                    Some(history)
                })
                .collect(),
        };

        let move_gens = (0..count).map(|_| std::sync::OnceLock::new()).collect();

        Ok(Self {
            boards,
            move_gens,
            halfmove_clocks: vec![0; count],
            fullmove_numbers: vec![1; count],
            repetition_detection_mode: mode,
            board_histories,
        })
    }

    /// Create a new batch of boards from a list of FEN strings.
    ///
    #[staticmethod]
    #[pyo3(signature = (fens, mode = PyRepetitionDetectionMode::Full))] // Default full repetition detection
    fn from_fens(fens: Vec<String>, mode: PyRepetitionDetectionMode) -> PyResult<Self> {
        let count = fens.len();

        let mut boards = Vec::with_capacity(count);
        let mut move_gens = Vec::with_capacity(count);
        let mut halfmove_clocks = Vec::with_capacity(count);
        let mut fullmove_numbers = Vec::with_capacity(count);
        let mut board_histories = Vec::with_capacity(count);

        for fen in fens {
            // Extract the halfmove clock and fullmove number from the FEN string
            let parts: Vec<&str> = fen.split_whitespace().collect();
            if parts.len() != 6 {
                return Err(PyValueError::new_err(
                    "FEN string must have exactly 6 parts",
                ));
            }

            // Parse the halfmove clock and fullmove number
            halfmove_clocks.push(
                parts[4]
                    .parse::<u8>()
                    .map_err(|_| PyValueError::new_err("Invalid halfmove clock"))?,
            );
            fullmove_numbers.push(
                parts[5]
                    .parse::<u8>()
                    .map_err(|_| PyValueError::new_err("Invalid fullmove number"))?,
            );

            // Parse the board using the chess crate
            let board = chess::Board::from_str(&fen)
                .map_err(|e| PyValueError::new_err(format!("Invalid FEN: {e}")))?;

            board_histories.push(match mode {
                PyRepetitionDetectionMode::None => None,
                PyRepetitionDetectionMode::Full => {
                    let mut history = Vec::with_capacity(256);
                    history.push(board.get_hash());
                    Some(history)
                }
            });

            boards.push(board);

            move_gens.push(std::sync::OnceLock::new());
        }

        Ok(Self {
            boards,
            move_gens,
            halfmove_clocks,
            fullmove_numbers,
            repetition_detection_mode: mode,
            board_histories,
        })
    }

    #[staticmethod]
    #[pyo3(signature = (boards, mode = PyRepetitionDetectionMode::Full))] // Default to full repetition detection
    fn from_boards(boards: &Bound<'_, PyList>, mode: PyRepetitionDetectionMode) -> PyResult<Self> {
        let pyboards: Vec<Py<PyBoard>> = boards
            .extract()
            .map_err(|_| PyValueError::new_err("Expected a list of Board objects"))?;

        let count = pyboards.len();

        let mut boards = Vec::with_capacity(count);
        let mut move_gens = Vec::with_capacity(count);
        let mut halfmove_clocks = Vec::with_capacity(count);
        let mut fullmove_numbers = Vec::with_capacity(count);
        let mut board_histories = Vec::with_capacity(count);

        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        for pyboard in pyboards {
            let b = pyboard.borrow(py);

            boards.push(b.board.clone());
            move_gens.push(std::sync::OnceLock::new());
            halfmove_clocks.push(b.halfmove_clock);
            fullmove_numbers.push(b.fullmove_number);
            board_histories.push(match mode {
                PyRepetitionDetectionMode::None => None,
                PyRepetitionDetectionMode::Full => {
                    let mut history = Vec::with_capacity(256);
                    history.push(b.board.get_hash());
                    Some(history)
                }
            });
        }

        Ok(Self {
            boards,
            move_gens,
            halfmove_clocks,
            fullmove_numbers,
            repetition_detection_mode: mode,
            board_histories,
        })
    }

    // /// Get the FEN string representation of the board.
    // ///
    // /// ```python
    // /// >>> rust_chess.Board().get_fen()
    // /// 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'
    // /// ```
    // #[inline]
    // fn get_fen(&self) -> String {
    //     let base_fen = self.board.to_string();

    //     // 0: board, 1: player, 2: castling, 3: en passant, 4: halfmove clock, 5: fullmove number
    //     let base_parts: Vec<&str> = base_fen.split_whitespace().collect();

    //     // The chess crate doesn't handle the halfmove and fullmove values so we need to do it ourselves
    //     format!(
    //         "{} {} {} {} {} {}",
    //         base_parts[0],        // board
    //         base_parts[1],        // player
    //         base_parts[2],        // castling
    //         base_parts[3],        // en passant
    //         self.halfmove_clock,  // halfmove clock
    //         self.fullmove_number, // fullmove number
    //     )
    // }

    // /// Get the FEN string representation of the board.
    // ///
    // /// ```python
    // /// >>> rust_chess.Board()
    // /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    // /// ```
    // #[inline]
    // fn __repr__(&self) -> String {
    //     self.get_fen()
    // }

    // /// Get the string representation of the board.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> print(board.display())
    // /// r n b q k b n r
    // /// p p p p p p p p
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// P P P P P P P P
    // /// R N B Q K B N R
    // ///
    // /// ```
    // #[inline]
    // fn display(&self) -> String {
    //     let mut s = String::new();
    //     for rank in (0..8).rev() {
    //         for file in 0..8 {
    //             let square = PySquare(unsafe { chess::Square::new(file + (rank * 8)) });
    //             if let Some(piece) = self.get_piece_on(square) {
    //                 unsafe { write!(s, "{} ", &piece.get_string()).unwrap_unchecked() }; // Safe code is for weaklings
    //             } else {
    //                 unsafe { write!(s, ". ").unwrap_unchecked() };
    //             }
    //         }
    //         unsafe { writeln!(s).unwrap_unchecked() };
    //     }
    //     s
    // }

    // /// Get the string representation of the board.
    // ///
    // /// ```python
    // /// >>> print(rust_chess.Board())
    // /// r n b q k b n r
    // /// p p p p p p p p
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// P P P P P P P P
    // /// R N B Q K B N R
    // ///
    // /// ```
    // #[inline]
    // fn __str__(&self) -> String {
    //     self.display()
    // }

    // /// Get the unicode string representation of the board.
    // ///
    // /// The dark mode parameter is enabled by default.
    // /// This inverts the color of the piece, which looks correct on a dark background.
    // /// Unicode assumes black text on white background, where in most terminals, it is the opposite.
    // /// Disable if you are a psychopath and use light mode in your terminal/IDE.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> print(board.display_unicode())
    // /// ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖
    // /// ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙
    // /// · · · · · · · ·
    // /// · · · · · · · ·
    // /// · · · · · · · ·
    // /// · · · · · · · ·
    // /// ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
    // /// ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    // ///
    // /// >>> print(board.display_unicode(dark_mode=False))
    // /// ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    // /// ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
    // /// · · · · · · · ·
    // /// · · · · · · · ·
    // /// · · · · · · · ·
    // /// · · · · · · · ·
    // /// ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙
    // /// ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖
    // ///
    // /// ```
    // #[inline]
    // #[pyo3(signature = (dark_mode = true))]
    // fn display_unicode(&self, dark_mode: bool) -> String {
    //     let mut s = String::new();
    //     for rank in (0..8).rev() {
    //         for file in 0..8 {
    //             let square = PySquare(unsafe { chess::Square::new(file + (rank * 8)) });
    //             if let Some(piece) = self.get_piece_on(square) {
    //                 unsafe { write!(s, "{} ", &piece.get_unicode(dark_mode)).unwrap_unchecked() }; // Safe code is for weaklings
    //             } else {
    //                 unsafe { write!(s, "· ").unwrap_unchecked() }; // This is a unicode middle dot, not a period
    //             }
    //         }
    //         unsafe { writeln!(s).unwrap_unchecked() };
    //     }
    //     s
    // }

    // /// Create a new move from a SAN string (e.g. "e4").
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> board.get_move_from_san("e4")
    // /// Move(e2, e4, None)
    // /// ```
    // #[inline]
    // fn get_move_from_san(&self, san: &str) -> PyResult<PyMove> {
    //     chess::ChessMove::from_san(&self.board, san)
    //         .map(PyMove)
    //         .map_err(|_| PyValueError::new_err("Invalid SAN move"))
    // }

    // // TODO: get_san_from_move

    // Get the Zobrist hashes of the boards.
    //
    #[getter]
    #[inline]
    fn get_zobrist_hash(&self) -> Vec<u64> {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_zobrist_hash(board))
            .collect()
    }

    /// Get a hash of the board batch based on the sum of the Zobrist hashes.
    /// Will likely overflow which is fine since this is a fast hash.
    #[inline]
    fn __hash__(&self) -> u64 {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_zobrist_hash(board))
            .sum()
    }

    /// Check if two board batches are equal based on the Zobrist hashes of their boards.
    ///
    #[inline]
    fn __eq__(&self, other: &Self) -> bool {
        self.boards
            .iter()
            .zip(other.boards.iter())
            .all(|(b1, b2)| PyBoard::_get_zobrist_hash(b1) == PyBoard::_get_zobrist_hash(b2))
    }

    /// Check if two board batches are not equal based on the Zobrist hashes of their boards.
    ///
    #[inline]
    fn __ne__(&self, other: &Self) -> bool {
        !self.__eq__(other)
    }

    /// Compare two board batches based on the Zobrist hashes of their boards.
    /// Returns a list of booleans where `True` indicates the respective boards match.
    ///
    #[inline]
    fn compare(&self, other: &Self) -> Vec<bool> {
        self.boards
            .iter()
            .zip(other.boards.iter())
            .map(|(b1, b2)| PyBoard::_get_zobrist_hash(b1) == PyBoard::_get_zobrist_hash(b2))
            .collect()
    }

    /// Get the current player to move for each board.
    ///
    #[getter]
    #[inline]
    fn get_turn(&self) -> Vec<PyColor> {
        self.boards
            .iter()
            .map(|board| PyColor(PyBoard::_get_turn(board)))
            .collect()
    }

    // TODO: List of colors for below functions?

    /// Get the king square of each board for a color.
    ///
    #[inline]
    fn get_king_square(&self, color: PyColor) -> Vec<PySquare> {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_king_square(board, color))
            .collect()
    }

    /// Get the castle rights of each board for a color.
    /// Returns a list `CastleRights` enum types, which has the values: `NO_RIGHTS`, `KING_SIDE`, `QUEEN_SIDE`, `BOTH`.
    ///
    #[inline]
    fn get_castle_rights(&self, color: PyColor) -> Vec<PyCastleRights> {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_castle_rights(board, color.0))
            .collect()
    }

    /// Get the castle rights of the current player to move for each board.
    ///
    #[inline]
    fn get_my_castle_rights(&self) -> Vec<PyCastleRights> {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_my_castle_rights(board))
            .collect()
    }

    /// Get the castle rights of the opponent for each board.
    ///
    #[inline]
    fn get_their_castle_rights(&self) -> Vec<PyCastleRights> {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_their_castle_rights(board))
            .collect()
    }

    /// Check if a color can castle (either side) for each board.
    /// Returns a list of booleans.
    ///
    #[inline]
    fn can_castle(&self, color: PyColor) -> Vec<bool> {
        self.boards
            .iter()
            .map(|board| PyBoard::_can_castle(board, color))
            .collect()
    }

    /// Check if a color can castle queenside for each board.
    /// Returns a list of booleans.
    ///
    #[inline]
    fn can_castle_queenside(&self, color: PyColor) -> Vec<bool> {
        self.boards
            .iter()
            .map(|board| PyBoard::_can_castle_queenside(board, color))
            .collect()
    }

    /// Check if a color can castle kingside for each board.
    /// Returns a list of booleans.
    ///
    #[inline]
    fn can_castle_kingside(&self, color: PyColor) -> Vec<bool> {
        self.boards
            .iter()
            .map(|board| PyBoard::_can_castle_kingside(board, color))
            .collect()
    }

    // TODO: Reword

    /// Check if the respective move is castling for each board.
    /// Assumes the moves are pseudo-legal.
    ///
    #[inline]
    fn is_castling(&self, chess_moves: Vec<PyMove>) -> Vec<bool> {
        chess_moves
            .iter()
            .zip(self.boards.iter())
            .map(|(chess_move, board)| PyBoard::_is_castling(board, *chess_move))
            .collect()
    }

    /// Check if the respective move is queenside castling for each board.
    /// Assumes the move is pseudo-legal.
    ///
    #[inline]
    fn is_castling_queenside(&self, chess_moves: Vec<PyMove>) -> Vec<bool> {
        chess_moves
            .iter()
            .zip(self.boards.iter())
            .map(|(chess_move, board)| PyBoard::_is_castling_queenside(board, *chess_move))
            .collect()
    }

    /// Check if the respective move is kingside castling for each board.
    /// Assumes the move is pseudo-legal.
    ///
    #[inline]
    fn is_castling_kingside(&self, chess_moves: Vec<PyMove>) -> Vec<bool> {
        chess_moves
            .iter()
            .zip(self.boards.iter())
            .map(|(chess_move, board)| PyBoard::_is_castling_kingside(board, *chess_move))
            .collect()
    }

    /// Get the color of the piece on a respective square for each board, otherwise None.
    ///
    #[inline]
    fn get_color_on(&self, squares: Vec<PySquare>) -> Vec<Option<PyColor>> {
        self.boards
            .iter()
            .zip(squares.iter())
            .map(|(board, square)| PyBoard::_get_color_on(board, *square))
            .collect()
    }

    /// Get the piece type on a respective square for each board, otherwise None.
    /// Different than `get_piece_on` because it returns the piece type, which does not include color.
    ///
    #[inline]
    fn get_piece_type_on(&self, squares: Vec<PySquare>) -> Vec<Option<PyPieceType>> {
        self.boards
            .iter()
            .zip(squares.iter())
            .map(|(board, square)| PyBoard::_get_piece_type_on(board, *square))
            .collect()
    }

    /// Get the piece on a respective square, otherwise None.
    /// Different than `get_piece_on` because it returns the piece, which includes color.
    ///
    #[inline]
    fn get_piece_on(&self, squares: Vec<PySquare>) -> Vec<Option<PyPiece>> {
        squares
            .iter()
            .zip(self.boards.iter())
            .map(|(square, board)| PyBoard::_get_piece_on(board, *square))
            .collect()
    }

    /// Get the en passant square of each board, otherwise None.
    ///
    #[getter]
    #[inline]
    fn get_en_passant(&self) -> Vec<Option<PySquare>> {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_en_passant(board))
            .collect()
    }

    /// Check if a respective move is en passant for each board.
    ///
    /// Assumes the moves are legal.
    ///
    #[inline]
    fn is_en_passant(&self, chess_moves: Vec<PyMove>) -> Vec<bool> {
        self.boards
            .iter()
            .zip(chess_moves.iter())
            .map(|(board, chess_move)| PyBoard::_is_en_passant(board, *chess_move))
            .collect()
    }

    /// Check if a respective move is a capture for each board.
    ///
    /// Assumes the moves are legal.
    #[inline]
    fn is_capture(&self, chess_moves: Vec<PyMove>) -> Vec<bool> {
        self.boards
            .iter()
            .zip(chess_moves.iter())
            .map(|(board, chess_move)| PyBoard::_is_capture(board, *chess_move))
            .collect()
    }

    /// Check if a respective move is a capture or a pawn move for each board.
    /// This type of move "zeros" the halfmove clock (sets it to 0).
    ///
    /// Assumes the moves are legal.
    ///
    #[inline]
    fn is_zeroing(&self, chess_moves: Vec<PyMove>) -> Vec<bool> {
        self.boards
            .iter()
            .zip(chess_moves.iter())
            .map(|(board, chess_move)| PyBoard::_is_zeroing(board, *chess_move))
            .collect()
    }

    /// Check if the move is legal (supposedly very slow according to the chess crate).
    /// Use this function for moves not generated by the move generator.
    /// `is_legal_quick` is faster for moves generated by the move generator.
    ///
    #[inline]
    fn is_legal_move(&self, chess_moves: Vec<PyMove>) -> Vec<bool> {
        self.boards
            .iter()
            .zip(chess_moves.iter())
            .map(|(board, chess_move)| PyBoard::_is_legal_move(board, *chess_move))
            .collect()
    }

    /// Check if the move generated by the generator is legal.
    /// Only use this function for moves generated by the move generator.
    /// You would want to use this when you have a psuedo-legal move (guarenteed by the generator).
    /// Slightly faster than using `is_legal_move` since it doesn't have to check as much stuff.
    ///
    #[inline]
    fn is_legal_generator_move(&self, chess_moves: Vec<PyMove>) -> Vec<bool> {
        self.boards
            .iter()
            .zip(chess_moves.iter())
            .map(|(board, chess_move)| PyBoard::_is_legal_generator_move(board, *chess_move))
            .collect()
    }

    // TODO: make_null_move (would require move history to undo (probably?))

    /// Make a null move for every board onto a new board batch.
    /// Returns None if the current player is in check.
    ///
    #[inline]
    fn make_null_move_new(&self) -> Option<Self> {
        let count = self.boards.len();

        let mut new_boards = Vec::with_capacity(count);
        let mut move_gens = Vec::with_capacity(count);
        let mut halfmove_clocks = Vec::with_capacity(count);
        let mut fullmove_numbers = Vec::with_capacity(count);

        for i in 0..count {
            // Get the new board using the chess crate
            new_boards.push(self.boards[i].null_move()?);

            // Create a new uninitialized move generator using the chess crate
            move_gens.push(std::sync::OnceLock::new());

            // // Increment the halfmove clock
            halfmove_clocks.push(self.halfmove_clocks[i] + 1); // Null moves aren't zeroing, so we can just add 1 here

            // // Increment fullmove number if black moves
            #[allow(clippy::cast_possible_truncation)]
            fullmove_numbers
                .push(self.fullmove_numbers[i] + (self.boards[i].side_to_move().to_index() as u8)); // White is 0, black is 1
        }

        Some(Self {
            boards: new_boards,
            move_gens,
            halfmove_clocks,
            fullmove_numbers,
            repetition_detection_mode: self.repetition_detection_mode.clone(),

            // Don't update move history when making a null move
            board_histories: self.board_histories.clone(),
        })
    }

    /// Make a respective move onto each board.
    ///
    /// Defaults to checking move legality, unless the optional legality parameter is `False`.
    /// Not checking move legality will provide a slight performance boost, but crash if the move is invalid.
    /// Checking legality will return an error if the move is illegal.
    ///
    // TODO: is_generator_move
    // TODO: Optimize
    #[pyo3(signature = (chess_moves, check_legality = true))]
    #[inline]
    fn make_move(&mut self, chess_moves: Vec<PyMove>, check_legality: bool) -> PyResult<()> {
        let count = self.boards.len();

        for i in 0..count {
            // Check if draw by fivefold
            if PyBoard::_is_n_repetition(&self.board_histories[i], self.halfmove_clocks[i], 5) {
                return Err(PyValueError::new_err(
                    "Game over due to fivefold repetition",
                ));
            }

            // If we are checking legality, check if the move is legal
            if check_legality && !PyBoard::_is_legal_move(&self.boards[i], chess_moves[i]) {
                return Err(PyValueError::new_err("Illegal move"));
            }

            // Make the move onto a new board using the chess crate
            let temp_board: chess::Board = self.boards[i].make_move_new(chess_moves[i].0);

            // Reset the halfmove clock if the move zeroes (is a capture or pawn move and therefore "zeroes" the halfmove clock)
            let mut is_zeroing = false;
            if PyBoard::_is_zeroing(&self.boards[i], chess_moves[i]) {
                self.halfmove_clocks[i] = 0;
                is_zeroing = true;
            } else {
                self.halfmove_clocks[i] += 1; // Add one if not zeroing
            }

            // Increment fullmove number if black moves
            self.fullmove_numbers[i] += self.boards[i].side_to_move().to_index() as u8; // White is 0, black is 1

            // Add the new board's Zobrist hash to history
            if let Some(history) = &mut self.board_histories[i] {
                if is_zeroing {
                    // Don't need previous history anymore since it is a zeroing move (irreversible)
                    history.clear()
                }
                history.push(temp_board.get_hash());
            }

            // Invalidate the move generator since the board has changed
            self.move_gens[i].take();

            // Update the current board
            self.boards[i] = temp_board;
        }

        Ok(())
    }

    /// Make a respective move onto a new board for each board.
    ///
    /// Defaults to checking move legality, unless the optional legality parameter is `False`.
    /// Not checking move legality will provide a slight performance boost, but crash if the move is invalid.
    /// Checking legality will return an error if the move is illegal.
    ///
    #[pyo3(signature = (chess_moves, check_legality = true))]
    #[inline]
    fn make_move_new(&self, chess_moves: Vec<PyMove>, check_legality: bool) -> PyResult<Self> {
        let count = self.boards.len();

        let mut new_boards = Vec::with_capacity(count);
        let mut move_gens = Vec::with_capacity(count);
        let mut halfmove_clocks = Vec::with_capacity(count);
        let mut fullmove_numbers = Vec::with_capacity(count);
        let mut board_histories = Vec::with_capacity(count);

        for i in 0..count {
            // Check if draw by fivefold
            if PyBoard::_is_n_repetition(&self.board_histories[i], self.halfmove_clocks[i], 5) {
                return Err(PyValueError::new_err(
                    "Game over due to fivefold repetition",
                ));
            }

            // If we are checking legality, check if the move is legal
            if check_legality && !PyBoard::_is_legal_move(&self.boards[i], chess_moves[i]) {
                return Err(PyValueError::new_err("Illegal move"));
            }

            // Make the move onto a new board using the chess crate
            new_boards.push(self.boards[i].make_move_new(chess_moves[i].0));

            // Reset the halfmove clock if the move zeroes (is a capture or pawn move and therefore "zeroes" the halfmove clock)
            let mut is_zeroing = false;
            if PyBoard::_is_zeroing(&self.boards[i], chess_moves[i]) {
                halfmove_clocks.push(0);
                is_zeroing = true;
            } else {
                halfmove_clocks.push(self.halfmove_clocks[i] + 1); // Add one if not zeroing
            }

            // Increment fullmove number if black moves
            fullmove_numbers
                .push(self.fullmove_numbers[i] + self.boards[i].side_to_move().to_index() as u8); // White is 0, black is 1

            board_histories.push(self.board_histories[i].as_ref().map(|history| {
                let mut history = history.clone();
                if is_zeroing {
                    // Don't need previous history since it's a zeroing move (irreversible)
                    history.clear();
                }
                history.push(new_boards[i].get_hash());
                history
            }));

            move_gens.push(std::sync::OnceLock::new());
        }

        Ok(Self {
            boards: new_boards,
            move_gens,
            halfmove_clocks,
            fullmove_numbers,
            repetition_detection_mode: self.repetition_detection_mode,
            board_histories,
        })
    }

    /// Get the bitboard of the side to move's pinned pieces for each board.
    ///
    #[inline]
    fn get_pinned_bitboard(&self) -> Vec<PyBitboard> {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_pinned_bitboard(board))
            .collect()
    }

    /// Get the bitboard of the pieces putting the side to move in check for each board.
    ///
    #[inline]
    fn get_checkers_bitboard(&self) -> Vec<PyBitboard> {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_checkers_bitboard(board))
            .collect()
    }

    /// Get the bitboard of all the pieces of a certain color for each board.
    ///
    #[inline]
    fn get_color_bitboard(&self, color: PyColor) -> Vec<PyBitboard> {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_color_bitboard(board, color))
            .collect()
    }

    /// Get the bitboard of all the pieces of a certain type for each board.
    ///
    #[inline]
    fn get_piece_type_bitboard(&self, piece_type: PyPieceType) -> Vec<PyBitboard> {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_piece_type_bitboard(board, piece_type))
            .collect()
    }

    /// Get the bitboard of all the pieces of a certain color and type for each board.
    ///
    #[inline]
    fn get_piece_bitboard(&self, piece: PyPiece) -> Vec<PyBitboard> {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_piece_bitboard(board, piece))
            .collect()
    }

    /// Get the bitboard of all the pieces for each board.
    ///
    #[inline]
    fn get_all_bitboard(&self) -> Vec<PyBitboard> {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_all_bitboard(board))
            .collect()
    }

    // /// Get the number of moves remaining in the move generator.
    // /// This is the number of remaining moves that can be generated.
    // /// Does not consume any iterations.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> board.get_generator_num_remaining()
    // /// 20
    // /// >>> next(board.generate_legal_moves())
    // /// Move(a2, a3, None)
    // /// >>> board.get_generator_num_remaining()
    // /// 19
    // /// ```
    // #[inline]
    // fn get_generator_num_remaining(&self) -> usize {
    //     // We can assume the GIL is acquired, since this function is only called from Python
    //     let py = unsafe { Python::assume_attached() };
    //     self.ensure_move_gen(py).borrow(py).__len__()
    // }

    /// Reset the move generators for the current boards.
    ///
    #[inline]
    fn reset_move_generators(&mut self) {
        // Invalidate the move generators
        for lock in &mut self.move_gens {
            lock.take();
        }
    }

    // /// Remove a move from the move generator.
    // /// Prevents the move from being generated.
    // /// Updates the generator mask to exclude the move.
    // /// Useful if you already have a certain move and don't need to generate it again.
    // ///
    // /// **WARNING**: using any form of `legal_move` or `legal_capture` generation
    // /// will set the generator mask, invalidating any previous removals by this function.
    // /// This also applies to setting the generator mask manually.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> len(board.generate_moves())  # Legal moves by default
    // /// 20
    // /// >>> move = rust_chess.Move("a2a3")
    // /// >>> board.remove_generator_move(move)
    // /// >>> len(board.generate_moves())
    // /// 19
    // /// >>> move in board.generate_moves()  # Consumes generator moves
    // /// False
    // /// >>> len(board.generate_moves())
    // /// 0
    // /// ```
    // #[inline]
    // fn remove_generator_move(&mut self, chess_move: PyMove) {
    //     // We can assume the GIL is acquired, since this function is only called from Python
    //     let py = unsafe { Python::assume_attached() };
    //     self.ensure_move_gen(py)
    //         .borrow_mut(py)
    //         .remove_move(chess_move.0);
    // }

    // /// Retains only moves whose destination squares are in the given mask.
    // ///
    // /// The mask is a bitboard of allowed landing squares.
    // /// Only moves landing on squares in the mask will be generated.
    // /// See `exclude_generator_mask` for the inverse.
    // ///
    // /// Moves that have already been iterated over will not be generated again, regardless of the mask value.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> len(board.generate_moves())
    // /// 20
    // /// >>> board.retain_generator_mask(rust_chess.E4.to_bitboard())
    // /// >>> len(board.generate_moves())
    // /// 1
    // /// >>> board.generate_next_move()
    // /// Move(e2, e4, None)
    // /// ```
    // #[inline]
    // fn retain_generator_mask(&mut self, mask: PyBitboard) {
    //     // We can assume the GIL is acquired, since this function is only called from Python
    //     let py = unsafe { Python::assume_attached() };
    //     self.ensure_move_gen(py).borrow_mut(py).retain_mask(mask.0);
    // }

    // /// Excludes moves whose destination squares are in the given mask.
    // ///
    // /// The mask is a bitboard of forbidden landing squares.
    // /// Only moves landing on squares not in the mask will be generated.
    // /// See `retain_generator_mask` for the inverse.
    // ///
    // /// Removed moves stay removed even if you later generate over all legal moves.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> len(board.generate_moves())
    // /// 20
    // /// >>> board.exclude_generator_mask(rust_chess.E4.to_bitboard())
    // /// >>> len(board.generate_moves())
    // /// 19
    // /// >>> rust_chess.Move("e2e4") in board.generate_moves()
    // /// False
    // /// >>> len(board.generate_moves())
    // /// 0
    // /// ```
    // #[inline]
    // fn exclude_generator_mask(&mut self, mask: PyBitboard) {
    //     // We can assume the GIL is acquired, since this function is only called from Python
    //     let py = unsafe { Python::assume_attached() };
    //     self.ensure_move_gen(py).borrow_mut(py).exclude_mask(mask.0);
    // }

    // TODO: Docs

    /// Get the next remaining move in each generator.
    /// Updates the move generators to the next move.
    ///
    /// Unless masks have been set, this will return the next legal move for each generator by default.
    ///
    #[inline]
    fn generate_next_moves(&mut self) -> Vec<Option<PyMove>> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        self.ensure_move_gens(py)
            .iter()
            .map(|gen_lock| gen_lock.borrow_mut(py).__next__())
            .collect()
    }

    /// Get the next remaining legal move in each generator.
    /// Updates the move generators to the next legal move.
    ///
    /// Allows all legal destination squares for each generator.
    ///
    #[inline]
    fn generate_next_legal_moves(&mut self) -> Vec<Option<PyMove>> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        self.ensure_move_gens(py)
            .iter()
            .map(|gen_lock| {
                gen_lock.borrow_mut(py).retain_mask(!chess::EMPTY);
                gen_lock.borrow_mut(py).__next__()
            })
            .collect()
    }

    /// Get the next remaining legal capture in each generator.
    /// Updates the move generators to the next move.
    ///
    /// Allows only enemy-occupied destination squares for each generator.
    ///
    #[inline]
    fn generate_next_legal_captures(&mut self) -> Vec<Option<PyMove>> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        self.ensure_move_gens(py)
            .iter()
            .zip(self.boards.iter())
            .map(|(gen_lock, board)| {
                let targets_mask = board.color_combined(!board.side_to_move());
                gen_lock.borrow_mut(py).retain_mask(*targets_mask);
                gen_lock.borrow_mut(py).__next__()
            })
            .collect()
    }

    // // TODO: Generate moves_list (PyList<PyMove>)

    // /// Generate the next remaining moves for the current board.
    // /// Exhausts the move generator if fully iterated over.
    // /// Updates the move generator.
    // ///
    // /// Unless a mask has been set, this will generate the next legal moves by default.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> len(board.generate_moves())
    // /// 20
    // /// >>> board.retain_generator_mask(rust_chess.Bitboard(402915328))
    // /// >>> len(board.generate_moves())
    // /// 4
    // /// >>> list(board.generate_moves())
    // /// [Move(c2, c3, None), Move(d2, d4, None), Move(e2, e4, None), Move(b1, c3, None)]
    // /// >>> len(board.generate_moves())
    // /// 0
    // /// ```
    // #[inline]
    // fn generate_moves(&mut self) -> Py<PyMoveGenerator> {
    //     // We can assume the GIL is acquired, since this function is only called from Python
    //     let py = unsafe { Python::assume_attached() };

    //     // Share ownership with Python
    //     self.ensure_move_gen(py)
    // }

    // /// Generate the next remaining legal moves for the current board.
    // /// Exhausts the move generator if fully iterated over.
    // /// Updates the move generator.
    // ///
    // /// Will not iterate over the same moves already generated by `generate_legal_captures`.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> len(board.generate_legal_moves())
    // /// 20
    // /// >>> list(board.generate_legal_moves())
    // /// [Move(a2, a3, None), Move(a2, a4, None), ..., Move(g1, h3, None)]
    // /// >>> len(board.generate_legal_moves())
    // /// 0
    // /// >>> board.reset_move_generator()
    // /// >>> len(board.generate_legal_moves())
    // /// 20
    // /// ```
    // #[inline]
    // fn generate_legal_moves(&mut self) -> Py<PyMoveGenerator> {
    //     // We can assume the GIL is acquired, since this function is only called from Python
    //     let py = unsafe { Python::assume_attached() };

    //     let gen_ref = self.ensure_move_gen(py);

    //     // Allow all destination squares again for iteration
    //     gen_ref.borrow_mut(py).retain_mask(!chess::EMPTY);

    //     // Share ownership with Python
    //     gen_ref
    // }

    // /// Generate the next remaining legal captures for the current board.
    // /// Exhausts the move generator if fully iterated over.
    // /// Updates the move generator.
    // ///
    // /// Can iterate over legal captures first and then legal moves without any duplicated moves.
    // /// Useful for move ordering, in case you want to check captures first before generating other moves.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> board.make_move(rust_chess.Move("e2e4"))
    // /// >>> board.make_move(rust_chess.Move("d7d5"))
    // /// >>> len(board.generate_legal_moves())
    // /// 31
    // /// >>> len(board.generate_legal_captures())
    // /// 1
    // /// >>> next(board.generate_legal_captures())
    // /// Move(e4, d5, None)
    // /// >>> len(board.generate_legal_moves())
    // /// 30
    // /// >>> len(board.generate_legal_captures())
    // /// 0
    // /// ```
    // #[inline]
    // fn generate_legal_captures(&mut self) -> Py<PyMoveGenerator> {
    //     // Get the mask of enemy‐occupied squares
    //     let targets_mask = self.board.color_combined(!self.board.side_to_move());

    //     // We can assume the GIL is acquired, since this function is only called from Python
    //     let py = unsafe { Python::assume_attached() };

    //     let gen_ref = self.ensure_move_gen(py);

    //     // Allow only capture destination squares for iteration
    //     gen_ref.borrow_mut(py).retain_mask(*targets_mask);

    //     // Share ownership with Python
    //     gen_ref
    // }

    /// Checks if the halfmoves since the last pawn move or capture is >= 100
    /// and the game is ongoing (not checkmate or stalemate) for each board.
    ///
    /// This is a claimable draw according to FIDE rules.
    ///
    #[inline]
    fn is_fifty_moves(&self) -> Vec<bool> {
        self.boards
            .iter()
            .zip(self.halfmove_clocks.iter())
            .map(|(board, halfmove_clock)| PyBoard::_is_fifty_moves(board, *halfmove_clock))
            .collect()
    }

    /// Checks if the halfmoves since the last pawn move or capture is >= 150
    /// and the game is ongoing (not checkmate or stalemate) for each board.
    ///
    /// This is an automatic draw according to FIDE rules.
    ///
    #[inline]
    fn is_seventy_five_moves(&self) -> Vec<bool> {
        self.boards
            .iter()
            .zip(self.halfmove_clocks.iter())
            .map(|(board, halfmove_clock)| PyBoard::_is_seventy_five_moves(board, *halfmove_clock))
            .collect()
    }

    /// Checks if the side to move has insufficient material to checkmate the opponent for each board.
    /// The cases where this is true are:
    ///     1. K vs K
    ///     2. K vs K + N
    ///     3. K vs K + B
    ///     4. K + B vs K + B with the bishops on the same color.
    ///
    #[inline]
    fn is_insufficient_material(&self) -> Vec<bool> {
        self.boards
            .iter()
            .map(|board| PyBoard::_is_insufficient_material(board))
            .collect()
    }

    /// Checks if the current position is a n-fold repetition for each board.
    ///
    /// TODO: Quick check (only check last few moves since that is common error for engines)
    /// TODO: Add option to use full, or no repetition checks
    #[inline]
    fn is_n_repetition(&self, n: u8) -> Vec<bool> {
        self.board_histories
            .iter()
            .zip(self.halfmove_clocks.iter())
            .map(|(board_history, halfmove_clock)| {
                PyBoard::_is_n_repetition(board_history, *halfmove_clock, n)
            })
            .collect()
    }

    /// Checks if the current position is a threefold repetition for each board.
    /// This is a claimable draw according to FIDE rules.
    ///
    #[inline]
    fn is_threefold_repetition(&self) -> Vec<bool> {
        self.is_n_repetition(3)
    }

    /// Checks if the current position is a fivefold repetition for each board.
    /// This is an automatic draw according to FIDE rules.
    ///
    #[inline]
    fn is_fivefold_repetition(&self) -> Vec<bool> {
        self.is_n_repetition(5)
    }

    /// Checks if the side to move is in check for each board.
    ///
    #[inline]
    fn is_check(&self) -> Vec<bool> {
        self.boards
            .iter()
            .map(|board| PyBoard::_is_check(board))
            .collect()
    }

    /// Checks if the side to move is in stalemate for each board.
    ///
    #[inline]
    fn is_stalemate(&self) -> Vec<bool> {
        self.boards
            .iter()
            .map(|board| PyBoard::_is_stalemate(board))
            .collect()
    }

    /// Checks if the side to move is in checkmate for each board.
    ///
    #[inline]
    fn is_checkmate(&self) -> Vec<bool> {
        self.boards
            .iter()
            .map(|board| PyBoard::_is_checkmate(board))
            .collect()
    }

    /// Get the status of each board (ongoing, draw, or game-ending).
    ///
    #[inline]
    fn get_status(&self) -> Vec<PyBoardStatus> {
        self.boards
            .iter()
            .zip(self.board_histories.iter())
            .zip(self.halfmove_clocks.iter())
            .map(|((board, board_history), halfmove_clock)| {
                PyBoard::_get_status(board, board_history, *halfmove_clock)
            })
            .collect()
    }
}

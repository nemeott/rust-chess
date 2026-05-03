use std::fmt::Write;
use std::str::FromStr;

use pyo3::{exceptions::PyValueError, prelude::*, types::PyList};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::types::board::{PyBoard, PyCastleRights};
use crate::types::{
    bitboard::PyBitboard,
    board::PyRepetitionDetectionMode,
    color::PyColor,
    r#move::{PyMove, PyMoveGenerator},
    piece::{PyPiece, PyPieceType},
    square::PySquare,
};

/// Helper function to check en passant (takes Rust types).
#[inline]
fn _is_en_passant(board: &chess::Board, chess_move: &PyMove) -> bool {
    let source = chess_move.0.get_source();
    let dest = chess_move.0.get_dest();

    // The Rust chess crate doesn't actually compute this right; it returns the square that the pawn was moved to.
    // The actual en passant square is the one that one can move to that would cause en passant.
    // TLDR: The actual en passant square is one above or below the one returned by the chess crate.
    let ep_square = board
        .en_passant()
        .and_then(|sq| match board.side_to_move() {
            chess::Color::White => sq.up(),
            chess::Color::Black => sq.down(),
        });

    ep_square.is_some_and(|ep_sq| ep_sq == dest) // Use our en passant square function since it is accurate
        && board.piece_on(source).is_some_and(|p| p == chess::Piece::Pawn) // Moving pawn
        && {
            // Moving diagonally
            #[allow(clippy::cast_possible_truncation)]
            let diff = (dest.to_int() as i8 - source.to_int() as i8).abs();
            diff == 7 || diff == 9
        }
        && board.piece_on(dest).is_none() // Target square is empty
}

/// BoardBatch class.
/// Represents a batch of chess boards.
/// Uses the same method names as `Board`, however they operate on a batch now.
///
/// TODO: docs

// Uses SoA apprach to improve cache locality
#[gen_stub_pyclass]
#[pyclass(name = "BoardBatch")]
pub struct PyBoardBatch {
    boards: Vec<chess::Board>,

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
        self.boards.iter().map(|board| board.get_hash()).collect()
    }

    /// Get a hash of the board batch based on the sum of the Zobrist hashes.
    /// Will likely overflow which is fine since this is a fast hash.
    #[inline]
    fn __hash__(&self) -> u64 {
        self.boards.iter().map(|board| board.get_hash()).sum()
    }

    /// Check if two board batches are equal based on the Zobrist hashes of their boards.
    ///
    #[inline]
    fn __eq__(&self, other: &Self) -> bool {
        self.boards
            .iter()
            .zip(other.boards.iter())
            .all(|(b1, b2)| b1.get_hash() == b2.get_hash())
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
            .map(|(b1, b2)| b1.get_hash() == b2.get_hash())
            .collect()
    }

    /// Get the current player to move for each board.
    ///
    #[getter]
    #[inline]
    fn get_turn(&self) -> Vec<PyColor> {
        self.boards
            .iter()
            .map(|board| PyColor(board.side_to_move()))
            .collect()
    }

    // TODO: List of colors for below functions?

    /// Get the king square of each board for a color.
    ///
    #[inline]
    fn get_king_square(&self, color: PyColor) -> Vec<PySquare> {
        self.boards
            .iter()
            .map(|board| PySquare(board.king_square(color.0)))
            .collect()
    }

    /// Get the castle rights of each board for a color.
    /// Returns a list `CastleRights` enum types, which has the values: `NO_RIGHTS`, `KING_SIDE`, `QUEEN_SIDE`, `BOTH`.
    ///
    #[inline]
    fn get_castle_rights(&self, color: PyColor) -> Vec<PyCastleRights> {
        self.boards
            .iter()
            .map(|board| match board.castle_rights(color.0) {
                chess::CastleRights::NoRights => PyCastleRights::NoRights,
                chess::CastleRights::QueenSide => PyCastleRights::QueenSide,
                chess::CastleRights::KingSide => PyCastleRights::KingSide,
                chess::CastleRights::Both => PyCastleRights::Both,
            })
            .collect()
    }

    /// Get the castle rights of the current player to move for each board.
    ///
    #[inline]
    fn get_my_castle_rights(&self) -> Vec<PyCastleRights> {
        self.boards
            .iter()
            .map(|board| match board.castle_rights(board.side_to_move()) {
                chess::CastleRights::NoRights => PyCastleRights::NoRights,
                chess::CastleRights::QueenSide => PyCastleRights::QueenSide,
                chess::CastleRights::KingSide => PyCastleRights::KingSide,
                chess::CastleRights::Both => PyCastleRights::Both,
            })
            .collect()
    }

    /// Get the castle rights of the opponent for each board.
    ///
    #[inline]
    fn get_their_castle_rights(&self) -> Vec<PyCastleRights> {
        self.boards
            .iter()
            .map(|board| match board.castle_rights(!board.side_to_move()) {
                chess::CastleRights::NoRights => PyCastleRights::NoRights,
                chess::CastleRights::QueenSide => PyCastleRights::QueenSide,
                chess::CastleRights::KingSide => PyCastleRights::KingSide,
                chess::CastleRights::Both => PyCastleRights::Both,
            })
            .collect()
    }

    /// Check if a color can castle (either side) for each board.
    /// Returns a list of booleans.
    ///
    #[inline]
    fn can_castle(&self, color: PyColor) -> Vec<bool> {
        self.boards
            .iter()
            .map(|board| board.castle_rights(color.0) != chess::CastleRights::NoRights)
            .collect()
    }

    /// Check if a color can castle queenside for each board.
    /// Returns a list of booleans.
    ///
    #[inline]
    fn can_castle_queenside(&self, color: PyColor) -> Vec<bool> {
        self.boards
            .iter()
            .map(|board| board.castle_rights(color.0).has_queenside())
            .collect()
    }

    /// Check if a color can castle kingside for each board.
    /// Returns a list of booleans.
    ///
    #[inline]
    fn can_castle_kingside(&self, color: PyColor) -> Vec<bool> {
        self.boards
            .iter()
            .map(|board| board.castle_rights(color.0).has_kingside())
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
            .map(|(chess_move, board)| {
                let source = chess_move.0.get_source();

                // Check if the moving piece is a king
                if board
                    .piece_on(source)
                    .is_some_and(|p| p == chess::Piece::King)
                {
                    // Check if the move is two squares horizontally
                    let dest = chess_move.0.get_dest();
                    // #[allow(clippy::cast_possible_truncation)] //
                    return (dest.to_int() as i8 - source.to_int() as i8).abs() == 2;
                }
                false
            })
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
            .map(|(chess_move, board)| {
                let source = chess_move.0.get_source();

                // Check if the moving piece is a king
                if board
                    .piece_on(source)
                    .is_some_and(|p| p == chess::Piece::King)
                {
                    // Check if the move is two squares to the left
                    let dest = chess_move.0.get_dest();
                    #[allow(clippy::cast_possible_truncation)]
                    return dest.to_int() as i8 - source.to_int() as i8 == -2;
                }
                false
            })
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
            .map(|(chess_move, board)| {
                let source = chess_move.0.get_source();

                // Check if the moving piece is a king
                if board
                    .piece_on(source)
                    .is_some_and(|p| p == chess::Piece::King)
                {
                    // Check if the move is two squares to the right
                    let dest = chess_move.0.get_dest();
                    #[allow(clippy::cast_possible_truncation)]
                    return dest.to_int() as i8 - source.to_int() as i8 == 2;
                }
                false
            })
            .collect()
    }

    /// Get the color of the piece on a respective square for each board, otherwise None.
    ///
    #[inline]
    fn get_color_on(&self, squares: Vec<PySquare>) -> Vec<Option<PyColor>> {
        // Get the color of the piece on the respective square using the chess crate
        self.boards
            .iter()
            .zip(squares.iter())
            .map(|(board, square)| board.color_on(square.0).map(PyColor))
            .collect()
    }

    /// Get the piece type on a respective square for each board, otherwise None.
    /// Different than `get_piece_on` because it returns the piece type, which does not include color.
    ///
    #[inline]
    fn get_piece_type_on(&self, squares: Vec<PySquare>) -> Vec<Option<PyPieceType>> {
        // Get the piece on the respective square using the chess crate
        self.boards
            .iter()
            .zip(squares.iter())
            .map(|(board, square)| board.piece_on(square.0).map(PyPieceType))
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
            .map(|(square, board)| {
                Some(PyPiece {
                    piece_type: board.piece_on(square.0).map(PyPieceType)?,
                    color: board.color_on(square.0).map(PyColor)?,
                })
            })
            .collect()
    }

    /// Get the en passant square of each board, otherwise None.
    ///
    #[getter]
    #[inline]
    fn get_en_passant(&self) -> Vec<Option<PySquare>> {
        // The Rust chess crate doesn't actually compute this right; it returns the square that the pawn was moved to.
        // The actual en passant square is the one that one can move to that would cause en passant.
        // TLDR: The actual en passant square is one above or below the one returned by the chess crate.
        self.boards
            .iter()
            .map(|board| {
                board.en_passant().map(|sq| match board.side_to_move() {
                    chess::Color::White => PySquare(sq.up().unwrap()),
                    chess::Color::Black => PySquare(sq.down().unwrap()),
                })
            })
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
            .map(|(board, chess_move)| _is_en_passant(board, chess_move))
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
            .map(|(board, chess_move)| {
                board.piece_on(chess_move.0.get_dest()).is_some() // Capture (moving piece onto other piece)
                || _is_en_passant(board, chess_move) // Or the move is en passant (also a capture)
            })
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
            .map(|(board, chess_move)| {
                board.piece_on(chess_move.0.get_source()).is_some_and(|p| p == chess::Piece::Pawn) // Pawn move
                || board.piece_on(chess_move.0.get_dest()).is_some() // Capture (moving piece onto other piece)
            })
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
            .map(|(board, chess_move)| chess::Board::legal(board, chess_move.0))
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
            .map(|(board, chess_move)| chess::MoveGen::legal_quick(board, chess_move.0))
            .collect()
    }

    // // TODO: make_null_move (would require move history to undo (probably?))

    // /// Make a null move onto a new board.
    // /// Returns None if the current player is in check.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> board
    // /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    // /// >>> new_board = board.make_null_move_new()
    // /// >>> new_board
    // /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 1 1
    // ///
    // /// >>> board = rust_chess.Board("rnbqkbnr/ppppp1pp/5p2/7Q/8/4P3/PPPP1PPP/RNB1KBNR b KQkq - 1 2")
    // /// >>> new_board = board.make_null_move_new()
    // /// >>> print(new_board)
    // /// None
    // /// ```
    // #[inline]
    // fn make_null_move_new(&self) -> Option<Self> {
    //     // Get the new board using the chess crate
    //     let new_board = self.board.null_move()?;

    //     Some(Self {
    //         board: new_board,
    //         // Create a new uninitialized move generator using the chess crate
    //         move_gen: std::sync::OnceLock::new(),
    //         // // Increment the halfmove clock
    //         halfmove_clock: self.halfmove_clock + 1, // Null moves aren't zeroing, so we can just add 1 here
    //         // // Increment fullmove number if black moves
    //         #[allow(clippy::cast_possible_truncation)]
    //         fullmove_number: self.fullmove_number + (self.board.side_to_move().to_int() as u8), // White is 0, black is 1
    //         repetition_detection_mode: self.repetition_detection_mode,
    //         // Don't update move history when making a null move
    //         board_history: self.board_history.clone(),
    //     })
    // }

    // /// Make a move onto the current board.
    // ///
    // /// Defaults to checking move legality, unless the optional legality parameter is `False`.
    // /// Not checking move legality will provide a slight performance boost, but crash if the move is invalid.
    // /// Checking legality will return an error if the move is illegal.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> board.make_move(rust_chess.Move("e2e4"))
    // /// >>> print(board)
    // /// r n b q k b n r
    // /// p p p p p p p p
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . P . . .
    // /// . . . . . . . .
    // /// P P P P . P P P
    // /// R N B Q K B N R
    // ///
    // /// ```
    // #[pyo3(signature = (chess_move, check_legality = true))]
    // #[inline]
    // fn make_move(&mut self, chess_move: PyMove, check_legality: bool) -> PyResult<()> {
    //     // Check if draw by fivefold
    //     if self.is_fivefold_repetition() {
    //         return Err(PyValueError::new_err(
    //             "Game over due to fivefold repetition",
    //         ));
    //     }

    //     // If we are checking legality, check if the move is legal
    //     if check_legality && !self.is_legal_move(chess_move) {
    //         return Err(PyValueError::new_err("Illegal move"));
    //     }

    //     // Make the move onto a new board using the chess crate
    //     let temp_board: chess::Board = self.board.make_move_new(chess_move.0);

    //     // Reset the halfmove clock if the move zeroes (is a capture or pawn move and therefore "zeroes" the halfmove clock)
    //     if self.is_zeroing(chess_move) {
    //         self.halfmove_clock = 0;

    //         // Don't need previous history anymore since it is a zeroing move (irreversible)
    //         if let Some(history) = &mut self.board_history {
    //             history.clear();
    //         }
    //     } else {
    //         self.halfmove_clock += 1; // Add one if not zeroing
    //     }

    //     // Increment fullmove number if black moves
    //     self.fullmove_number += self.board.side_to_move().to_int() as u8; // White is 0, black is 1

    //     // Invalidate the move generator since the board has changed
    //     self.move_gen.take();

    //     // Update the current board
    //     self.board = temp_board;

    //     // Add the new board's Zobrist hash to history
    //     if let Some(history) = &mut self.board_history {
    //         history.push(temp_board.get_hash());
    //     }

    //     Ok(())
    // }

    // /// Make a move onto a new board.
    // ///
    // /// Defaults to checking move legality, unless the optional legality parameter is `False`.
    // /// Not checking move legality will provide a slight performance boost, but crash if the move is invalid.
    // /// Checking legality will return an error if the move is illegal.
    // ///
    // /// ```python
    // /// >>> old_board = rust_chess.Board()
    // /// >>> new_board = old_board.make_move_new(rust_chess.Move("e2e4"))
    // /// >>> print(new_board)
    // /// r n b q k b n r
    // /// p p p p p p p p
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . P . . .
    // /// . . . . . . . .
    // /// P P P P . P P P
    // /// R N B Q K B N R
    // ///
    // /// >>> print(old_board)
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
    // #[pyo3(signature = (chess_move, check_legality = true))]
    // #[inline]
    // fn make_move_new(&self, chess_move: PyMove, check_legality: bool) -> PyResult<Self> {
    //     // Check if draw by fivefold
    //     if self.is_fivefold_repetition() {
    //         return Err(PyValueError::new_err(
    //             "Game over due to fivefold repetition",
    //         ));
    //     }

    //     // If we are checking legality, check if the move is legal
    //     if check_legality && !self.is_legal_move(chess_move) {
    //         return Err(PyValueError::new_err("Illegal move"));
    //     }

    //     // Make the move onto a new board using the chess crate
    //     let new_board: chess::Board = self.board.make_move_new(chess_move.0);

    //     let is_zeroing: bool = self.is_zeroing(chess_move);

    //     Ok(Self {
    //         board: new_board,
    //         move_gen: std::sync::OnceLock::new(),
    //         // Reset the halfmove clock if the move zeroes (is a capture or pawn move and therefore "zeroes" the halfmove clock)
    //         halfmove_clock: if is_zeroing {
    //             0
    //         } else {
    //             self.halfmove_clock + 1
    //         },
    //         // Increment fullmove number if black moves
    //         #[allow(clippy::cast_possible_truncation)]
    //         fullmove_number: self.fullmove_number + (self.board.side_to_move().to_int() as u8), // White is 0, black is 1
    //         repetition_detection_mode: self.repetition_detection_mode,
    //         // Add the new board's Zobrist hash to history
    //         board_history: self.board_history.as_ref().map(|history| {
    //             let mut new_history = if is_zeroing {
    //                 Vec::with_capacity(history.capacity()) // Don't need previous history anymore since it is a zeroing move (irreversible)
    //             } else {
    //                 history.clone()
    //             };

    //             new_history.push(new_board.get_hash());
    //             new_history
    //         }),
    //     })
    // }

    // /// Get the bitboard of the side to move's pinned pieces.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> board.get_pinned_bitboard().popcnt()
    // /// 0
    // ///
    // /// >>> board.make_move(rust_chess.Move("e2e4"))
    // /// >>> board.make_move(rust_chess.Move("d7d5"))
    // /// >>> board.make_move(rust_chess.Move("d1h5"))
    // /// >>> board.get_pinned_bitboard().popcnt()
    // /// 1
    // /// >>> board.get_pinned_bitboard()
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . . X . .
    // /// . . . . . . . .
    // /// ```
    // #[inline]
    // fn get_pinned_bitboard(&self) -> PyBitboard {
    //     PyBitboard(*self.board.pinned())
    // }

    // /// Get the bitboard of the pieces putting the side to move in check.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> board.get_checkers_bitboard().popcnt()
    // /// 0
    // ///
    // /// >>> board.make_move(rust_chess.Move("e2e4"))
    // /// >>> board.make_move(rust_chess.Move("f7f6"))
    // /// >>> board.make_move(rust_chess.Move("d1h5"))
    // /// >>> board.get_checkers_bitboard().popcnt()
    // /// 1
    // /// >>> board.get_checkers_bitboard()
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . . . . X
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// ```
    // #[inline]
    // fn get_checkers_bitboard(&self) -> PyBitboard {
    //     PyBitboard(*self.board.checkers())
    // }

    // /// Get the bitboard of all the pieces of a certain color.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> board.make_move(rust_chess.Move("e2e4"))
    // /// >>> board.get_color_bitboard(rust_chess.WHITE).popcnt()
    // /// 16
    // /// >>> board.get_color_bitboard(rust_chess.WHITE)
    // /// X X X X X X X X
    // /// X X X X . X X X
    // /// . . . . . . . .
    // /// . . . . X . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// ```
    // #[inline]
    // fn get_color_bitboard(&self, color: PyColor) -> PyBitboard {
    //     PyBitboard(*self.board.color_combined(color.0))
    // }

    // /// Get the bitboard of all the pieces of a certain type.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> board.make_move(rust_chess.Move("e2e4"))
    // /// >>> board.get_piece_type_bitboard(rust_chess.PAWN).popcnt()
    // /// 16
    // /// >>> board.get_piece_type_bitboard(rust_chess.PAWN)
    // /// . . . . . . . .
    // /// X X X X . X X X
    // /// . . . . . . . .
    // /// . . . . X . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// X X X X X X X X
    // /// . . . . . . . .
    // /// ```
    // #[inline]
    // fn get_piece_type_bitboard(&self, piece_type: PyPieceType) -> PyBitboard {
    //     PyBitboard(*self.board.pieces(piece_type.0))
    // }

    // /// Get the bitboard of all the pieces of a certain color and type.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> board.make_move(rust_chess.Move("e2e4"))
    // /// >>> board.get_piece_bitboard(rust_chess.WHITE_PAWN).popcnt()
    // /// 8
    // /// >>> board.get_piece_bitboard(rust_chess.WHITE_PAWN)
    // /// . . . . . . . .
    // /// X X X X . X X X
    // /// . . . . . . . .
    // /// . . . . X . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// ```
    // #[inline]
    // fn get_piece_bitboard(&self, piece: PyPiece) -> PyBitboard {
    //     PyBitboard(self.board.pieces(piece.piece_type.0) & self.board.color_combined(piece.color.0))
    // }

    // /// Get the bitboard of all the pieces.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> board.make_move(rust_chess.Move("e2e4"))
    // /// >>> board.get_all_bitboard().popcnt()
    // /// 32
    // /// >>> board.get_all_bitboard()
    // /// X X X X X X X X
    // /// X X X X . X X X
    // /// . . . . . . . .
    // /// . . . . X . . .
    // /// . . . . . . . .
    // /// . . . . . . . .
    // /// X X X X X X X X
    // /// X X X X X X X X
    // /// ```
    // #[inline]
    // fn get_all_bitboard(&self) -> PyBitboard {
    //     PyBitboard(*self.board.combined())
    // }

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

    // /// Checks if the halfmoves since the last pawn move or capture is >= 100
    // /// and the game is ongoing (not checkmate or stalemate).
    // ///
    // /// This is a claimable draw according to FIDE rules.
    // ///
    // /// ```python
    // /// >>> rust_chess.Board().is_fifty_moves()
    // /// False
    // /// >>> rust_chess.Board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 100 1").is_fifty_moves()
    // /// True
    // /// ```
    // #[inline]
    // fn is_fifty_moves(&self) -> bool {
    //     self.halfmove_clock >= 100 && self.board.status() == chess::BoardStatus::Ongoing
    // }

    // /// Checks if the halfmoves since the last pawn move or capture is >= 150
    // /// and the game is ongoing (not checkmate or stalemate).
    // ///
    // /// This is an automatic draw according to FIDE rules.
    // ///
    // /// ```python
    // /// >>> rust_chess.Board().is_seventy_five_moves()
    // /// False
    // /// >>> rust_chess.Board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 150 1").is_seventy_five_moves()
    // /// True
    // /// ```
    // #[inline]
    // fn is_seventy_five_moves(&self) -> bool {
    //     self.halfmove_clock >= 150 && self.board.status() == chess::BoardStatus::Ongoing
    // }

    // /// Checks if the side to move has insufficient material to checkmate the opponent.
    // /// The cases where this is true are:
    // ///     1. K vs K
    // ///     2. K vs K + N
    // ///     3. K vs K + B
    // ///     4. K + B vs K + B with the bishops on the same color.
    // ///
    // /// ```python
    // /// >>> rust_chess.Board().is_insufficient_material()
    // /// False
    // /// >>> rust_chess.Board("4k3/8/8/8/8/8/8/4K3 w - - 0 1").is_insufficient_material() # K vs K
    // /// True
    // /// >>> rust_chess.Board("4k3/8/8/8/5N2/8/8/4K3 w - - 0 1").is_insufficient_material() # K vs K + N
    // /// True
    // /// >>> rust_chess.Board("4k3/8/8/8/5B2/8/8/4K3 w - - 0 1").is_insufficient_material() # K vs K + B
    // /// True
    // /// >>> rust_chess.Board("4k3/8/8/5b2/5B2/8/8/4K3 w - - 0 1").is_insufficient_material() # K + B vs K + B different color
    // /// False
    // /// >>> rust_chess.Board("4k3/8/5b2/8/5B2/8/8/4K3 w - - 0 1").is_insufficient_material() # K + B vs K + B same color
    // /// True
    // /// ```
    // #[inline]
    // fn is_insufficient_material(&self) -> bool {
    //     let kings = self.board.pieces(chess::Piece::King);

    //     // Get the bitboards of the white and black pieces without the kings
    //     let white_bb = self.board.color_combined(chess::Color::White) & !kings;
    //     let black_bb = self.board.color_combined(chess::Color::Black) & !kings;
    //     let combined_bb = white_bb | black_bb;

    //     // King vs King: Combined bitboard minus kings is empty
    //     if combined_bb == chess::EMPTY {
    //         return true;
    //     }

    //     let num_remaining_pieces = combined_bb.popcnt();
    //     if num_remaining_pieces <= 2 {
    //         let knights = self.board.pieces(chess::Piece::Knight);
    //         let bishops = self.board.pieces(chess::Piece::Bishop);

    //         // King vs King + Knight/Bishop: Combined bitboard minus kings and knight/bishop is empty
    //         if num_remaining_pieces == 1 && combined_bb & !(knights | bishops) == chess::EMPTY {
    //             return true;
    //         } else if *knights == chess::EMPTY {
    //             // Only bishops left
    //             let white_bishops = bishops & white_bb;
    //             let black_bishops = bishops & black_bb;

    //             // Both sides have a bishop
    //             if white_bishops != chess::EMPTY && black_bishops != chess::EMPTY {
    //                 let white_bishop_index = white_bishops.to_square().to_index();
    //                 let black_bishop_index = black_bishops.to_square().to_index();

    //                 // King + Bishop vs King + Bishop same color: White and black bishops are on the same color square
    //                 return ((9 * (white_bishop_index ^ black_bishop_index)) & 8) == 0; // Check if square colors are the same
    //             }
    //         }
    //     }
    //     false
    // }

    // /// Checks if the current position is a n-fold repetition.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> board.is_n_repetition(4)  # Check for fourfold repetition
    // /// False
    // /// >>> for _ in range(3):
    // /// ...     board.make_move(rust_chess.Move("g1f3"))
    // /// ...     board.make_move(rust_chess.Move("b8c6"))
    // /// ...     board.make_move(rust_chess.Move("f3g1"))
    // /// ...     board.make_move(rust_chess.Move("c6b8"))
    // /// >>> board.is_n_repetition(4)  # Check for fourfold repetition
    // /// True
    // /// >>> board.board_history.count(board.zobrist_hash)  # Position appears 4 times
    // /// 4
    // /// ```
    // ///
    // /// TODO: Quick check (only check last few moves since that is common error for engines)
    // /// TODO: Add option to use full, or no repetition checks
    // #[inline]
    // fn is_n_repetition(&self, n: u8) -> bool {
    //     if let Some(history) = &self.board_history {
    //         // Move history length is one greater than the halfmove clock since when halfmove clock is 0, there is 1 position in history
    //         let length: i16 = i16::from(self.halfmove_clock + 1);
    //         // If checking threefold (n = 3), then it would be (4 * (3-1)) + 1 = 9
    //         // Fivefold requires 17 positions minimum
    //         //   Takes 4 halfmoves to return to a position
    //         let calc_min_pos_req_for_nfold = |n: u8| -> i16 { i16::from((4 * (n - 1)) + 1) };

    //         // n-fold repetition is not possible when length is less than (n * 4) - 1
    //         // For example, threefold repetition (n=3) can occur with a move history length minimum of 9
    //         // A color cannot repeat a position back to back--some move has to be made, and then another to return to the position
    //         // Example: index 0, 4, 8 are the minimum required for a threefold repetition
    //         //   (2 and 6 are in-between positions that allow returning to repeated position (0, 4, 8))
    //         if length < calc_min_pos_req_for_nfold(n) {
    //             return false;
    //         }

    //         #[allow(clippy::cast_sign_loss)]
    //         let current_hash: u64 = history[length as usize - 1];
    //         let mut num_repetitions: u8 = 1;

    //         // (length - 5) since we compare to current, which is at length - 1, and positions can't repeat back-to-back for a color
    //         let mut i: i16 = length - 5;
    //         // n-fold still possible if enough positions still left in history
    //         while i >= calc_min_pos_req_for_nfold(n - num_repetitions) - 1 {
    //             #[allow(clippy::cast_sign_loss)]
    //             if history[i as usize] == current_hash {
    //                 num_repetitions += 1;
    //                 if num_repetitions >= n {
    //                     return true;
    //                 }

    //                 // Can subtract another 2 here since if we found a repetition, our position before can't be the same
    //                 i -= 2;
    //             }

    //             // Step by 2 since only need to check our moves
    //             i -= 2;
    //         }
    //     }

    //     false
    // }

    // /// Checks if the current position is a threefold repetition.
    // /// This is a claimable draw according to FIDE rules.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> board.is_threefold_repetition()
    // /// False
    // /// >>> for _ in range(2):
    // /// ...     board.make_move(rust_chess.Move("g1f3"))
    // /// ...     board.make_move(rust_chess.Move("b8c6"))
    // /// ...     board.make_move(rust_chess.Move("f3g1"))
    // /// ...     board.make_move(rust_chess.Move("c6b8"))
    // /// >>> board.is_threefold_repetition()
    // /// True
    // /// >>> board.board_history.count(board.zobrist_hash)  # Position has appeared 3 times
    // /// 3
    // /// ```
    // #[inline]
    // fn is_threefold_repetition(&self) -> bool {
    //     self.is_n_repetition(3)
    // }

    // /// Checks if the current position is a fivefold repetition.
    // /// This is an automatic draw according to FIDE rules.
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> board.is_fivefold_repetition()
    // /// False
    // /// >>> for _ in range(4):
    // /// ...     board.make_move(rust_chess.Move("g1f3"))
    // /// ...     board.make_move(rust_chess.Move("b8c6"))
    // /// ...     board.make_move(rust_chess.Move("f3g1"))
    // /// ...     board.make_move(rust_chess.Move("c6b8"))
    // /// >>> board.is_fivefold_repetition()
    // /// True
    // /// >>> board.board_history.count(board.zobrist_hash)  # Position has appeared 5 times
    // /// 5
    // /// ```
    // #[inline]
    // fn is_fivefold_repetition(&self) -> bool {
    //     self.is_n_repetition(5)
    // }

    // /// Checks if the side to move is in check.
    // ///
    // /// ```python
    // /// >>> rust_chess.Board().is_check()
    // /// False
    // /// >>> rust_chess.Board("rnb1kbnr/pppp1ppp/4p3/8/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3").is_check()
    // /// True
    // /// ```
    // #[inline]
    // fn is_check(&self) -> bool {
    //     *self.board.checkers() != chess::EMPTY
    // }

    // // TODO: Docs

    // /// Checks if the side to move is in stalemate
    // ///
    // /// ```python
    // /// >>> rust_chess.Board().is_stalemate()
    // /// False
    // /// ```
    // /// TODO
    // #[inline]
    // fn is_stalemate(&self) -> bool {
    //     self.board.status() == chess::BoardStatus::Stalemate
    // }

    // /// Checks if the side to move is in checkmate
    // ///
    // /// ```python
    // /// >>> rust_chess.Board().is_checkmate()
    // /// False
    // /// ```
    // /// TODO
    // #[inline]
    // fn is_checkmate(&self) -> bool {
    //     self.board.status() == chess::BoardStatus::Checkmate
    // }

    // /// Get the status of the board (ongoing, draw, or game-ending).
    // ///
    // /// ```python
    // /// >>> board = rust_chess.Board()
    // /// >>> board.get_status()
    // /// BoardStatus.ONGOING
    // /// ```
    // /// TODO
    // #[inline]
    // fn get_status(&self) -> PyBoardBatchStatus {
    //     match self.board.status() {
    //         chess::BoardStatus::Ongoing => {
    //             if self.is_seventy_five_moves() {
    //                 PyBoardBatchStatus::SeventyFiveMoves
    //             } else if self.is_insufficient_material() {
    //                 PyBoardBatchStatus::InsufficientMaterial
    //             } else if self.is_fivefold_repetition() {
    //                 PyBoardBatchStatus::FiveFoldRepetition
    //             } else {
    //                 PyBoardBatchStatus::Ongoing
    //             }
    //         }
    //         chess::BoardStatus::Stalemate => PyBoardBatchStatus::Stalemate,
    //         chess::BoardStatus::Checkmate => PyBoardBatchStatus::Checkmate,
    //     }
    // }
}

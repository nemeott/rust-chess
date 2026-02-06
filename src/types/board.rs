use std::fmt::Write;
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

// TODO: Comparision and partial ord (use Zobrist?)
// TODO: Get castle rights

/// Board status enum class.
/// Represents the status of a chess board.
/// The status can be one of the following:
///     Ongoing, seventy-five moves, five-fold repetition, insufficient material, stalemate, or checkmate.
/// Supports comparison and equality.
/// TODO: docs
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

/// Castle rights enum class..
/// The castle rights can be one of the following:
///     No rights, king-side, queen-side, both.
/// TODO: docs
#[gen_stub_pyclass_enum]
#[pyclass(name = "CastleRights", frozen, eq, ord)]
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub(crate) enum PyCastleRights {
    #[pyo3(name = "NO_RIGHTS")]
    NoRights,
    #[pyo3(name = "KING_SIDE")]
    KingSide,
    #[pyo3(name = "QUEEN_SIDE")]
    QueenSide,
    #[pyo3(name = "BOTH")]
    Both,
}

// TODO: Check when making move?
#[gen_stub_pyclass_enum]
#[pyclass(name = "RepetitionDetectionMode", frozen, eq, ord)]
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub(crate) enum PyRepetitionDetectionMode {
    #[pyo3(name = "NONE")]
    None,
    #[pyo3(name = "PARTIAL")]
    Partial,
    #[pyo3(name = "FULL")]
    Full,
}

/// Board class.
/// Represents the state of a chess board.
///
/// TODO: docs
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
    // (theoretical maximum is 218 moves (fits within 2^8=256))
    /// The repetition dectection mode the board will use.
    #[pyo3(get)]
    repetition_detection_mode: PyRepetitionDetectionMode,

    /// Store board Zobrist hashes for move history
    #[pyo3(get)]
    move_history: Option<Vec<u64>>,
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
    #[pyo3(signature = (fen = None, mode = PyRepetitionDetectionMode::Full))] // Default to no fen and full repetition detection
    fn new(fen: Option<&str>, mode: PyRepetitionDetectionMode) -> PyResult<Self> {
        match fen {
            // If no FEN string is provided, use the default starting position
            None => {
                let board = chess::Board::default();

                // We can assume the GIL is acquired, since this function is only called from Python
                let py = unsafe { Python::assume_attached() };

                // Create a new move generator using the chess crate
                let move_gen = Py::new(py, PyMoveGenerator(chess::MoveGen::new_legal(&board)))?;

                // Create move history vector and add the initial board hash
                let mut move_history = match mode {
                    PyRepetitionDetectionMode::None => None,
                    PyRepetitionDetectionMode::Partial => Some(Vec::with_capacity(16)), // TODO: Change to deque
                    PyRepetitionDetectionMode::Full => Some(Vec::with_capacity(256)),
                };
                if let Some(history) = &mut move_history {
                    history.push(board.get_hash());
                }

                Ok(PyBoard {
                    board,
                    move_gen,
                    halfmove_clock: 0,
                    fullmove_number: 1,
                    repetition_detection_mode: mode,
                    move_history: move_history,
                })
            }
            // Otherwise, parse the FEN string using the chess crate
            Some(fen_str) => PyBoard::from_fen(fen_str, mode),
        }
    }

    /// Create a new board from a FEN string.
    ///
    /// ```python
    /// >>> rust_chess.Board.from_fen("rnbqkbnr/ppp1pppp/8/3p4/2P1P3/8/PP1P1PPP/RNBQKBNR b KQkq - 0 2")
    /// rnbqkbnr/ppp1pppp/8/3p4/2P1P3/8/PP1P1PPP/RNBQKBNR b KQkq - 0 2
    /// ```
    #[staticmethod]
    #[pyo3(signature = (fen, mode = PyRepetitionDetectionMode::Full))] // Default to no fen and full repetition detection
    fn from_fen(fen: &str, mode: PyRepetitionDetectionMode) -> PyResult<Self> {
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

        // Create move history vector and add the initial board hash
        let mut move_history = match mode {
            PyRepetitionDetectionMode::None => None,
            PyRepetitionDetectionMode::Partial => Some(Vec::with_capacity(16)), // TODO: Change to deque
            PyRepetitionDetectionMode::Full => Some(Vec::with_capacity(256)),
        };
        if let Some(history) = &mut move_history {
            history.push(board.get_hash());
        }

        Ok(PyBoard {
            board,
            move_gen,
            halfmove_clock,
            fullmove_number,
            repetition_detection_mode: mode,
            move_history: move_history,
        })
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
    /// >>> rust_chess.Board()
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    /// ```
    #[inline]
    fn __repr__(&self) -> String {
        self.get_fen()
    }

    /// Get the string representation of the board.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> print(board.display())
    /// r n b q k b n r
    /// p p p p p p p p
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// P P P P P P P P
    /// R N B Q K B N R
    ///
    /// ```
    #[inline]
    fn display(&self) -> String {
        let mut s = String::new();
        for rank in (0..8).rev() {
            for file in 0..8 {
                let square = PySquare(unsafe { chess::Square::new(file + (rank * 8)) });
                if let Some(piece) = self.get_piece_on(square) {
                    unsafe { write!(s, "{} ", &piece.get_string()).unwrap_unchecked() }; // Safe code is for weaklings
                } else {
                    unsafe { write!(s, ". ").unwrap_unchecked() };
                }
            }
            unsafe { write!(s, "\n").unwrap_unchecked() };
        }
        s
    }

    /// Get the string representation of the board.
    ///
    /// ```python
    /// >>> print(rust_chess.Board())
    /// r n b q k b n r
    /// p p p p p p p p
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// P P P P P P P P
    /// R N B Q K B N R
    ///
    /// ```
    #[inline]
    fn __str__(&self) -> String {
        self.display()
    }

    /// Get the unicode string representation of the board.
    ///
    /// The dark mode parameter is enabled by default.
    /// This inverts the color of the piece, which looks correct on a dark background.
    /// Unicode assumes black text on white background, where in most terminals, it is the opposite.
    /// Disable if you are a psychopath and use light mode in your terminal/IDE.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> print(board.display_unicode())
    /// ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖
    /// ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙
    /// · · · · · · · ·
    /// · · · · · · · ·
    /// · · · · · · · ·
    /// · · · · · · · ·
    /// ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
    /// ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    ///
    /// >>> print(board.display_unicode(dark_mode=False))
    /// ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    /// ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
    /// · · · · · · · ·
    /// · · · · · · · ·
    /// · · · · · · · ·
    /// · · · · · · · ·
    /// ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙
    /// ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖
    ///
    /// ```
    #[inline]
    #[pyo3(signature = (dark_mode = true))]
    fn display_unicode(&self, dark_mode: bool) -> String {
        let mut s = String::new();
        for rank in (0..8).rev() {
            for file in 0..8 {
                let square = PySquare(unsafe { chess::Square::new(file + (rank * 8)) });
                if let Some(piece) = self.get_piece_on(square) {
                    unsafe { write!(s, "{} ", &piece.get_unicode(dark_mode)).unwrap_unchecked() }; // Safe code is for weaklings
                } else {
                    unsafe { write!(s, "· ").unwrap_unchecked() }; // This is a unicode middle dot, not a period
                }
            }
            unsafe { write!(s, "\n").unwrap_unchecked() };
        }
        s
    }

    // Get the Zobrist hash of the board
    //
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.zobrist_hash
    /// 9023329949471135578
    /// >>> board.make_move(rust_chess.Move("e2e4"))
    /// >>> board.zobrist_hash
    /// 9322854110900140515
    /// ```
    #[getter]
    #[inline]
    fn get_zobrist_hash(&self) -> u64 {
        self.board.get_hash()
    }

    /// Get the hash of the board based on its Zobrist hash.
    /// **This is not the same as the `zobrist_hash` field since Python doesn't support unsigned 64-bit integers for this function.**
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> hash(board)
    /// 9023329949471135578
    /// >>> board.make_move(rust_chess.Move("e2e4"))
    /// >>> hash(board)
    /// -9123889962809411101
    /// >>> board.zobrist_hash
    /// 9322854110900140515
    /// >>> hash(board) == board.zobrist_hash
    /// False
    /// ```
    #[inline]
    fn __hash__(&self) -> u64 {
        self.get_zobrist_hash()
    }

    /// Check if two boards are equal based on their Zobrist hash.
    ///
    /// ```python
    /// >>> board1 = rust_chess.Board()
    /// >>> board2 = rust_chess.Board()
    /// >>> board1 == board2
    /// True
    /// >>> board1.make_move(rust_chess.Move("e2e4"))
    /// >>> board1 == board2
    /// False
    /// ```
    #[inline]
    fn __eq__(&self, other: &PyBoard) -> bool {
        self.get_zobrist_hash() == other.get_zobrist_hash()
    }

    /// Check if two boards are not equal based on their Zobrist hash.
    ///
    /// ```python
    /// >>> board1 = rust_chess.Board()
    /// >>> board2 = rust_chess.Board()
    /// >>> board1 != board2
    /// False
    /// >>> board1.make_move(rust_chess.Move("e2e4"))
    /// >>> board1 != board2
    /// True
    /// ```
    #[inline]
    fn __ne__(&self, other: &PyBoard) -> bool {
        !self.__eq__(&other)
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

    /// Get the castle rights of a color.
    /// Returns a `CastlingRights` enum type, which has values: NO_RIGHTS, KING_SIDE, QUEEN_SIDE, BOTH.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.get_castle_rights(board.turn)
    /// CastleRights.BOTH
    /// >>> board = rust_chess.Board("r6r/4k3/8/8/8/8/7R/R3K3 w Q - 2 2")
    /// >>> board.get_castle_rights(rust_chess.WHITE)
    /// CastleRights.QUEEN_SIDE
    /// >>> board.get_castle_rights(rust_chess.BLACK)
    /// CastleRights.NO_RIGHTS
    /// ```
    #[inline]
    fn get_castle_rights(&self, color: PyColor) -> PyCastleRights {
        match self.board.castle_rights(color.0) {
            chess::CastleRights::NoRights => PyCastleRights::NoRights,
            chess::CastleRights::KingSide => PyCastleRights::KingSide,
            chess::CastleRights::QueenSide => PyCastleRights::QueenSide,
            chess::CastleRights::Both => PyCastleRights::Both,
        }
    }

    /// Get the castle rights of the current player to move.
    ///
    /// ```python
    /// >>> rust_chess.Board().get_my_castle_rights()
    /// CastleRights.BOTH
    /// >>> board = rust_chess.Board("r6r/4k3/8/8/8/8/7R/R3K3 w Q - 2 2")
    /// >>> board.get_my_castle_rights()  # White to move
    /// CastleRights.QUEEN_SIDE
    /// ```
    #[inline]
    fn get_my_castle_rights(&self) -> PyCastleRights {
        self.get_castle_rights(self.get_turn())
    }

    /// Get the castle rights of the opponent.
    ///
    /// ```python
    /// >>> rust_chess.Board().get_their_castle_rights()
    /// CastleRights.BOTH
    /// >>> board = rust_chess.Board("r6r/4k3/8/8/8/8/7R/R3K3 w Q - 2 2")
    /// >>> board.get_their_castle_rights()  # White to move (so black is opponent)
    /// CastleRights.NO_RIGHTS
    /// ```
    #[inline]
    fn get_their_castle_rights(&self) -> PyCastleRights {
        self.get_castle_rights(PyColor(!self.board.side_to_move()))
    }

    /// Check if a color can castle (either side).
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.can_castle(board.turn)
    /// True
    /// >>> board = rust_chess.Board("r6r/4k3/8/8/8/8/7R/R3K3 w Q - 2 2")
    /// >>> board.can_castle(rust_chess.WHITE)
    /// True
    /// >>> board.can_castle(rust_chess.BLACK)
    /// False
    /// ```
    #[inline]
    fn can_castle(&self, color: PyColor) -> bool {
        self.board.castle_rights(color.0) != chess::CastleRights::NoRights
    }

    /// Check if a color can castle kingside.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.can_castle_kingside(board.turn)
    /// True
    /// >>> board = rust_chess.Board("r6r/4k3/8/8/8/8/7R/R3K3 w Q - 2 2")
    /// >>> board.can_castle_kingside(rust_chess.WHITE)
    /// False
    /// ```
    #[inline]
    fn can_castle_kingside(&self, color: PyColor) -> bool {
        self.board.castle_rights(color.0).has_kingside()
    }

    /// Check if a color can castle queenside.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.can_castle_queenside(board.turn)
    /// True
    /// >>> board = rust_chess.Board("r6r/4k3/8/8/8/8/7R/R3K3 w Q - 2 2")
    /// >>> board.can_castle_queenside(rust_chess.WHITE)
    /// True
    /// >>> board.can_castle_queenside(rust_chess.BLACK)
    /// False
    /// ```
    #[inline]
    fn can_castle_queenside(&self, color: PyColor) -> bool {
        self.board.castle_rights(color.0).has_queenside()
    }

    /// Check if a move is castling.
    /// Assumes the move is pseudo-legal.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.is_castling(rust_chess.Move("e1g1"))
    /// True
    /// >>> board.is_castling(rust_chess.Move("e1e2"))
    /// False
    /// ```
    #[inline]
    fn is_castling(&self, chess_move: PyMove) -> bool {
        let source = chess_move.0.get_source();

        // Check if the moving piece is a king
        if self
            .board
            .piece_on(source)
            .is_some_and(|p| p == chess::Piece::King)
        {
            // Check if the move is two squares horizontally
            let dest = chess_move.0.get_dest();
            return (dest.to_index() as i8 - source.to_index() as i8).abs() == 2;
        }
        return false;
    }

    /// Check if a move is kingside castling.
    /// Assumes the move is pseudo-legal.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.is_castling_kingside(rust_chess.Move("e1g1"))
    /// True
    /// >>> board.is_castling_kingside(rust_chess.Move("e1c1"))
    /// False
    /// ```
    #[inline]
    fn is_castling_kingside(&self, chess_move: PyMove) -> bool {
        let source = chess_move.0.get_source();

        // Check if the moving piece is a king
        if self
            .board
            .piece_on(source)
            .is_some_and(|p| p == chess::Piece::King)
        {
            // Check if the move is two squares to the right
            let dest = chess_move.0.get_dest();
            return dest.to_index() as i8 - source.to_index() as i8 == 2;
        }
        return false;
    }

    /// Check if a move is queenside castling.
    /// Assumes the move is pseudo-legal.    
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.is_castling_queenside(rust_chess.Move("e1c1"))
    /// True
    /// >>> board.is_castling_queenside(rust_chess.Move("e1g1"))
    /// False
    /// ```
    #[inline]
    fn is_castling_queenside(&self, chess_move: PyMove) -> bool {
        let source = chess_move.0.get_source();

        // Check if the moving piece is a king
        if self
            .board
            .piece_on(source)
            .is_some_and(|p| p == chess::Piece::King)
        {
            // Check if the move is two squares to the left
            let dest = chess_move.0.get_dest();
            return dest.to_index() as i8 - source.to_index() as i8 == -2;
        }
        return false;
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

    // TODO: make_null_move (would require move history to undo (probably?))

    /// Make a null move onto a new board.
    /// Returns None if the current player is in check.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    /// >>> new_board = board.make_null_move_new()
    /// >>> new_board
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

        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        Ok(Some(PyBoard {
            board: new_board,
            // Create a new move generator using the chess crate
            move_gen: Py::new(py, PyMoveGenerator(chess::MoveGen::new_legal(&new_board)))?,
            // // Increment the halfmove clock
            halfmove_clock: self.halfmove_clock + 1, // Null moves aren't zeroing, so we can just add 1 here
            // // Increment fullmove number if black moves
            fullmove_number: self.fullmove_number + (self.board.side_to_move().to_index() as u8), // White is 0, black is 1
            repetition_detection_mode: self.repetition_detection_mode,
            // Don't update move history when making a null move
            move_history: self.move_history.clone(),
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
    /// r n b q k b n r
    /// p p p p p p p p
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . P . . .
    /// . . . . . . . .
    /// P P P P . P P P
    /// R N B Q K B N R
    ///
    /// ```
    #[pyo3(signature = (chess_move, check_legality = true))]
    #[inline]
    fn make_move(&mut self, chess_move: PyMove, check_legality: bool) -> PyResult<()> {
        // Check if draw by fivefold
        if self.is_fivefold_repetition() {
            return Err(PyValueError::new_err(
                "Game over due to fivefold repetition",
            ));
        }

        // If we are checking legality, check if the move is legal
        if check_legality && !self.is_legal_move(chess_move) {
            return Err(PyValueError::new_err("Illegal move"));
        }

        // Make the move onto a new board using the chess crate
        let temp_board: chess::Board = self.board.make_move_new(chess_move.0);

        // Reset the halfmove clock if the move zeroes (is a capture or pawn move and therefore "zeroes" the halfmove clock)
        if self.is_zeroing(chess_move) {
            self.halfmove_clock = 0;

            // Don't need previous history anymore since it is a zeroing move (irreversible)
            if let Some(history) = &mut self.move_history {
                history.clear();
            }
        } else {
            self.halfmove_clock += 1 // Add one if not zeroing
        };

        // Increment fullmove number if black moves
        self.fullmove_number += self.board.side_to_move().to_index() as u8; // White is 0, black is 1

        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        // Create a new move generator using the chess crate
        self.move_gen = Py::new(py, PyMoveGenerator(chess::MoveGen::new_legal(&temp_board)))?;

        // Update the current board
        self.board = temp_board;

        // Add the new board's Zobrist hash to history
        if let Some(history) = &mut self.move_history {
            history.push(temp_board.get_hash())
        }

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
    /// r n b q k b n r
    /// p p p p p p p p
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . P . . .
    /// . . . . . . . .
    /// P P P P . P P P
    /// R N B Q K B N R
    ///
    /// >>> print(old_board)
    /// r n b q k b n r
    /// p p p p p p p p
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// P P P P P P P P
    /// R N B Q K B N R
    ///
    /// ```
    #[pyo3(signature = (chess_move, check_legality = true))]
    #[inline]
    fn make_move_new(&self, chess_move: PyMove, check_legality: bool) -> PyResult<Self> {
        // Check if draw by fivefold
        if self.is_fivefold_repetition() {
            return Err(PyValueError::new_err(
                "Game over due to fivefold repetition",
            ));
        }

        // If we are checking legality, check if the move is legal
        if check_legality && !self.is_legal_move(chess_move) {
            return Err(PyValueError::new_err("Illegal move"));
        }

        // Make the move onto a new board using the chess crate
        let new_board: chess::Board = self.board.make_move_new(chess_move.0);

        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        let is_zeroing: bool = self.is_zeroing(chess_move);

        Ok(PyBoard {
            board: new_board,
            move_gen: Py::new(py, PyMoveGenerator(chess::MoveGen::new_legal(&new_board)))?,
            // Reset the halfmove clock if the move zeroes (is a capture or pawn move and therefore "zeroes" the halfmove clock)
            halfmove_clock: if is_zeroing {
                0
            } else {
                self.halfmove_clock + 1
            },
            // Increment fullmove number if black moves
            fullmove_number: self.fullmove_number + (self.board.side_to_move().to_index() as u8), // White is 0, black is 1
            repetition_detection_mode: self.repetition_detection_mode,
            // Add the new board's Zobrist hash to history
            move_history: self.move_history.as_ref().map(|history| {
                let mut new_history = if is_zeroing {
                    Vec::with_capacity(history.capacity()) // Don't need previous history anymore since it is a zeroing move (irreversible)
                } else {
                    history.clone()
                };

                new_history.push(new_board.get_hash());
                new_history
            }),
        })
    }

    /// Get the bitboard of the side to move's pinned pieces.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.get_pinned_bitboard().popcnt()
    /// 0
    ///
    /// >>> board.make_move(rust_chess.Move("e2e4"))
    /// >>> board.make_move(rust_chess.Move("d7d5"))
    /// >>> board.make_move(rust_chess.Move("d1h5"))
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
    /// >>> board.make_move(rust_chess.Move("e2e4"))
    /// >>> board.make_move(rust_chess.Move("f7f6"))
    /// >>> board.make_move(rust_chess.Move("d1h5"))
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

    // FIXME: Sometimes consumes the entire generator (length -> 0) (maybe only when using next move?)
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
    /// >>> board.remove_generator_move(rust_chess.Move("a2a3"))  # FIXME: Currently makes len -> 0
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
    /// >>> board.set_generator_mask(rust_chess.Bitboard(402915328))
    /// >>> len(board.generate_moves())
    /// 4
    /// >>> list(board.generate_moves())
    /// [Move(c2, c3, None), Move(d2, d4, None), Move(e2, e4, None), Move(b1, c3, None)]
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

    /// Checks if the current position is a n-fold repetition.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.is_n_repetition(4)  # Check for fourfold repetition
    /// False
    /// >>> for _ in range(3):
    /// ...     board.make_move(rust_chess.Move("g1f3"))
    /// ...     board.make_move(rust_chess.Move("b8c6"))
    /// ...     board.make_move(rust_chess.Move("f3g1"))
    /// ...     board.make_move(rust_chess.Move("c6b8"))
    /// >>> board.is_n_repetition(4)  # Check for fourfold repetition
    /// True
    /// >>> board.move_history.count(board.zobrist_hash)  # Position appears 4 times
    /// 4
    /// ```
    ///
    /// TODO: Quick check (only check last few moves since that is common error for engines)
    /// TODO: Add option to use full, partial, or no repetition checks
    #[inline]
    fn is_n_repetition(&self, n: u8) -> bool {
        if let Some(history) = &self.move_history {
            // Move history length is one greater than the halfmove clock since when halfmove clock is 0, there is 1 position in history
            let length: usize = (self.halfmove_clock + 1) as usize;
            // If checking threefold (n = 3), then it would be (4 * (3-1)) + 1 = 9
            // Fivefold requires 17 positions minimum
            //   Takes 4 halfmoves to return to a position
            let calc_min_pos_req_for_nfold = |n: u8| -> usize { ((4 * (n - 1)) + 1) as usize };

            // n-fold repetition is not possible when length is less than (n * 4) - 1
            // For example, threefold repetition (n=3) can occur with a move history length minimum of 9
            // A color cannot repeat a position back to back--some move has to be made, and then another to return to the position
            // Example: index 0, 4, 8 are the minimum required for a threefold repetition
            //   (2 and 6 are in-between positions that allow returning to repeated position (0, 4, 8))
            if length < calc_min_pos_req_for_nfold(n) {
                return false;
            }

            let current_hash: u64 = history[length - 1];
            let mut num_repetitions: u8 = 1;

            // (length - 5) since we compare to current, which is at length - 1, and positions can't repeat back-to-back for a color
            let mut i: usize = length - 5;
            // n-fold still possible if enough positions still left in history
            while i >= calc_min_pos_req_for_nfold(n - num_repetitions) - 1 {
                if history[i] == current_hash {
                    num_repetitions += 1;
                    if num_repetitions >= n {
                        return true;
                    }

                    // Can subtract another 2 here since if we found a repetition, our position before can't be the same
                    i -= 2;
                }

                // Step by 2 since only need to check our moves
                i -= 2;
            }
        }

        false
    }

    /// Checks if the current position is a threefold repetition.
    /// This is a claimable draw according to FIDE rules.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.is_threefold_repetition()
    /// False
    /// >>> for _ in range(2):
    /// ...     board.make_move(rust_chess.Move("g1f3"))
    /// ...     board.make_move(rust_chess.Move("b8c6"))
    /// ...     board.make_move(rust_chess.Move("f3g1"))
    /// ...     board.make_move(rust_chess.Move("c6b8"))
    /// >>> board.is_threefold_repetition()
    /// True
    /// >>> board.move_history.count(board.zobrist_hash)  # Position has appeared 3 times
    /// 3
    /// ```
    #[inline]
    fn is_threefold_repetition(&self) -> bool {
        self.is_n_repetition(3)
    }

    /// Checks if the current position is a fivefold repetition.
    /// This is an automatic draw according to FIDE rules.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.is_fivefold_repetition()
    /// False
    /// >>> for _ in range(4):
    /// ...     board.make_move(rust_chess.Move("g1f3"))
    /// ...     board.make_move(rust_chess.Move("b8c6"))
    /// ...     board.make_move(rust_chess.Move("f3g1"))
    /// ...     board.make_move(rust_chess.Move("c6b8"))
    /// >>> board.is_fivefold_repetition()
    /// True
    /// >>> board.move_history.count(board.zobrist_hash)  # Position has appeared 5 times
    /// 5
    /// ```
    #[inline]
    fn is_fivefold_repetition(&self) -> bool {
        self.is_n_repetition(5)
    }

    // 3 -> 5 = +2
    // 4 -> 7 = +3
    // 5 -> 9 = +4

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

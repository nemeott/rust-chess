use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::types::color::{PyColor, WHITE};

// Piece constants
pub(crate) const PAWN: PyPieceType = PyPieceType(chess::Piece::Pawn);
pub(crate) const KNIGHT: PyPieceType = PyPieceType(chess::Piece::Knight);
pub(crate) const BISHOP: PyPieceType = PyPieceType(chess::Piece::Bishop);
pub(crate) const ROOK: PyPieceType = PyPieceType(chess::Piece::Rook);
pub(crate) const QUEEN: PyPieceType = PyPieceType(chess::Piece::Queen);
pub(crate) const KING: PyPieceType = PyPieceType(chess::Piece::King);
pub(crate) const PIECES: [PyPieceType; 6] = [PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING];

/// Piece type enum class.
/// Represents the different types of chess pieces.
/// Indexing starts at 0 (PAWN) and ends at 5 (KING).
/// Supports comparison and equality.
/// Does not include color.
///
/// `rust_chess` has constants for each piece type (e.g. PAWN, KNIGHT, etc.).
///
/// ```python
/// >>> piece = rust_chess.PAWN
///
/// >>> print(piece)
/// P
/// >>> piece == rust_chess.PAWN
/// True
/// >>> piece == rust_chess.KNIGHT
/// False
/// >>> piece.get_index()
/// 0
/// >>> piece < rust_chess.KNIGHT
/// True
/// ```
#[gen_stub_pyclass]
#[pyclass(name = "PieceType", frozen, eq, ord)]
#[derive(PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Hash)]
pub(crate) struct PyPieceType(pub(crate) chess::Piece);

#[gen_stub_pymethods]
#[pymethods]
impl PyPieceType {
    /// Get the index of the piece.
    /// Ranges from 0 (PAWN) to 5 (KING).
    ///
    /// ```python
    /// >>> rust_chess.BISHOP.get_index()
    /// 2
    /// ```
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn get_index(&self) -> u8 {
        self.0.to_index() as u8
    }

    /// Convert the piece to a string.
    /// Returns the capital piece type letter.
    ///
    /// ```python
    /// >>> rust_chess.PAWN.get_string()
    /// P
    /// ```
    #[inline]
    #[pyo3(signature = (color = WHITE))]
    fn get_string(&self, color: PyColor) -> String {
        self.0.to_string(color.0)
    }

    /// Convert the piece to a string.
    /// Returns the capital piece type letter.
    ///
    /// ```python
    /// >>> print(rust_chess.PAWN)
    /// P
    /// ```
    #[inline]
    fn __str__(&self) -> String {
        self.get_string(WHITE)
    }

    /// Convert the piece to a string.
    /// Returns the capital piece type letter.
    ///
    /// ```python
    /// >>> rust_chess.PAWN
    /// P
    /// ```
    #[inline]
    fn __repr__(&self) -> String {
        self.get_string(WHITE)
    }
}

/// Piece class.
/// Represents a chess piece with a type and color.
/// Uses the PieceType and Color classes.
/// Supports comparison and equality.
/// A white piece is considered less than a black piece of the same type.
///
/// ```python
/// TODO
/// ```
#[gen_stub_pyclass]
#[pyclass(name = "Piece", frozen, eq, ord)]
#[derive(PartialOrd, PartialEq, Eq, Copy, Clone, Hash)]
pub(crate) struct PyPiece {
    /// Get the piece type of the piece
    #[pyo3(get)]
    pub(crate) piece_type: PyPieceType,
    /// Get the color of the piece
    #[pyo3(get)]
    pub(crate) color: PyColor,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyPiece {
    /// Create a new piece from a piece type and color
    #[new]
    #[inline]
    fn new(piece_type: PyPieceType, color: PyColor) -> Self {
        PyPiece { piece_type, color }
    }

    /// Get the index of the piece (0-5)
    #[inline]
    fn get_index(&self) -> u8 {
        self.piece_type.get_index()
    }

    /// Convert the piece to a string
    #[inline]
    fn get_string(&self) -> String {
        self.piece_type.get_string(self.color)
    }

    /// Convert the piece to a string
    #[inline]
    fn __str__(&self) -> String {
        self.get_string()
    }

    /// Convert the piece to a string
    #[inline]
    fn __repr__(&self) -> String {
        self.get_string()
    }
}

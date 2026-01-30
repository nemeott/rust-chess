use pyo3::{exceptions::PyValueError, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::types::color::{PyColor, WHITE};

// Piece constants
pub(crate) const PAWN: PyPieceType = PyPieceType(chess::Piece::Pawn);
pub(crate) const KNIGHT: PyPieceType = PyPieceType(chess::Piece::Knight);
pub(crate) const BISHOP: PyPieceType = PyPieceType(chess::Piece::Bishop);
pub(crate) const ROOK: PyPieceType = PyPieceType(chess::Piece::Rook);
pub(crate) const QUEEN: PyPieceType = PyPieceType(chess::Piece::Queen);
pub(crate) const KING: PyPieceType = PyPieceType(chess::Piece::King);

pub(crate) const PIECE_TYPES: [PyPieceType; 6] = [PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING];

// Colored piece constants
#[rustfmt::skip]
pub(crate) mod pieces{
    use crate::{PyPiece, PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING, WHITE, BLACK};
    
    pub(crate) const WHITE_PAWN: PyPiece = PyPiece { piece_type: PAWN, color: WHITE };
    pub(crate) const WHITE_KNIGHT: PyPiece = PyPiece { piece_type: KNIGHT, color: WHITE };
    pub(crate) const WHITE_BISHOP: PyPiece = PyPiece { piece_type: BISHOP, color: WHITE };
    pub(crate) const WHITE_ROOK: PyPiece = PyPiece { piece_type: ROOK, color: WHITE };
    pub(crate) const WHITE_QUEEN: PyPiece = PyPiece { piece_type: QUEEN, color: WHITE };
    pub(crate) const WHITE_KING: PyPiece = PyPiece { piece_type: KING, color: WHITE };
    
    pub(crate) const BLACK_PAWN: PyPiece = PyPiece { piece_type: PAWN, color: BLACK };
    pub(crate) const BLACK_KNIGHT: PyPiece = PyPiece { piece_type: KNIGHT, color: BLACK };
    pub(crate) const BLACK_BISHOP: PyPiece = PyPiece { piece_type: BISHOP, color: BLACK };
    pub(crate) const BLACK_ROOK: PyPiece = PyPiece { piece_type: ROOK, color: BLACK };
    pub(crate) const BLACK_QUEEN: PyPiece = PyPiece { piece_type: QUEEN, color: BLACK };
    pub(crate) const BLACK_KING: PyPiece = PyPiece { piece_type: KING, color: BLACK };
    
    pub(crate) const PIECES: [PyPiece; 12] = [
        WHITE_PAWN, WHITE_KNIGHT, WHITE_BISHOP, WHITE_ROOK, WHITE_QUEEN, WHITE_KING,
        BLACK_PAWN, BLACK_KNIGHT, BLACK_BISHOP, BLACK_ROOK, BLACK_QUEEN, BLACK_KING,
    ];
}

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

    /// Allow the piece type to be used as an index.
    /// Returns the index of the piece.
    ///
    /// ```python
    /// >>> arr = [1, 2, 3, 4, 5, 6]
    /// >>> arr[rust_chess.BISHOP]
    /// 3
    /// ```
    #[inline]
    fn __index__(&self) -> u8 {
        self.get_index()
    }

    /// Convert the piece to a string.
    /// Returns the capital piece type letter by default.
    /// If using the optional color parameter, white is uppercase and black is lowercase.
    ///
    /// ```python
    /// >>> rust_chess.PAWN.get_string()
    /// 'P'
    /// >>> rust_chess.PAWN.get_string(rust_chess.BLACK)
    /// 'p'
    /// ```
    #[inline]
    #[pyo3(signature = (color = WHITE))] // Default piece color is white (capital letter)
    pub(crate) fn get_string(&self, color: PyColor) -> String {
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

    /// Convert the piece to a unicode string.
    /// Returns the hollow unicode piece by default.
    /// If using the optional color parameter, white is hollow and black is full.
    ///
    /// ```python
    /// >>> rust_chess.PAWN.get_unicode()
    /// '♙'
    /// >>> rust_chess.PAWN.get_unicode(rust_chess.BLACK)
    /// '♟'
    /// ```
    #[rustfmt::skip]
    #[inline]
    #[pyo3(signature = (color = WHITE))] // Default piece color is white (hollow)
    fn get_unicode(&self, color: PyColor) -> &'static str {
        match self.get_string(color).as_str() {
            "P" => "♙", "p" => "♟",
            "N" => "♘", "n" => "♞",
            "B" => "♗", "b" => "♝",
            "R" => "♖", "r" => "♜",
            "Q" => "♕", "q" => "♛",
            "K" => "♔", "k" => "♚",
            _ => "",
        }
    }
}

/// Piece class.
/// Represents a chess piece with a type and color.
/// Uses the PieceType and Color classes.
/// Supports comparison and equality.
/// A white piece is considered less than a black piece of the same type.
///
/// ```python
/// >>> piece = rust_chess.WHITE_PAWN
/// >>> piece
/// P
/// >>> piece.piece_type
/// P
/// >>> piece.color
/// True
/// ```
/// TODO
#[gen_stub_pyclass]
#[pyclass(name = "Piece", frozen, eq, ord)]
#[derive(PartialOrd, PartialEq, Eq, Copy, Clone, Hash)]
pub(crate) struct PyPiece {
    /// Get the piece type of the piece
    /// TODO
    #[pyo3(get)]
    pub(crate) piece_type: PyPieceType,
    /// Get the color of the piece
    /// TODO
    #[pyo3(get)]
    pub(crate) color: PyColor,
}

#[gen_stub_pymethods]
#[pymethods]
impl PyPiece {
    /// Create a new piece from a piece type and color
    #[new]
    #[inline]
    fn new(piece_type: PyPieceType, color_or_bool: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(color) = color_or_bool.extract::<PyColor>() {
            Ok(PyPiece { piece_type, color })
        } else if let Ok(boolean) = color_or_bool.extract::<bool>() {
            Ok(PyPiece {
                piece_type,
                color: PyColor(if boolean {
                    chess::Color::White
                } else {
                    chess::Color::Black
                }),
            })
        } else {
            Err(PyValueError::new_err("Color must be a color or bool."))
        }
    }

    /// Get the index of the piece (0-5)
    #[inline]
    fn get_index(&self) -> u8 {
        self.piece_type.get_index()
    }

    /// Convert the piece to a string.
    /// White is uppercase and black is lowercase.
    ///
    /// ```python
    /// >>> rust_chess.WHITE_PAWN.get_string()
    /// 'P'
    /// >>> rust_chess.BLACK_PAWN.get_string()
    /// 'p'
    /// ```
    #[inline]
    fn get_string(&self) -> String {
        self.piece_type.get_string(self.color)
    }

    /// Convert the piece to a string.
    /// White is uppercase and black is lowercase.
    ///
    /// ```python
    /// >>> print(rust_chess.WHITE_PAWN)
    /// P
    /// >>> print(rust_chess.BLACK_PAWN)
    /// p
    /// ```    
    #[inline]
    fn __str__(&self) -> String {
        self.get_string()
    }

    /// Convert the piece to a string.
    /// White is uppercase and black is lowercase.
    ///
    /// ```python
    /// >>> rust_chess.WHITE_PAWN
    /// P
    /// >>> rust_chess.BLACK_PAWN
    /// p
    /// ```
    #[inline]
    fn __repr__(&self) -> String {
        self.get_string()
    }

    /// Convert the piece to a unicode string.
    /// White is hollow and black is full.
    ///
    /// ```python
    /// >>> rust_chess.WHITE_PAWN.get_unicode()
    /// '♙'
    /// >>> rust_chess.BLACK_PAWN.get_unicode()
    /// '♟'
    /// ```
    #[inline]
    fn get_unicode(&self) -> &'static str {
        self.piece_type.get_unicode(self.color)
    }
}

use std::str::FromStr;

use pyo3::{basic::CompareOp, exceptions::PyValueError, prelude::*, types::PyAny};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::types::{
    bitboard::PyBitboard,
    color::{COLORS, PyColor},
};

/// Bitboard u64 consisting of dark colored squares (used for color calculation).
const DARK_SQUARES_BB: u64 = 0xAA55AA55AA55AA55;

/// Square class.
/// Represents a square on the chessboard.
/// The square is represented as an integer (0-63) or a string (e.g. "e4").
/// Supports comparison and equality.
///
/// rust-chess has constants for each square (e.g. A1, B2, etc.).
///
/// ```python
/// >>> square = rust_chess.Square(0)
/// >>> square
/// a1
/// >>> print(square)
/// a1
/// >>> square == rust_chess.Square("a1")
/// True
/// >>> square == rust_chess.A1
/// True
/// >>> square.get_index()
/// 0
/// >>> rust_chess.A4 == 24
/// True
/// >>> rust_chess.G4.get_rank()
/// 3
/// >>> rust_chess.G4.get_file()
/// 6
/// TODO
/// ```
#[gen_stub_pyclass]
#[pyclass(name = "Square", frozen)]
#[derive(PartialEq, Ord, Eq, PartialOrd, Copy, Clone, Default, Hash)]
pub(crate) struct PySquare(pub(crate) chess::Square);

#[gen_stub_pymethods]
#[pymethods]
impl PySquare {
    /// Creates a new square from an integer (0-63) or a string (e.g. "e4").
    ///
    /// ```python
    /// >>> rust_chess.Square(0)
    /// a1
    /// >>> rust_chess.Square("e4")
    /// e4
    /// ```
    #[new]
    #[inline]
    fn new(square_index_or_name: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(index) = square_index_or_name.extract::<u8>() {
            return PySquare::from_index(index);
        } else if let Ok(square_name) = square_index_or_name.extract::<&str>() {
            return PySquare::from_name(square_name);
        }
        Err(PyValueError::new_err(
            "Square must be an integer (0-63) or a string (e.g. \"e4\")",
        ))
    }

    /// Get the index of the square (0-63).
    /// Indexing starts at 0 (a1) and ends at 63 (h8).
    ///
    /// ```python
    /// >>> rust_chess.Square("e4").get_index()
    /// 28
    /// ```
    #[inline]
    fn get_index(&self) -> u8 {
        self.0.to_int()
    }

    /// Get the index of the square as an integer for indexing.
    ///
    /// ```python
    /// >>> int(rust_chess.Square("e4"))
    /// 28
    /// ```
    #[inline]
    fn __index__(&self) -> u8 {
        self.get_index()
    }

    /// Get the index of the square as an integer.
    ///
    /// ```python
    /// >>> arr = [1, 2, 3, 4, 5, 6]
    /// >>> arr[rust_chess.Square("a1")]
    /// 1
    /// ```
    #[inline]
    fn __int__(&self) -> u8 {
        self.get_index()
    }

    /// Hash the square based on its index.
    ///
    /// ```python
    /// >>> hash(rust_chess.E4)
    /// 28
    /// ```
    #[inline]
    fn __hash__(&self) -> u64 {
        self.get_index() as u64
    }

    /// Flips a square (eg. A1 -> A8).
    ///
    /// ```python
    /// >>> rust_chess.A1.flip()
    /// a8
    /// >>> rust_chess.H8.flip()
    /// h1
    /// ```
    #[inline]
    fn flip(&self) -> PySquare {
        PySquare(unsafe { chess::Square::new(self.get_index() ^ 56) })
    }

    /// Convert a square to a bitboard.
    ///
    /// ```python
    /// >>> bitboard = rust_chess.E4.to_bitboard()
    /// >>> bitboard.popcnt()
    /// 1
    /// >>> bitboard
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// ```
    #[inline]
    fn to_bitboard(&self) -> PyBitboard {
        PyBitboard::from_square(*self)
    }

    /// Create a new square from an index.
    /// Indexing starts at 0 (a1) and ends at 63 (h8).
    ///
    /// ```python
    /// >>> rust_chess.Square.from_index(0)
    /// a1
    /// ```
    #[staticmethod]
    #[inline]
    pub(crate) fn from_index(index: u8) -> PyResult<Self> {
        if index > 63 {
            return Err(PyValueError::new_err(
                "Square index must be between 0 and 63",
            ));
        }
        Ok(PySquare(unsafe { chess::Square::new(index) }))
    }

    /// Create a new square from rank and file.
    /// Rank and file are 0-indexed (0-7).
    ///
    /// ```python
    /// >>> rust_chess.Square.from_rank_file(0, 3)
    /// d1
    /// ```
    #[staticmethod]
    #[inline]
    fn from_rank_file(rank: u8, file: u8) -> PyResult<Self> {
        if rank > 7 || file > 7 {
            return Err(PyValueError::new_err(
                "Rank and file must be between 0 and 7",
            ));
        }
        Ok(PySquare(chess::Square::make_square(
            chess::Rank::from_index(rank as usize),
            chess::File::from_index(file as usize),
        )))
    }

    /// Create a new square from file and rank.
    /// File and rank are 0-indexed (0-7).
    ///
    /// ```python
    /// >>> rust_chess.Square.from_file_rank(3, 0)
    /// d1
    /// ```
    #[staticmethod]
    #[inline]
    fn from_file_rank(file: u8, rank: u8) -> PyResult<Self> {
        if rank > 7 || file > 7 {
            return Err(PyValueError::new_err(
                "Rank and file must be between 0 and 7",
            ));
        }
        Ok(PySquare(chess::Square::make_square(
            chess::Rank::from_index(rank as usize),
            chess::File::from_index(file as usize),
        )))
    }

    /// Get the name of the square (e.g. "e4").
    ///
    /// ```python
    /// >>> rust_chess.E4.get_name()
    /// 'e4'
    /// ```
    #[inline]
    fn get_name(&self) -> String {
        // Convert the square to a string using the chess crate
        self.0.to_string()
    }

    /// Get the name of the square (e.g. "e4"),
    ///
    /// ```python
    /// >>> print(rust_chess.E4)
    /// e4
    /// ```
    #[inline]
    fn __str__(&self) -> String {
        self.get_name()
    }

    /// Get the name of the square (e.g. "e4").
    ///
    /// ```python
    /// >>> rust_chess.E4
    /// e4
    /// ```
    #[inline]
    fn __repr__(&self) -> String {
        self.get_name()
    }

    /// Get the color of the square on the chessboard.
    ///
    /// ```python
    /// >>> rust_chess.A1.get_color() == rust_chess.BLACK
    /// True
    /// >>> rust_chess.E4.get_color() == rust_chess.WHITE
    /// True
    /// ```
    #[inline]
    pub(crate) fn get_color(&self) -> PyColor {
        let is_dark = ((DARK_SQUARES_BB >> self.get_index()) & 1) as usize; // 1 if true
        COLORS[is_dark] // [WHITE, BLACK]
    }

    /// Create a new square from a name (e.g. "e4").
    /// Not really needed since you can use the square constants.
    /// Could also just call the constructor with the name string.
    ///
    /// ```python
    /// >>> rust_chess.Square.from_name("d2")
    /// d2
    /// ```
    #[staticmethod]
    #[inline]
    fn from_name(square_name: &str) -> PyResult<Self> {
        // Parse the square using the chess crate
        let square_name = square_name.to_lowercase();
        chess::Square::from_str(&square_name)
            .map(PySquare)
            .map_err(|_| PyValueError::new_err("Invalid square"))
    }

    /// Compare the square to another square or integer.
    ///
    /// ```python
    /// >>> rust_chess.Square("d2") == rust_chess.D2
    /// True
    /// >>> rust_chess.Square("d2") == 11
    /// True
    /// >>> rust_chess.G6 > rust_chess.D3
    /// True
    /// >>> rust_chess.G6 <= 56
    /// True
    /// ```
    #[inline]
    fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyResult<bool> {
        // Convert self to index
        let self_index = self.get_index();

        // Convert other to index
        let other_index = if let Ok(other_square) = other.extract::<PySquare>() {
            other_square.get_index()
        } else if let Ok(other_index) = other.extract::<u8>() {
            other_index
        } else {
            return Err(PyValueError::new_err(
                "Square must be an integer (0-63) or a Square",
            ));
        };

        Ok(match op {
            CompareOp::Eq => self_index == other_index,
            CompareOp::Ne => self_index != other_index,
            CompareOp::Lt => self_index < other_index,
            CompareOp::Le => self_index <= other_index,
            CompareOp::Gt => self_index > other_index,
            CompareOp::Ge => self_index >= other_index,
        })
    }

    /// Get the rank of the square as an integer (0-7).
    ///
    /// ```python
    /// >>> rust_chess.E4.get_rank()
    /// 3
    /// ```
    #[inline]
    fn get_rank(&self) -> u8 {
        self.0.get_rank() as u8
    }

    /// Get the file of the square as an integer (0-7).
    ///
    /// ```python
    /// >>> rust_chess.E4.get_file()
    /// 4
    /// ```
    #[inline]
    fn get_file(&self) -> u8 {
        self.0.get_file() as u8
    }

    /// Returns the square above, otherwise None.
    ///
    /// ```python
    /// >>> rust_chess.H5.up()
    /// h6
    /// ```
    #[inline]
    fn up(&self) -> Option<Self> {
        self.0.up().map(PySquare)
    }

    /// Returns the square below, otherwise None.
    ///
    /// ```python
    /// >>> rust_chess.H5.down()
    /// h4
    /// ```
    #[inline]
    fn down(&self) -> Option<Self> {
        self.0.down().map(PySquare)
    }

    /// Returns the square to the left, otherwise None.
    ///
    /// ```python
    /// >>> rust_chess.H5.left()
    /// g5
    /// ```
    #[inline]
    fn left(&self) -> Option<Self> {
        self.0.left().map(PySquare)
    }

    /// Returns the square to the right, otherwise None
    ///
    /// ```python
    /// >>> rust_chess.H5.right()
    ///
    /// >>> rust_chess.H5.right() == None
    /// True
    /// ```
    #[inline]
    fn right(&self) -> Option<Self> {
        self.0.right().map(PySquare)
    }
}

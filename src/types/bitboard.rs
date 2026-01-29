use std::u64::MAX;

use chess::EMPTY;
use pyo3::{basic::CompareOp, exceptions::PyValueError, prelude::*, types::PyAny};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::types::square::PySquare;

pub(crate) const BB_EMPTY: PyBitboard = PyBitboard(EMPTY);
pub(crate) const BB_FULL: PyBitboard = PyBitboard(chess::BitBoard(MAX));

pub(crate) const BB_FILE_A: PyBitboard = PyBitboard(chess::BitBoard(0x0101_0101_0101_0101 << 0));
pub(crate) const BB_FILE_B: PyBitboard = PyBitboard(chess::BitBoard(0x0101_0101_0101_0101 << 1));
pub(crate) const BB_FILE_C: PyBitboard = PyBitboard(chess::BitBoard(0x0101_0101_0101_0101 << 2));
pub(crate) const BB_FILE_D: PyBitboard = PyBitboard(chess::BitBoard(0x0101_0101_0101_0101 << 3));
pub(crate) const BB_FILE_E: PyBitboard = PyBitboard(chess::BitBoard(0x0101_0101_0101_0101 << 4));
pub(crate) const BB_FILE_F: PyBitboard = PyBitboard(chess::BitBoard(0x0101_0101_0101_0101 << 5));
pub(crate) const BB_FILE_G: PyBitboard = PyBitboard(chess::BitBoard(0x0101_0101_0101_0101 << 6));
pub(crate) const BB_FILE_H: PyBitboard = PyBitboard(chess::BitBoard(0x0101_0101_0101_0101 << 7));

pub(crate) const BB_FILES: [PyBitboard; 8] = [
    BB_FILE_A, BB_FILE_B, BB_FILE_C, BB_FILE_D, BB_FILE_E, BB_FILE_F, BB_FILE_G, BB_FILE_H,
];

pub(crate) const BB_RANK_1: PyBitboard = PyBitboard(chess::BitBoard(0xff << (8 * 0)));
pub(crate) const BB_RANK_2: PyBitboard = PyBitboard(chess::BitBoard(0xff << (8 * 1)));
pub(crate) const BB_RANK_3: PyBitboard = PyBitboard(chess::BitBoard(0xff << (8 * 2)));
pub(crate) const BB_RANK_4: PyBitboard = PyBitboard(chess::BitBoard(0xff << (8 * 3)));
pub(crate) const BB_RANK_5: PyBitboard = PyBitboard(chess::BitBoard(0xff << (8 * 4)));
pub(crate) const BB_RANK_6: PyBitboard = PyBitboard(chess::BitBoard(0xff << (8 * 5)));
pub(crate) const BB_RANK_7: PyBitboard = PyBitboard(chess::BitBoard(0xff << (8 * 6)));
pub(crate) const BB_RANK_8: PyBitboard = PyBitboard(chess::BitBoard(0xff << (8 * 7)));

pub(crate) const BB_RANKS: [PyBitboard; 8] = [
    BB_RANK_1, BB_RANK_2, BB_RANK_3, BB_RANK_4, BB_RANK_5, BB_RANK_6, BB_RANK_7, BB_RANK_8,
];

/// Bitboard class.
/// Represents a 64-bit unsigned integer.
/// Each bit represents a square on the chessboard.
/// The least-significant bit represents a1, and the most-significant bit represents h8.
/// Supports bitwise operations and iteration.
/// Also supports comparison and equality.
///
#[gen_stub_pyclass]
#[pyclass(name = "Bitboard")]
#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Default, Hash)]
pub(crate) struct PyBitboard(pub(crate) chess::BitBoard);

impl PyBitboard {
    #[inline]
    fn extract_bitboard_or_u64(&self, other: &Bound<'_, PyAny>) -> PyResult<u64> {
        if let Ok(other_bitboard) = other.extract::<PyBitboard>() {
            Ok(other_bitboard.0.0)
        } else if let Ok(other_u64) = other.extract::<u64>() {
            Ok(other_u64)
        } else {
            Err(PyValueError::new_err(
                "Operand must be a Bitboard or an integer",
            ))
        }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyBitboard {
    /// Create a new Bitboard from a 64-bit integer or a square
    #[new]
    #[inline]
    fn new(bitboard_or_square: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(square) = bitboard_or_square.extract::<PySquare>() {
            Ok(PyBitboard::from_square(square))
        } else if let Ok(bitboard) = bitboard_or_square.extract::<u64>() {
            Ok(PyBitboard::from_uint(bitboard))
        } else {
            Err(PyValueError::new_err(
                "Bitboard must be a 64-bit integer or a square",
            ))
        }
    }

    /// Create a new Bitboard from a square.
    ///
    /// ```python
    /// >>> rust_chess.Bitboard.from_square(rust_chess.E4)
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// ```
    #[staticmethod]
    #[inline]
    pub(crate) fn from_square(square: PySquare) -> Self {
        PyBitboard(chess::BitBoard::from_square(square.0))
    }

    /// Create a new Bitboard from an unsigned 64-bit integer.
    ///
    /// ```python
    /// >>> rust_chess.Bitboard.from_uint(7)
    /// X X X . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// ```
    #[staticmethod]
    #[inline]
    fn from_uint(bitboard: u64) -> Self {
        PyBitboard(chess::BitBoard(bitboard))
    }

    /// Convert the Bitboard to a square.
    /// This grabs the least-significant square.
    ///
    /// ```python
    /// >>> bb = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb.to_square()
    /// e4
    /// >>> rust_chess.Bitboard(2351).to_square()
    /// a1
    /// ```
    #[inline]
    fn to_square(&self) -> PySquare {
        PySquare(self.0.to_square())
    }

    /// Convert the Bitboard to an unsigned 64-bit integer.
    ///
    /// ```python
    /// >>> bb = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb.to_uint()
    /// 268435456
    /// ```
    #[inline]
    fn to_uint(&self) -> u64 {
        self.0.0
    }

    /// Convert the Bitboard to an integer.
    ///
    /// ```python
    /// >>> bb = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> int(bb)
    /// 268435456
    /// ```
    #[inline]
    fn __int__(&self) -> u64 {
        self.to_uint()
    }

    /// Convert the Bitboard to a string.
    /// Displays the bitboard in an 8x8 grid.
    /// a1 is the top-left corner, h8 is the bottom-right corner.
    /// To make a1 the bottom-left corner and h8 the top-right corner, call `flip_vertical()` on the bitboard.
    /// Very useful for debugging purposes.
    ///
    /// ```python```
    /// >>> bb = rust_chess.Bitboard(16961066976411648)
    /// >>> for line in bb.get_string().split("\n"):
    /// ...     print(line)
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . X . . X . .
    /// . . X . . X . .
    /// . . . . . . . .
    /// . X . . . . X .
    /// . . X X X X . .
    /// . . . . . . . .
    #[inline]
    fn get_string(&self) -> String {
        self.0.to_string()
    }

    /// Convert the Bitboard to a string.
    /// Displays the bitboard in an 8x8 grid.
    /// a1 is the top-left corner, h8 is the bottom-right corner.
    /// To make a1 the bottom-left corner and h8 the top-right corner, call `flip_vertical()` on the bitboard.
    /// Very useful for debugging purposes.
    ///
    /// ```python```
    /// >>> bb = rust_chess.Bitboard(18643319766908928)
    /// >>> print(bb)
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . X . . X . .
    /// . . X . . X . .
    /// . . . . . . . .
    /// . . X X X X . .
    /// . X . . . . X .
    /// . . . . . . . .
    #[inline]
    fn __str__(&self) -> String {
        self.get_string()
    }

    /// Convert the Bitboard to a string.
    /// Displays the bitboard in an 8x8 grid.
    /// a1 is the top-left corner, h8 is the bottom-right corner.
    /// To make a1 the bottom-left corner and h8 the top-right corner, call `flip_vertical()` on the bitboard.
    /// Very useful for debugging purposes.
    ///
    /// ```python```
    /// >>> bb = rust_chess.Bitboard(35465847671881728)
    /// >>> bb
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . X . . X . .
    /// . . X . . X . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . X X X X X X .
    /// . . . . . . . .
    #[inline]
    fn __repr__(&self) -> String {
        self.get_string()
    }

    /// Count the number of squares in the Bitboard
    ///
    /// ```python
    /// >>> bb = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb.popcnt()
    /// 1
    /// >>> rust_chess.Bitboard(0).popcnt()
    /// 0
    /// >>> rust_chess.Board().get_all_bitboard().popcnt()
    /// 32
    /// ```
    #[inline]
    fn popcnt(&self) -> u32 {
        self.0.popcnt()
    }

    /// Flip a bitboard vertically.
    /// View it from the opponent's perspective.
    /// Useful for operations that rely on symmetry, like piece-square tables.
    ///
    /// ```python
    /// >>> bb = rust_chess.Bitboard(6781892917204992)
    /// >>> bb
    /// . . . . . . . .
    /// . . . X X . . .
    /// . . X X X X . .
    /// . X X X X X X .
    /// . . . X X . . .
    /// . . . X X . . .
    /// . . . X X . . .
    /// . . . . . . . .
    /// >>> bb.flip_vertical()
    /// . . . . . . . .
    /// . . . X X . . .
    /// . . . X X . . .
    /// . . . X X . . .
    /// . X X X X X X .
    /// . . X X X X . .
    /// . . . X X . . .
    /// . . . . . . . .
    /// ```
    #[inline]
    fn flip_vertical(&self) -> Self {
        PyBitboard(self.0.reverse_colors())
    }

    /// Return an iterator of the bitboard.
    ///
    /// ```python
    /// >>> bb = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> list(bb)
    /// [e4]
    /// >>> list(rust_chess.Bitboard(1025))
    /// [a1, c2]
    /// ```
    #[inline]
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    /// Get the next square in the Bitboard.
    /// Removes the square from the Bitboard.
    ///
    /// ```python
    /// >>> bb = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> next(bb)
    /// e4
    /// >>> next(bb)
    /// Traceback (most recent call last):
    /// Exception: message
    /// ```
    /// TODO: Next on bb with multiple squares
    #[inline]
    fn __next__(&mut self) -> Option<PySquare> {
        self.0.next().map(PySquare)
    }

    /// Rich comparison operations for the Bitboard type.
    ///
    /// ```python
    /// >>> bb_e4 = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb_e4_2 = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb_d4 = rust_chess.Bitboard.from_square(rust_chess.D4)
    /// >>> bb_a2 = rust_chess.Bitboard.from_square(rust_chess.A2)
    /// >>> bb_f4 = rust_chess.Bitboard.from_square(rust_chess.F4)
    /// # Equality (==)
    /// >>> bb_e4 == bb_e4_2
    /// True
    /// >>> bb_e4 == bb_e4.to_uint()
    /// True
    /// >>> bb_e4.to_uint() == bb_e4
    /// True
    /// # Inequality (!=)
    /// >>> bb_e4 != bb_d4
    /// True
    /// >>> bb_e4 != bb_e4
    /// False
    /// >>> bb_e4 != bb_e4.to_uint()
    /// False
    /// >>> bb_e4.to_uint() != bb_e4
    /// False
    /// # Less than (<)
    /// >>> bb_a2 < bb_f4
    /// True
    /// >>> bb_a2.to_uint() < bb_f4
    /// True
    /// >>> bb_a2 < bb_f4.to_uint()
    /// True
    /// # Less than or equal (<=)
    /// >>> bb_e4 <= bb_e4_2
    /// True
    /// >>> bb_e4.to_uint() <= bb_e4_2
    /// True
    /// >>> bb_e4 <= bb_e4_2.to_uint()
    /// True
    /// # Greater than (>)
    /// >>> bb_f4 > bb_a2
    /// True
    /// >>> bb_f4.to_uint() > bb_a2
    /// True
    /// >>> bb_f4 > bb_a2.to_uint()
    /// True
    /// # Greater than or equal (>=)
    /// >>> bb_e4 >= bb_e4_2
    /// True
    /// >>> bb_e4.to_uint() >= bb_e4_2
    /// True
    /// >>> bb_e4 >= bb_e4_2.to_uint()
    /// True
    /// ```
    #[inline]
    fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyResult<bool> {
        let self_value = self.0.0;

        let other_value = if let Ok(other_bitboard) = other.extract::<PyBitboard>() {
            other_bitboard.0.0
        } else if let Ok(other_u64) = other.extract::<u64>() {
            other_u64
        } else {
            return Err(PyValueError::new_err(
                "Bitboard must be an integer or a Bitboard",
            ));
        };

        Ok(match op {
            CompareOp::Eq => self_value == other_value,
            CompareOp::Ne => self_value != other_value,
            CompareOp::Lt => self_value < other_value,
            CompareOp::Le => self_value <= other_value,
            CompareOp::Gt => self_value > other_value,
            CompareOp::Ge => self_value >= other_value,
        })
    }

    // Bitwise operations
    
    // FIXME: Can't do "${op}=" with self

    /// Bitwise NOT operation (~self).
    ///
    /// ```python
    /// >>> bb = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> ~bb
    /// X X X X X X X X
    /// X X X X X X X X
    /// X X X X X X X X
    /// X X X X . X X X
    /// X X X X X X X X
    /// X X X X X X X X
    /// X X X X X X X X
    /// X X X X X X X X
    /// ```
    #[inline]
    fn __invert__(&self) -> Self {
        PyBitboard(!self.0)
    }

    /// Bitwise AND operation (self & other).
    ///
    /// ```python
    /// >>> bb1 = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb2 = rust_chess.Bitboard.from_square(rust_chess.B2)
    /// >>> bb1 & bb1
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// >>> (bb1 & bb1) == (bb1 & bb1.to_uint())
    /// True
    /// >>> (bb1 & bb2).popcnt() == 0
    /// True
    /// >>> (bb1 & bb2) == (bb1 & bb2.to_uint())
    /// True
    /// ```
    #[inline]
    fn __and__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let other_value = self.extract_bitboard_or_u64(other)?;
        Ok(PyBitboard::from_uint(self.0.0 & other_value))
    }

    /// Reflected bitwise AND operation (other & self).
    ///
    /// ```python
    /// >>> bb1 = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb2 = rust_chess.Bitboard.from_square(rust_chess.B2)
    /// >>> bb1 & bb1
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// >>> (bb1 & bb1) == (bb1.to_uint() & bb1)
    /// True
    /// >>> (bb2 & bb1).popcnt() == 0
    /// True
    /// >>> (bb2 & bb1) == (bb2.to_uint() & bb1)
    /// True
    /// ```
    #[inline]
    fn __rand__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        self.__and__(other)
    }

    /// In-place bitwise AND operation (self &= other).
    ///
    /// ```python
    /// >>> bb1 = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb2 = rust_chess.Bitboard.from_square(rust_chess.B2)
    /// >>> bb1 &= bb1
    /// >>> bb1
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// >>> bb1 &= bb1.to_uint()
    /// >>> bb1 == rust_chess.Bitboard.from_square(rust_chess.E4)
    /// True
    /// >>> bb1 &= bb2
    /// >>> bb1.popcnt() == 0
    /// True
    /// ```
    #[inline]
    fn __iand__(&mut self, other: &Bound<'_, PyAny>) -> PyResult<()> {
        let other_value = self.extract_bitboard_or_u64(other)?;
        self.0.0 &= other_value;
        Ok(())
    }

    /// Bitwise OR operation (self | other).
    ///
    /// ```python
    /// >>> bb1 = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb2 = rust_chess.Bitboard.from_square(rust_chess.D4)
    /// >>> bb1 | bb2
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . X X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// >>> (bb1 | bb2) == (bb1 | bb2.to_uint())
    /// True
    /// ```
    #[inline]
    fn __or__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let other_value = self.extract_bitboard_or_u64(other)?;
        Ok(PyBitboard::from_uint(self.0.0 | other_value))
    }

    /// Reflected bitwise OR operation (other | self).
    ///
    /// ```python
    /// >>> bb1 = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb2 = rust_chess.Bitboard.from_square(rust_chess.D4)
    /// >>> bb2 | bb1
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . X X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// >>> (bb2 | bb1) == (bb2.to_uint() | bb1)
    /// True
    /// ```
    #[inline]
    fn __ror__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        self.__or__(other)
    }

    /// In-place bitwise OR operation (self |= other).
    ///
    /// ```python
    /// >>> bb1 = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb2 = rust_chess.Bitboard.from_square(rust_chess.D4)
    /// >>> bb1 |= bb2
    /// >>> bb1
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . X X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// >>> bb1 |= bb2.to_uint()
    /// >>> bb1
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . X X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// ```
    #[inline]
    fn __ior__(&mut self, other: &Bound<'_, PyAny>) -> PyResult<()> {
        let other_value = self.extract_bitboard_or_u64(other)?;
        self.0.0 |= other_value;
        Ok(())
    }

    /// Bitwise XOR operation (self ^ other).
    ///
    /// ```python
    /// >>> bb1 = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb2 = rust_chess.Bitboard.from_square(rust_chess.B2)
    /// >>> bb1 ^ bb2
    /// . . . . . . . .
    /// . X . . . . . .
    /// . . . . . . . .
    /// . . . . X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// >>> (bb1 ^ bb2) == (bb1 ^ bb2.to_uint())
    /// True
    /// ```
    #[inline]
    fn __xor__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let other_value = self.extract_bitboard_or_u64(other)?;
        Ok(PyBitboard::from_uint(self.0.0 ^ other_value))
    }

    /// Reflected bitwise XOR operation (other ^ self).
    ///
    /// ```python
    /// >>> bb1 = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb2 = rust_chess.Bitboard.from_square(rust_chess.B2)
    /// >>> bb2 ^ bb1
    /// . . . . . . . .
    /// . X . . . . . .
    /// . . . . . . . .
    /// . . . . X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// >>> (bb2 ^ bb1) == (bb2.to_uint() ^ bb1)
    /// True
    /// ```
    #[inline]
    fn __rxor__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        self.__xor__(other)
    }

    /// In-place bitwise XOR operation (self ^= other).
    ///
    /// ```python
    /// >>> bb1 = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb2 = rust_chess.Bitboard.from_square(rust_chess.B2)
    /// >>> bb1 ^= bb2
    /// >>> bb1
    /// . . . . . . . .
    /// . X . . . . . .
    /// . . . . . . . .
    /// . . . . X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// >>> bb1 ^= bb2.to_uint()
    /// >>> bb1
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
    fn __ixor__(&mut self, other: &Bound<'_, PyAny>) -> PyResult<()> {
        let other_value = self.extract_bitboard_or_u64(other)?;
        self.0.0 ^= other_value;
        Ok(())
    }

    /// Left shift operation (self << shift).
    ///
    /// ```python
    /// >>> bb = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// >>> bb << 2
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . X .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// ```
    #[inline]
    fn __lshift__(&self, shift: u32) -> Self {
        PyBitboard::from_uint(self.0.0 << shift)
    }

    /// In-place left shift operation (self <<= shift).
    ///
    /// ```python
    /// >>> bb = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// >>> bb <<= 2
    /// >>> bb
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . X .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// ```
    #[inline]
    fn __ilshift__(&mut self, shift: u32) {
        self.0.0 <<= shift;
    }

    /// Right shift operation (self >> shift).
    ///
    /// ```python
    /// >>> bb = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// >>> bb >> 2
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . X . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// ```
    #[inline]
    fn __rshift__(&self, shift: u32) -> Self {
        PyBitboard::from_uint(self.0.0 >> shift)
    }

    /// In-place right shift operation (self >>= shift).
    ///
    /// ```python
    /// >>> bb = rust_chess.Bitboard.from_square(rust_chess.E4)
    /// >>> bb
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . X . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// >>> bb >>= 2
    /// >>> bb
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . X . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// ```
    #[inline]
    fn __irshift__(&mut self, shift: u32) {
        self.0.0 >>= shift;
    }
}

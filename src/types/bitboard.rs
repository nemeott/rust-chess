use pyo3::{exceptions::PyValueError, prelude::*, types::PyAny};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::types::square::PySquare;

/// Bitboard class.
/// Represents a 64-bit unsigned integer.
/// Each bit represents a square on the chessboard.
/// The least-significant bit represents a1, and the most-significant bit represents h8.
/// Supports bitwise operations and iteration.
/// Also supports comparison and equality.
///
#[gen_stub_pyclass]
#[pyclass(name = "Bitboard", eq, ord)]
#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Default, Hash)]
pub(crate) struct PyBitboard(pub(crate) chess::BitBoard);

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

    /// Create a new Bitboard from a square
    #[staticmethod]
    #[inline]
    pub(crate) fn from_square(square: PySquare) -> Self {
        PyBitboard(chess::BitBoard::from_square(square.0))
    }

    /// Create a new Bitboard from an unsigned 64-bit integer
    #[staticmethod]
    #[inline]
    fn from_uint(bitboard: u64) -> Self {
        PyBitboard(chess::BitBoard(bitboard))
    }

    /// Convert the Bitboard to a square.
    /// This grabs the least-significant square.
    ///
    #[inline]
    fn to_square(&self) -> PySquare {
        PySquare(self.0.to_square())
    }

    /// Convert the Bitboard to an unsigned 64-bit integer
    #[inline]
    fn to_uint(&self) -> u64 {
        self.0 .0
    }

    /// Convert the Bitboard to a string.
    /// Displays the bitboard in an 8x8 grid.
    /// a1 is the top-left corner, h8 is the bottom-right corner.
    /// To make a1 the bottom-left corner and h8 the top-right corner, call `flip_vertical()` on the bitboard.
    /// Very useful for debugging purposes.
    ///
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
    #[inline]
    fn __repr__(&self) -> String {
        self.get_string()
    }

    /// Count the number of squares in the Bitboard
    #[inline]
    fn popcnt(&self) -> u32 {
        self.0.popcnt()
    }

    /// Flip a bitboard vertically.
    /// View it from the opponent's perspective.
    /// Useful for operations that rely on symmetry, like piece-square tables.
    ///
    #[inline]
    fn flip_vertical(&self) -> Self {
        PyBitboard(self.0.reverse_colors())
    }

    /// Return an iterator of the bitboard
    #[inline]
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    /// Get the next square in the Bitboard.
    /// Removes the square from the Bitboard.
    ///
    #[inline]
    fn __next__(&mut self) -> Option<PySquare> {
        self.0.next().map(PySquare)
    }

    // Bitwise operations

    /// Bitwise NOT operation
    #[inline]
    fn __invert__(&self) -> Self {
        PyBitboard(!self.0)
    }

    /// Bitwise AND operation (self & other).
    #[inline]
    fn __and__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(other_bitboard) = other.extract::<PyBitboard>() {
            Ok(PyBitboard(self.0 & other_bitboard.0))
        } else if let Ok(other_u64) = other.extract::<u64>() {
            Ok(PyBitboard::from_uint(self.0 .0 & other_u64))
        } else {
            Err(PyValueError::new_err(
                "Operand must be a Bitboard or an integer",
            ))
        }
    }

    /// Reflected bitwise AND operation (other & self).
    #[inline]
    fn __rand__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        self.__and__(other)
    }

    /// In-place bitwise AND operation (self &= other).
    #[inline]
    fn __iand__(&mut self, other: &Bound<'_, PyAny>) -> PyResult<()> {
        if let Ok(other_bitboard) = other.extract::<PyBitboard>() {
            self.0 &= other_bitboard.0;
            Ok(())
        } else if let Ok(other_u64) = other.extract::<u64>() {
            self.0 .0 &= other_u64;
            Ok(())
        } else {
            Err(PyValueError::new_err(
                "Operand must be a Bitboard or an integer",
            ))
        }
    }

    /// Bitwise OR operation (self | other).
    #[inline]
    fn __or__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(other_bitboard) = other.extract::<PyBitboard>() {
            Ok(PyBitboard(self.0 | other_bitboard.0))
        } else if let Ok(other_u64) = other.extract::<u64>() {
            Ok(PyBitboard::from_uint(self.0 .0 | other_u64))
        } else {
            Err(PyValueError::new_err(
                "Operand must be a Bitboard or an integer",
            ))
        }
    }

    /// Reflected bitwise OR operation (other | self).
    #[inline]
    fn __ror__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        self.__or__(other)
    }

    /// In-place bitwise OR operation (self |= other).
    #[inline]
    fn __ior__(&mut self, other: &Bound<'_, PyAny>) -> PyResult<()> {
        if let Ok(other_bitboard) = other.extract::<PyBitboard>() {
            self.0 |= other_bitboard.0;
            Ok(())
        } else if let Ok(other_u64) = other.extract::<u64>() {
            self.0 .0 |= other_u64;
            Ok(())
        } else {
            Err(PyValueError::new_err(
                "Operand must be a Bitboard or an integer",
            ))
        }
    }

    /// Bitwise XOR operation (self ^ other).
    #[inline]
    fn __xor__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(other_bitboard) = other.extract::<PyBitboard>() {
            Ok(PyBitboard(self.0 ^ other_bitboard.0))
        } else if let Ok(other_u64) = other.extract::<u64>() {
            Ok(PyBitboard::from_uint(self.0 .0 ^ other_u64))
        } else {
            Err(PyValueError::new_err(
                "Operand must be a Bitboard or an integer",
            ))
        }
    }

    /// Reflected bitwise XOR operation (other ^ self).
    #[inline]
    fn __rxor__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        self.__xor__(other)
    }

    /// In-place bitwise XOR operation (self ^= other).
    #[inline]
    fn __ixor__(&mut self, other: &Bound<'_, PyAny>) -> PyResult<()> {
        if let Ok(other_bitboard) = other.extract::<PyBitboard>() {
            self.0 ^= other_bitboard.0;
            Ok(())
        } else if let Ok(other_u64) = other.extract::<u64>() {
            self.0 .0 ^= other_u64;
            Ok(())
        } else {
            Err(PyValueError::new_err(
                "Operand must be a Bitboard or an integer",
            ))
        }
    }

    /// Multiplication operation (self * other).
    #[inline]
    fn __mul__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(other_bitboard) = other.extract::<PyBitboard>() {
            Ok(PyBitboard(self.0 * other_bitboard.0))
        } else if let Ok(other_u64) = other.extract::<u64>() {
            Ok(PyBitboard::from_uint(self.0 .0 * other_u64))
        } else {
            Err(PyValueError::new_err(
                "Operand must be a Bitboard or an integer",
            ))
        }
    }

    /// Reflected multiplication operation (other * self).
    #[inline]
    fn __rmul__(&self, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        self.__mul__(other)
    }

    /// In-place multiplication operation (self *= other).
    #[inline]
    fn __imul__(&mut self, other: &Bound<'_, PyAny>) -> PyResult<()> {
        if let Ok(other_bitboard) = other.extract::<PyBitboard>() {
            self.0 = self.0 * other_bitboard.0;
            Ok(())
        } else if let Ok(other_u64) = other.extract::<u64>() {
            self.0 .0 *= other_u64;
            Ok(())
        } else {
            Err(PyValueError::new_err(
                "Operand must be a Bitboard or an integer",
            ))
        }
    }

    /// Left shift operation (self << shift).
    #[inline]
    fn __lshift__(&self, shift: u32) -> Self {
        PyBitboard::from_uint(self.0 .0 << shift)
    }

    /// Reflected left shift operation (not typically used)
    #[inline]
    fn __rlshift__(&self, _other: &Bound<'_, PyAny>) -> PyResult<Self> {
        Err(PyValueError::new_err(
            "Cannot perform shift with Bitboard on right",
        ))
    }

    /// In-place left shift operation (self <<= shift).
    #[inline]
    fn __ilshift__(&mut self, shift: u32) {
        self.0 .0 <<= shift;
    }

    /// Right shift operation (self >> shift).
    #[inline]
    fn __rshift__(&self, shift: u32) -> Self {
        PyBitboard::from_uint(self.0 .0 >> shift)
    }

    /// Reflected right shift operation (not typically used)
    #[inline]
    fn __rrshift__(&self, _other: &Bound<'_, PyAny>) -> PyResult<Self> {
        Err(PyValueError::new_err(
            "Cannot perform shift with Bitboard on right",
        ))
    }

    /// In-place right shift operation (self >>= shift).
    #[inline]
    fn __irshift__(&mut self, shift: u32) {
        self.0 .0 >>= shift;
    }
}

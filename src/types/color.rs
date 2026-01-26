use pyo3::{prelude::*, types::PyAny};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

// Color constants
pub(crate) const WHITE: PyColor = PyColor(chess::Color::White);
pub(crate) const BLACK: PyColor = PyColor(chess::Color::Black);
pub(crate) const COLORS: [PyColor; 2] = [WHITE, BLACK];

/// Color enum class.
/// White is True, Black is False.
///
/// ```python
/// >>> color = rust_chess.WHITE
///
/// >>> color
/// True
/// >>> print(color)
/// WHITE
/// >>> color == rust_chess.BLACK
/// False
/// >>> color == (not rust_chess.BLACK)
/// True
/// ```
#[gen_stub_pyclass]
#[pyclass(name = "Color", frozen)]
#[derive(PartialOrd, PartialEq, Eq, Copy, Clone, Hash)]
pub(crate) struct PyColor(pub(crate) chess::Color);

#[gen_stub_pymethods]
#[pymethods]
impl PyColor {
    /// Get the color as a string.
    ///
    /// ```python
    /// >>> rust_chess.WHITE.get_string()
    /// 'WHITE'
    /// >>> rust_chess.BLACK.get_string()
    /// 'BLACK'
    /// ```
    #[inline]
    fn get_string(&self) -> &str {
        if *self == WHITE {
            "WHITE"
        } else {
            "BLACK"
        }
    }

    /// Get the color as a string.
    ///
    /// ```python
    /// >>> print(rust_chess.WHITE)
    /// WHITE
    /// >>> print(rust_chess.BLACK)
    /// BLACK
    /// ```
    #[inline]
    fn __str__(&self) -> &str {
        self.get_string()
    }

    /// Get the color as a boolean.
    ///
    /// ```python
    /// >>> bool(rust_chess.WHITE)
    /// True
    /// >>> bool(rust_chess.BLACK)
    /// False
    /// ```
    #[inline]
    fn __bool__(&self) -> bool {
        *self == WHITE
    }
    
    #[inline]
    fn __hash__(&self) -> u64 {
        self.__bool__() as u64
    }

    /// Get the color as a boolean string.
    ///
    /// ```python
    /// >>> rust_chess.WHITE
    /// True
    /// >>> rust_chess.BLACK
    /// False
    /// ```
    #[inline]
    fn __repr__(&self) -> &str {
        if self.__bool__() {
            "True"
        } else {
            "False"
        }
    }

    /// Compare the color to another color or boolean.
    ///
    /// ```python
    /// >>> rust_chess.WHITE == rust_chess.BLACK
    /// False
    /// >>> rust_chess.WHITE == True
    /// True
    /// ```
    #[inline]
    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        if let Ok(other_color) = other.extract::<PyColor>() {
            self.__bool__() == other_color.__bool__()
        } else if let Ok(other_bool) = other.extract::<bool>() {
            self.__bool__() == other_bool
        } else {
            false
        }
    }
}

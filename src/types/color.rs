use pyo3::{basic::CompareOp, prelude::*, types::PyAny};
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

    /// Rich comparison operations for Color.
    ///
    /// Equality (==):
    /// ```python
    /// >>> rust_chess.WHITE == rust_chess.WHITE
    /// True
    /// >>> rust_chess.WHITE == True
    /// True
    /// >>> True == rust_chess.WHITE
    /// True
    /// ```
    ///
    /// Inequality (!=):
    /// ```python
    /// >>> rust_chess.WHITE != rust_chess.BLACK
    /// True
    /// >>> rust_chess.WHITE != False
    /// True
    /// >>> rust_chess.WHITE != True
    /// False
    /// >>> False != rust_chess.WHITE
    /// True
    /// >>> True != rust_chess.WHITE
    /// False
    /// ```
    #[inline]
    fn __richcmp__(&self, other: &Bound<'_, PyAny>, op: CompareOp) -> PyResult<bool> {
        let self_bool = self.__bool__();
        let other_bool = if let Ok(other_color) = other.extract::<PyColor>() {
            other_color.__bool__()
        } else if let Ok(other_bool) = other.extract::<bool>() {
            other_bool
        } else {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Color must be compared to a Color or bool",
            ));
        };

        match op {
            CompareOp::Eq => Ok(self_bool == other_bool),
            CompareOp::Ne => Ok(self_bool != other_bool),
            _ => Err(pyo3::exceptions::PyValueError::new_err(
                "Colors do not support ordering comparisons",
            )),
        }
    }
}

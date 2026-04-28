use std::str::FromStr;

use pyo3::{exceptions::PyValueError, prelude::*, types::PyAny};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::types::{color::WHITE, piece::PyPieceType, square::PySquare};

/// Move class.
/// Represents a chess move.
/// The move is represented as a source square, destination square, and optional promotion piece.
///
/// ```python
/// >>> move = rust_chess.Move(rust_chess.A4, rust_chess.B1)
/// >>> move
/// Move(a4, b1, None)
/// >>> print(move)
/// a4b1
/// >>> rust_chess.Move("a2a1q")
/// Move(a2, a1, Q)
/// >>> move == rust_chess.Move.from_uci("a4b1")
/// True
/// >>> move.source
/// a4
/// >>> move.dest
/// b1
/// >>> move.promotion
///
/// >>> move.promotion == None
/// True
/// ```
#[gen_stub_pyclass]
#[pyclass(name = "Move", frozen, eq, from_py_object)]
#[derive(Clone, Copy, Eq, PartialOrd, PartialEq, Default, Hash)]

// TODO: Keeping this as a wrapper for the chess crate for now for more performance in other functions.
// Small functions are slow however, so maybe also cache my class representations too?
pub(crate) struct PyMove(pub(crate) chess::ChessMove);

#[gen_stub_pymethods]
#[pymethods]
impl PyMove {
    /// Create a new move from a source, destination, and optional promotion piece or UCI string.
    ///
    /// ```python
    /// >>> rust_chess.Move(rust_chess.A2, rust_chess.A4)
    /// Move(a2, a4, None)
    /// >>> rust_chess.Move("g2g1q")
    /// Move(g2, g1, Q)
    /// ```
    #[new]
    #[pyo3(signature = (source_or_uci, dest = None, promotion = None))] // Default dest (enable UCI option) and promotion to None
    fn new(
        source_or_uci: &Bound<'_, PyAny>,
        dest: Option<PySquare>,
        promotion: Option<PyPieceType>,
    ) -> PyResult<Self> {
        // Expect source and destination squares
        if let Ok(source) = source_or_uci.extract::<PySquare>() {
            if let Some(dest) = dest {
                // Create a new move using the chess crate
                return Ok(PyMove(chess::ChessMove::new(
                    source.0,
                    dest.0,
                    promotion.map(|p| p.0),
                )));
            }
        }
        // Otherwise, try treating the first argument as a UCI string
        if let Ok(uci) = source_or_uci.extract::<&str>() {
            return PyMove::from_uci(uci);
        }
        // If we reach here, the input was invalid
        Err(PyValueError::new_err(
            "Move must be a UCI string or a source and destination square with optional promotion piece type",
        ))
    }

    /// Create a new move from a UCI string (e.g. "e2e4").
    ///
    /// ```python
    /// >>> rust_chess.Move.from_uci("e2e4")
    /// Move(e2, e4, None)
    /// ```
    #[staticmethod]
    #[inline]
    fn from_uci(uci: &str) -> PyResult<Self> {
        // Parse the move using the chess crate
        let uci = uci.to_lowercase();
        chess::ChessMove::from_str(&uci)
            .map(PyMove)
            .map_err(|_| PyValueError::new_err("Invalid UCI move"))
    }

    /// Get the UCI string representation of the move (e.g. "e2e4").
    ///
    /// ```python
    /// >>> move = rust_chess.Move(rust_chess.A2, rust_chess.A4)
    /// >>> move.get_uci()
    /// 'a2a4'
    /// ```
    #[inline]
    fn get_uci(&self) -> String {
        // Convert the move to a UCI string using the chess crate
        self.0.to_string()
    }

    /// Get the UCI string representation of the move (e.g. "e2e4").
    ///
    /// ```python
    /// >>> move = rust_chess.Move(rust_chess.A2, rust_chess.A4)
    /// >>> print(move)
    /// a2a4
    /// ```
    #[inline]
    fn __str__(&self) -> String {
        self.get_uci()
    }

    /// Get the internal representation of the move (e.g. "Move(e2, e4, None)").
    ///
    /// ```python
    /// >>> move = rust_chess.Move(rust_chess.E2, rust_chess.E4)
    /// >>> move
    /// Move(e2, e4, None)
    /// ```
    #[inline]
    fn __repr__(&self) -> String {
        format!(
            "Move({}, {}, {})",
            self.0.get_source(),
            self.0.get_dest(),
            self.get_promotion()
                .map(|p| p.get_string(WHITE))
                .unwrap_or_else(|| String::from("None"))
        )
    }

    /// Get the source square of the move.
    ///
    /// ```python
    /// >>> move = rust_chess.Move(rust_chess.A2, rust_chess.A4)
    /// >>> move.source
    /// a2
    /// ```
    #[getter]
    #[inline]
    pub(crate) fn get_source(&self) -> PySquare {
        PySquare(self.0.get_source())
    }

    /// Get the destination square of the move.
    ///
    ///
    /// ```python
    /// >>> move = rust_chess.Move(rust_chess.A2, rust_chess.A4)
    /// >>> move.dest
    /// a4
    /// ```
    #[getter]
    #[inline]
    pub(crate) fn get_dest(&self) -> PySquare {
        PySquare(self.0.get_dest())
    }

    /// Get the promotion piece of the move, otherwise None.
    ///
    /// ```python
    /// >>> move = rust_chess.Move(rust_chess.A2, rust_chess.A4)
    /// >>> move.promotion
    ///
    /// >>> move.promotion == None
    /// True
    /// >>> move = rust_chess.Move("g2g1q")
    /// >>> move.promotion
    /// Q
    /// ```
    #[getter]
    #[inline]
    fn get_promotion(&self) -> Option<PyPieceType> {
        self.0.get_promotion().map(PyPieceType)
    }
}

/// Move iterator class for generating legal moves.
/// Not intended for direct use.
/// Use the `Board` class methods for generating moves.
///
/// The generator stores all currently available legal moves in a buffer.
/// `retain_mask` narrows which destination squares are returned next.
/// `exclude_mask` permanently removes buffered moves whose destination squares match the mask.
///

// TODO: Use old lazy implementation if this is fixed: https://github.com/jordanbray/chess/issues/66
#[gen_stub_pyclass]
#[pyclass(name = "MoveGenerator")]
pub(crate) struct PyMoveGenerator {
    moves: Vec<chess::ChessMove>,
    allowed_mask: chess::BitBoard,
}

impl PyMoveGenerator {
    pub(crate) fn new(board: &chess::Board) -> Self {
        let mut moves: Vec<chess::ChessMove> = chess::MoveGen::new_legal(board).collect();
        moves.reverse(); // Reverse so we can pop from the end
        Self {
            moves,
            allowed_mask: !chess::EMPTY,
        }
    }

    pub(crate) fn remove_move(&mut self, move_to_remove: chess::ChessMove) {
        self.moves.retain(|&m| m != move_to_remove);
    }

    pub(crate) fn exclude_mask(&mut self, mask: chess::BitBoard) {
        self.moves
            .retain(|&m| (mask & chess::BitBoard::from_square(m.get_dest())).to_size(0) == 0);
    }

    pub(crate) fn retain_mask(&mut self, mask: chess::BitBoard) {
        self.allowed_mask = mask;
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyMoveGenerator {
    /// Return an iterator of the generator.
    ///
    /// The generator for a board saves state, regardless of how it is called.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> list(board.generate_legal_moves())
    /// [Move(a2, a3, None), Move(a2, a4, None), ..., Move(g1, h3, None)]
    /// >>> list(board.generate_legal_moves())
    /// []
    /// ```
    #[inline]
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    /// Get the next move in the generator.
    ///
    /// The generator for a board saves state, regardless of how it is called.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> moves = board.generate_legal_moves()
    /// >>> next(moves)
    /// Move(a2, a3, None)
    /// >>> next(board.generate_legal_moves())
    /// Move(a2, a4, None)
    /// ```
    #[inline]
    pub(crate) fn __next__(&mut self) -> Option<PyMove> {
        let mut idx_to_remove = None;
        for (i, m) in self.moves.iter().enumerate().rev() {
            if (self.allowed_mask & chess::BitBoard::from_square(m.get_dest())).to_size(0) != 0 {
                idx_to_remove = Some(i);
                break;
            }
        }
        if let Some(i) = idx_to_remove {
            Some(PyMove(self.moves.remove(i)))
        } else {
            None
        }
    }

    /// Get the length of the generator.
    ///
    /// Does not consume any iterations.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> moves = board.generate_legal_moves()
    /// >>> len(moves)
    /// 20
    /// >>> next(moves)
    /// Move(a2, a3, None)
    /// >>> len(moves)
    /// 19
    /// ```
    #[inline]
    pub(crate) fn __len__(&self) -> usize {
        self.moves
            .iter()
            .filter(|&m| {
                (self.allowed_mask & chess::BitBoard::from_square(m.get_dest())).to_size(0) != 0
            })
            .count()
    }

    /// Get the type of the move generator.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.generate_legal_moves()
    /// MoveGenerator()
    /// ```
    #[inline]
    fn __repr__(&self) -> &'static str {
        "MoveGenerator()"
    }
}

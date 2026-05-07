use std::fmt::Write;
use std::str::FromStr;
use std::sync::OnceLock;

use pyo3::{exceptions::PyValueError, prelude::*, types::PyList};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::types::board::{DEFAULT_BOARD, PyBoard, PyBoardStatus, PyCastleRights};
use crate::types::{
    bitboard::PyBitboard,
    board::PyRepetitionDetectionMode,
    color::PyColor,
    r#move::{PyMove, PyMoveGenerator},
    piece::{PyPiece, PyPieceType},
    square::PySquare,
};

/// Board batch class.
/// Represents a batch of chess boards.
/// Uses the same method names as `Board`, however they operate on a batch now.
///
// Uses SoA apprach to improve cache locality.
// TODO: Use hybrid approach? SoA only useful if iterating over one element; pack small into one struct?
#[gen_stub_pyclass]
#[pyclass(name = "BoardBatch")]
pub struct PyBoardBatch {
    boards: Vec<chess::Board>,

    // Lazily initialized per board, reset to None when a move is applied
    move_gens: Vec<OnceLock<Py<PyMoveGenerator>>>, // Use a Py to be able to share between Python and Rust

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

#[gen_stub_pymethods]
#[pymethods]
impl PyBoardBatch {
    // TODO: Reword docs to make more sense
    // TODO: Optimize
    // TODO: Length checks when passing in vectors

    /// Create a new batch of boards.
    ///
    /// ```python
    /// >>> rust_chess.BoardBatch(2)
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    /// ```
    #[new]
    #[pyo3(signature = (count, mode = PyRepetitionDetectionMode::Full))] // Default to full repetition detection
    #[must_use]
    #[inline]
    fn new(count: usize, mode: PyRepetitionDetectionMode) -> Self {
        let boards = vec![DEFAULT_BOARD.board; count];

        let board_histories = match mode {
            PyRepetitionDetectionMode::None => vec![None; count],
            PyRepetitionDetectionMode::Full => vec![DEFAULT_BOARD.board_history.clone(); count],
        };

        let move_gens = (0..count).map(|_| OnceLock::new()).collect();

        Self {
            boards,
            move_gens,
            halfmove_clocks: vec![0; count],
            fullmove_numbers: vec![1; count],
            repetition_detection_mode: mode,
            board_histories,
        }
    }

    /// Create a new batch of boards from a list of FEN strings.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch.from_fens([
    /// ...     "rnbqkbnr/ppp1pppp/8/3p4/2P1P3/8/PP1P1PPP/RNBQKBNR b KQkq - 0 2",
    /// ...     "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    /// ... ])
    /// >>> batch
    /// rnbqkbnr/ppp1pppp/8/3p4/2P1P3/8/PP1P1PPP/RNBQKBNR b KQkq - 0 2
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    /// ```
    #[staticmethod]
    #[pyo3(signature = (fens, mode = PyRepetitionDetectionMode::Full))] // Default full repetition detection
    fn from_fens(fens: Vec<String>, mode: PyRepetitionDetectionMode) -> PyResult<Self> {
        let count = fens.len();

        let mut boards = Vec::with_capacity(count);
        let mut move_gens = Vec::with_capacity(count);
        let mut halfmove_clocks = Vec::with_capacity(count);
        let mut fullmove_numbers = Vec::with_capacity(count);
        let mut board_histories = Vec::with_capacity(count);

        for (i, fen) in fens.into_iter().enumerate() {
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
            boards.push(
                chess::Board::from_str(&fen)
                    .map_err(|e| PyValueError::new_err(format!("Invalid FEN: {e}")))?,
            );

            board_histories.push(match mode {
                PyRepetitionDetectionMode::None => None,
                PyRepetitionDetectionMode::Full => {
                    let mut history = Vec::with_capacity(256);
                    history.push(boards[i].get_hash());
                    Some(history)
                }
            });

            move_gens.push(OnceLock::new());
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

            boards.push(b.board);
            move_gens.push(OnceLock::new());
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

    // TODO: to_boards

    /// Get the FEN string representation of each board on a newline.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> print(batch.get_fens())
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    /// ```
    #[must_use]
    #[inline]
    fn get_fens(&self) -> String {
        let mut fens = String::with_capacity(self.boards.len() * 56); // Default fen len is 56

        self.boards
            .iter()
            .zip(self.halfmove_clocks.iter())
            .zip(self.fullmove_numbers.iter())
            .for_each(|((board, halfmove_clock), fullmove_number)| {
                let base_fen = board.to_string();

                // 0: board, 1: player, 2: castling, 3: en passant, 4: halfmove clock, 5: fullmove number
                let base_parts: Vec<&str> = base_fen.split_whitespace().collect();

                // The chess crate doesn't handle the halfmove and fullmove values so we need to do it ourselves
                unsafe {
                    writeln!(
                        fens,
                        "{} {} {} {} {} {}",
                        base_parts[0],   // board
                        base_parts[1],   // player
                        base_parts[2],   // castling
                        base_parts[3],   // en passant
                        halfmove_clock,  // halfmove clock
                        fullmove_number, // fullmove number
                    )
                    .unwrap_unchecked();
                }
            });

        fens.trim_end().to_string() // Could ignore the extra line at the end for a little more speed (fast enough for now)
    }

    /// Get the FEN string representation of each board.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    /// ```
    #[inline]
    fn __repr__(&self) -> String {
        self.get_fens()
    }

    /// Print the string representation of each board separated by newlines.
    /// Labels are hidden by default.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.display()
    /// r n b q k b n r
    /// p p p p p p p p
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// P P P P P P P P
    /// R N B Q K B N R
    ///
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
    #[pyo3(signature = (show_labels = false))]
    #[inline]
    fn display(&self, show_labels: bool) {
        print!(
            "{}",
            self.boards
                .iter()
                .map(|board| PyBoard::_display(board, show_labels, false)) // 3rd parameter unused
                .collect::<String>()
        );
    }

    /// Get the string representation of each board.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> print(batch)
    /// r n b q k b n r
    /// p p p p p p p p
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// P P P P P P P P
    /// R N B Q K B N R
    /// <BLANKLINE>
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
        self.boards
            .iter()
            .map(|board| PyBoard::_display(board, false, false)) // 3rd parameter unused
            .collect()
    }

    /// Print the unicode string representation of each board separated by newlines.
    /// Labels are hidden by default.
    ///
    /// The dark mode parameter is enabled by default.
    /// This inverts the color of the piece, which looks correct on a dark background.
    /// Unicode assumes black text on white background, where in most terminals, it is the opposite.
    /// Disable if you are a psychopath and use light mode in your terminal/IDE.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.display_unicode() # This looks fine printed to terminal
    /// ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖
    /// ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙
    /// · · · · · · · ·
    /// · · · · · · · ·
    /// · · · · · · · ·
    /// · · · · · · · ·
    /// ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
    /// ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    ///
    /// ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖
    /// ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙
    /// · · · · · · · ·
    /// · · · · · · · ·
    /// · · · · · · · ·
    /// · · · · · · · ·
    /// ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
    /// ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    ///
    /// ```
    #[pyo3(signature = (show_labels = false, dark_mode = true))]
    #[inline]
    fn display_unicode(&self, show_labels: bool, dark_mode: bool) {
        print!(
            "{}",
            self.boards
                .iter()
                .map(|board| PyBoard::_display_unicode(board, show_labels, dark_mode)) // 3rd parameter unused
                .collect::<String>()
        );
    }

    /// Print the unicode string representation of each board with ANSI color codes.
    /// The boards are a bit tiny, but it looks pretty good.
    /// Labels are hidden by default.
    ///
    /// The default board color is tan/brown.
    /// Enable the `green_mode` parameter to change the color to olive/sand.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.display_color(show_labels=True, green_mode=True)
    /// 8 ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    /// 7 ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
    /// 6
    /// 5
    /// 4
    /// 3
    /// 2 ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
    /// 1 ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    ///   a b c d e f g h
    ///
    /// 8 ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    /// 7 ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
    /// 6
    /// 5
    /// 4
    /// 3
    /// 2 ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
    /// 1 ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    ///   a b c d e f g h
    ///
    /// ```
    #[pyo3(signature = (show_labels = false, green_mode = false))]
    #[inline]
    fn display_color(&self, show_labels: bool, green_mode: bool) {
        print!(
            "{}",
            self.boards
                .iter()
                .map(|board| PyBoard::_display_color(board, show_labels, green_mode)) // 3rd parameter unused
                .collect::<String>()
        );
    }

    /// Print the string representation of each board separated by newlines.
    /// Detects the terminal's width and tiles the boards accordingly.
    /// Labels are hidden by default.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.display_tiled()
    /// r n b q k b n r  r n b q k b n r
    /// p p p p p p p p  p p p p p p p p
    /// . . . . . . . .  . . . . . . . .
    /// . . . . . . . .  . . . . . . . .
    /// . . . . . . . .  . . . . . . . .
    /// . . . . . . . .  . . . . . . . .
    /// P P P P P P P P  P P P P P P P P
    /// R N B Q K B N R  R N B Q K B N R
    ///
    /// ```
    #[pyo3(signature = (show_labels = false))]
    #[inline]
    fn display_tiled(&self, show_labels: bool) {
        // TODO: Make constant for 136 default board size
        PyBoard::_display_tiled(PyBoard::_display, 136, &self.boards, show_labels, false); // 3rd parameter unused
    }

    /// Print the unicode string representation of each board separated by newlines.
    /// Detects the terminal's width and tiles the boards accordingly.
    /// Labels are hidden by default.
    ///
    /// The dark mode parameter is enabled by default.
    /// This inverts the color of the piece, which looks correct on a dark background.
    /// Unicode assumes black text on white background, where in most terminals, it is the opposite.
    /// Disable if you are a psychopath and use light mode in your terminal/IDE.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.display_unicode_tiled() # Looks fine printed to terminal
    /// ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖  ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖
    /// ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙  ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙
    /// · · · · · · · ·  · · · · · · · ·
    /// · · · · · · · ·  · · · · · · · ·
    /// · · · · · · · ·  · · · · · · · ·
    /// · · · · · · · ·  · · · · · · · ·
    /// ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟  ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
    /// ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜  ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    ///
    /// ```
    #[pyo3(signature = (show_labels = false, dark_mode = true))]
    #[inline]
    fn display_unicode_tiled(&self, show_labels: bool, dark_mode: bool) {
        PyBoard::_display_tiled(
            PyBoard::_display_unicode,
            232,
            &self.boards,
            show_labels,
            dark_mode,
        );
    }

    /// Print the unicode string representation of each board with ANSI color codes.
    /// Detects the terminal's width and tiles the boards accordingly.
    /// The boards are a bit tiny, but it looks pretty good.
    /// Labels are hidden by default.
    ///
    /// The default board color is tan/brown.
    /// Enable the `green_mode` parameter to change the color to olive/sand.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.display_color_tiled()
    /// ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜  ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    /// ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟  ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
    ///
    ///
    ///
    ///
    /// ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟  ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
    /// ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜  ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    ///
    /// ```
    #[pyo3(signature = (show_labels = false, green_mode = false))]
    #[inline]
    fn display_color_tiled(&self, show_labels: bool, green_mode: bool) {
        PyBoard::_display_tiled(
            PyBoard::_display_color,
            2666, // Isn't exactly correct but it's pretty close
            &self.boards,
            show_labels,
            green_mode,
        );
    }

    /// Create new moves from SAN strings (e.g. "e4") for each board.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.get_move_from_san(["e4", "d4"])
    /// [Move(e2, e4, None), Move(d2, d4, None)]
    /// ```
    #[allow(clippy::needless_pass_by_value)] // Can't pass reference of string to Python
    #[inline]
    fn get_move_from_san(&self, sans: Vec<String>) -> PyResult<Vec<PyMove>> {
        self.boards
            .iter()
            .zip(sans.iter())
            .map(|(board, san)| PyBoard::_get_move_from_san(board, san))
            .collect()
    }

    // TODO: get_san_from_move

    /// Get the number of boards in the batch.
    ///
    /// ```python
    /// >>> len(rust_chess.BoardBatch(2))
    /// 2
    /// >>> len(rust_chess.BoardBatch(312))
    /// 312
    /// ```
    #[inline]
    const fn __len__(&self) -> usize {
        self.boards.len()
    }

    // Get the Zobrist hash of each board.
    //
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.zobrist_hashes
    /// [9023329949471135578, 9023329949471135578]
    /// ```
    #[getter]
    #[inline]
    fn get_zobrist_hashes(&self) -> Vec<u64> {
        self.boards.iter().map(PyBoard::_get_zobrist_hash).collect()
    }

    /// Get a hash of the board batch based on the sum of the Zobrist hashes.
    /// Will likely overflow which is fine since this is a fast hash.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> hash(batch)
    /// -400084174767280460
    /// ```
    #[inline]
    fn __hash__(&self) -> u64 {
        self.boards.iter().map(PyBoard::_get_zobrist_hash).sum()
    }

    /// Check if two board batches are equal based on the Zobrist hashes of their boards.
    ///
    /// ```python
    /// >>> batch1 = rust_chess.BoardBatch(2)
    /// >>> batch2 = rust_chess.BoardBatch(2)
    /// >>> batch1 == batch2
    /// True
    /// >>> batch1.make_move([rust_chess.Move("e2e4"), rust_chess.Move("d2d4")])
    /// >>> batch1 == batch2
    /// False
    /// ```
    #[inline]
    fn __eq__(&self, other: &Self) -> bool {
        self.boards
            .iter()
            .zip(other.boards.iter())
            .all(|(b1, b2)| PyBoard::_get_zobrist_hash(b1) == PyBoard::_get_zobrist_hash(b2))
    }

    /// Check if two board batches are not equal based on the Zobrist hashes of their boards.
    ///
    /// ```python
    /// >>> batch1 = rust_chess.BoardBatch(2)
    /// >>> batch2 = rust_chess.BoardBatch(2)
    /// >>> batch1 != batch2
    /// False
    /// >>> batch1.make_move([rust_chess.Move("e2e4"), rust_chess.Move("d2d4")])
    /// >>> batch1 != batch2
    /// True
    /// ```
    #[inline]
    fn __ne__(&self, other: &Self) -> bool {
        !self.__eq__(other)
    }

    /// Compare two board batches based on the Zobrist hashes of their boards.
    /// Returns a list of booleans where `True` indicates the respective boards match.
    ///
    /// ```python
    /// >>> batch1 = rust_chess.BoardBatch(2)
    /// >>> batch2 = rust_chess.BoardBatch(2)
    /// >>> batch1.compare(batch2)
    /// [True, True]
    /// >>> batch1.make_move([rust_chess.Move("e2e4"), rust_chess.Move("d2d4")])
    /// >>> batch2.make_move([rust_chess.Move("e2e4"), rust_chess.Move("a2a3")])
    /// >>> batch1.compare(batch2)
    /// [True, False]
    /// ```
    #[inline]
    fn compare(&self, other: &Self) -> Vec<bool> {
        self.boards
            .iter()
            .zip(other.boards.iter())
            .map(|(b1, b2)| PyBoard::_get_zobrist_hash(b1) == PyBoard::_get_zobrist_hash(b2))
            .collect()
    }

    /// Get the current player to move for each board.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.turn
    /// [True, True]
    /// >>> batch.make_move([rust_chess.Move("e2e4"), rust_chess.Move("d2d4")])
    /// >>> batch.turn
    /// [False, False]
    /// ```
    #[getter]
    #[inline]
    fn get_turn(&self) -> Vec<PyColor> {
        self.boards
            .iter()
            .map(|board| PyColor(PyBoard::_get_turn(board)))
            .collect()
    }

    // TODO: List of colors for below functions?

    /// Get the king square of each board for a color.
    ///
    /// ```python
    /// >>> rust_chess.BoardBatch(2).get_king_square(rust_chess.WHITE)
    /// [e1, e1]
    /// >>> rust_chess.BoardBatch(2).get_king_square(rust_chess.BLACK)
    /// [e8, e8]
    /// ```
    #[inline]
    fn get_king_square(&self, color: PyColor) -> Vec<PySquare> {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_king_square(board, color))
            .collect()
    }

    /// Get the castle rights of each board for a color.
    /// Returns a list `CastleRights` enum types, which has the values: `NO_RIGHTS`, `KING_SIDE`, `QUEEN_SIDE`, `BOTH`.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.get_castle_rights(rust_chess.WHITE)
    /// [CastleRights.BOTH, CastleRights.BOTH]
    /// >>> batch.get_castle_rights(rust_chess.BLACK)
    /// [CastleRights.BOTH, CastleRights.BOTH]
    /// ```
    #[inline]
    fn get_castle_rights(&self, color: PyColor) -> Vec<PyCastleRights> {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_castle_rights(board, color.0))
            .collect()
    }

    /// Get the castle rights of the current player to move for each board.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.get_my_castle_rights()
    /// [CastleRights.BOTH, CastleRights.BOTH]
    /// ```
    #[inline]
    fn get_my_castle_rights(&self) -> Vec<PyCastleRights> {
        self.boards
            .iter()
            .map(PyBoard::_get_my_castle_rights)
            .collect()
    }

    /// Get the castle rights of the opponent for each board.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.get_their_castle_rights()
    /// [CastleRights.BOTH, CastleRights.BOTH]
    /// ```
    #[inline]
    fn get_their_castle_rights(&self) -> Vec<PyCastleRights> {
        self.boards
            .iter()
            .map(PyBoard::_get_their_castle_rights)
            .collect()
    }

    /// Check if a color can castle (either side) for each board.
    /// Returns a list of booleans.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.can_castle(rust_chess.WHITE)
    /// [True, True]
    /// >>> batch.can_castle(rust_chess.BLACK)
    /// [True, True]
    /// ```
    #[inline]
    fn can_castle(&self, color: PyColor) -> Vec<bool> {
        self.boards
            .iter()
            .map(|board| PyBoard::_can_castle(board, color))
            .collect()
    }

    /// Check if a color can castle queenside for each board.
    /// Returns a list of booleans.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.can_castle_queenside(rust_chess.WHITE)
    /// [True, True]
    /// >>> batch.can_castle_queenside(rust_chess.BLACK)
    /// [True, True]
    /// ```
    #[inline]
    fn can_castle_queenside(&self, color: PyColor) -> Vec<bool> {
        self.boards
            .iter()
            .map(|board| PyBoard::_can_castle_queenside(board, color))
            .collect()
    }

    /// Check if a color can castle kingside for each board.
    /// Returns a list of booleans.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.can_castle_kingside(rust_chess.WHITE)
    /// [True, True]
    /// >>> batch.can_castle_kingside(rust_chess.BLACK)
    /// [True, True]
    /// ```
    #[inline]
    fn can_castle_kingside(&self, color: PyColor) -> Vec<bool> {
        self.boards
            .iter()
            .map(|board| PyBoard::_can_castle_kingside(board, color))
            .collect()
    }

    // TODO: Reword

    /// Check if the respective move is castling for each board.
    /// Assumes the moves are pseudo-legal.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.is_castling([rust_chess.Move("e2e4"), rust_chess.Move("d2d4")])
    /// [False, False]
    /// ```
    #[allow(clippy::needless_pass_by_value)] // Can't pass reference of PyMove to Python
    #[inline]
    fn is_castling(&self, chess_moves: Vec<PyMove>) -> Vec<bool> {
        chess_moves
            .iter()
            .zip(self.boards.iter())
            .map(|(chess_move, board)| PyBoard::_is_castling(board, *chess_move))
            .collect()
    }

    /// Check if the respective move is queenside castling for each board.
    /// Assumes the move is pseudo-legal.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.is_castling_queenside([rust_chess.Move("e2e4"), rust_chess.Move("d2d4")])
    /// [False, False]
    /// ```
    #[allow(clippy::needless_pass_by_value)] // Can't pass reference of PyMove to Python
    #[inline]
    fn is_castling_queenside(&self, chess_moves: Vec<PyMove>) -> Vec<bool> {
        chess_moves
            .iter()
            .zip(self.boards.iter())
            .map(|(chess_move, board)| PyBoard::_is_castling_queenside(board, *chess_move))
            .collect()
    }

    /// Check if the respective move is kingside castling for each board.
    /// Assumes the move is pseudo-legal.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.is_castling_kingside([rust_chess.Move("e2e4"), rust_chess.Move("d2d4")])
    /// [False, False]
    /// ```
    #[allow(clippy::needless_pass_by_value)] // Can't pass reference of PyMove to Python
    #[inline]
    fn is_castling_kingside(&self, chess_moves: Vec<PyMove>) -> Vec<bool> {
        chess_moves
            .iter()
            .zip(self.boards.iter())
            .map(|(chess_move, board)| PyBoard::_is_castling_kingside(board, *chess_move))
            .collect()
    }

    /// Get the color of the piece on a respective square for each board, otherwise None.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.get_color_on([rust_chess.Square("e1"), rust_chess.Square("e8")])
    /// [True, False]
    /// ```
    #[allow(clippy::needless_pass_by_value)] // Can't pass reference of PySquare to Python
    #[inline]
    fn get_color_on(&self, squares: Vec<PySquare>) -> Vec<Option<PyColor>> {
        self.boards
            .iter()
            .zip(squares.iter())
            .map(|(board, square)| PyBoard::_get_color_on(board, *square))
            .collect()
    }

    /// Get the piece type on a respective square for each board, otherwise None.
    /// Different than `get_piece_on` because it returns the piece type, which does not include color.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.get_piece_type_on([rust_chess.Square("e1"), rust_chess.Square("e7")])
    /// [K, P]
    /// ```
    #[allow(clippy::needless_pass_by_value)] // Can't pass reference of PySquare to Python
    #[inline]
    fn get_piece_type_on(&self, squares: Vec<PySquare>) -> Vec<Option<PyPieceType>> {
        self.boards
            .iter()
            .zip(squares.iter())
            .map(|(board, square)| PyBoard::_get_piece_type_on(board, *square))
            .collect()
    }

    /// Get the piece on a respective square, otherwise None.
    /// Different than `get_piece_on` because it returns the piece, which includes color.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.get_piece_on([rust_chess.Square("e1"), rust_chess.Square("e7")])
    /// [K, p]
    /// ```
    #[allow(clippy::needless_pass_by_value)] // Can't pass reference of PySquare to Python
    #[inline]
    fn get_piece_on(&self, squares: Vec<PySquare>) -> Vec<Option<PyPiece>> {
        squares
            .iter()
            .zip(self.boards.iter())
            .map(|(square, board)| PyBoard::_get_piece_on(board, *square))
            .collect()
    }

    /// Get the en passant square of each board, otherwise None.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.en_passant
    /// [None, None]
    /// ```
    #[getter]
    #[inline]
    fn get_en_passant(&self) -> Vec<Option<PySquare>> {
        self.boards.iter().map(PyBoard::_get_en_passant).collect()
    }

    /// Check if a respective move is en passant for each board.
    ///
    /// Assumes the moves are legal.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.is_en_passant([rust_chess.Move("e2e4"), rust_chess.Move("d2d4")])
    /// [False, False]
    /// ```
    #[allow(clippy::needless_pass_by_value)] // Can't pass reference of PyMove to Python
    #[inline]
    fn is_en_passant(&self, chess_moves: Vec<PyMove>) -> Vec<bool> {
        self.boards
            .iter()
            .zip(chess_moves.iter())
            .map(|(board, chess_move)| PyBoard::_is_en_passant(board, *chess_move))
            .collect()
    }

    /// Check if a respective move is a capture for each board.
    ///
    /// Assumes the moves are legal.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.is_capture([rust_chess.Move("e2e4"), rust_chess.Move("d2d4")])
    /// [False, False]
    /// ```
    #[allow(clippy::needless_pass_by_value)] // Can't pass reference of PyMove to Python
    #[inline]
    fn is_capture(&self, chess_moves: Vec<PyMove>) -> Vec<bool> {
        self.boards
            .iter()
            .zip(chess_moves.iter())
            .map(|(board, chess_move)| PyBoard::_is_capture(board, *chess_move))
            .collect()
    }

    /// Check if a respective move is a capture or a pawn move for each board.
    /// This type of move "zeros" the halfmove clock (sets it to 0).
    ///
    /// Assumes the moves are legal.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.is_zeroing([rust_chess.Move("e2e4"), rust_chess.Move("d2d4")])
    /// [True, True]
    /// ```
    #[allow(clippy::needless_pass_by_value)] // Can't pass reference of PyMove to Python
    #[inline]
    fn is_zeroing(&self, chess_moves: Vec<PyMove>) -> Vec<bool> {
        self.boards
            .iter()
            .zip(chess_moves.iter())
            .map(|(board, chess_move)| PyBoard::_is_zeroing(board, *chess_move))
            .collect()
    }

    /// Check if the move is legal (supposedly very slow according to the chess crate).
    /// Use this function for moves not generated by the move generator.
    /// `is_legal_quick` is faster for moves generated by the move generator.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> move_list = [rust_chess.Move("e2e4"), rust_chess.Move("e2e5")]
    /// >>> batch.is_legal_move(move_list)
    /// [True, False]
    /// ```
    #[allow(clippy::needless_pass_by_value)] // Can't pass reference of PyMove to Python
    #[inline]
    fn is_legal_move(&self, chess_moves: Vec<PyMove>) -> Vec<bool> {
        self.boards
            .iter()
            .zip(chess_moves.iter())
            .map(|(board, chess_move)| PyBoard::_is_legal_move(board, *chess_move))
            .collect()
    }

    /// Check if the move generated by the generator is legal.
    /// Only use this function for moves generated by the move generator.
    /// You would want to use this when you have a psuedo-legal move (guarenteed by the generator).
    /// Slightly faster than using `is_legal_move` since it doesn't have to check as much stuff.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.is_legal_generator_move([rust_chess.Move("e2e4"), rust_chess.Move("d2d4")])
    /// [True, True]
    /// ```
    // FIXME: Use generator moves for docs
    #[allow(clippy::needless_pass_by_value)] // Can't pass reference of PyMove to Python
    #[inline]
    fn is_legal_generator_move(&self, chess_moves: Vec<PyMove>) -> Vec<bool> {
        self.boards
            .iter()
            .zip(chess_moves.iter())
            .map(|(board, chess_move)| PyBoard::_is_legal_generator_move(board, *chess_move))
            .collect()
    }

    // TODO: make_null_move (would require move history to undo (probably?))

    /// Make a null move for every board onto a new board batch.
    /// Returns None if the current player is in check.
    ///
    #[inline]
    fn make_null_move_new(&self) -> Option<Self> {
        let count = self.boards.len();

        let mut new_boards = Vec::with_capacity(count);
        let mut move_gens = Vec::with_capacity(count);
        let mut halfmove_clocks = Vec::with_capacity(count);
        let mut fullmove_numbers = Vec::with_capacity(count);

        for i in 0..count {
            // Get the new board using the chess crate
            new_boards.push(self.boards[i].null_move()?);

            // Create a new uninitialized move generator using the chess crate
            move_gens.push(OnceLock::new());

            // // Increment the halfmove clock
            halfmove_clocks.push(self.halfmove_clocks[i] + 1); // Null moves aren't zeroing, so we can just add 1 here

            // // Increment fullmove number if black moves
            #[allow(clippy::cast_possible_truncation)]
            fullmove_numbers
                .push(self.fullmove_numbers[i] + (self.boards[i].side_to_move().to_index() as u8)); // White is 0, black is 1
        }

        Some(Self {
            boards: new_boards,
            move_gens,
            halfmove_clocks,
            fullmove_numbers,
            repetition_detection_mode: self.repetition_detection_mode,

            // Don't update move history when making a null move
            board_histories: self.board_histories.clone(),
        })
    }

    /// Make a respective move onto each board.
    ///
    /// Defaults to checking move legality, unless the optional legality parameter is `False`.
    /// Not checking move legality will provide a slight performance boost, but crash if the move is invalid.
    /// Checking legality will return an error if the move is illegal.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> batch.make_move([rust_chess.Move("e2e4"), rust_chess.Move("d2d4")])
    /// >>> print(batch)
    /// r n b q k b n r
    /// p p p p p p p p
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . P . . .
    /// . . . . . . . .
    /// P P P P . P P P
    /// R N B Q K B N R
    /// <BLANKLINE>
    /// r n b q k b n r
    /// p p p p p p p p
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . P . . . .
    /// . . . . . . . .
    /// P P P . P P P P
    /// R N B Q K B N R
    ///
    /// ```
    // TODO: is_generator_move
    // TODO: Optimize
    #[pyo3(signature = (chess_moves, check_legality = true))]
    #[allow(clippy::needless_pass_by_value)] // Can't pass reference of PyMove to Python
    #[inline]
    fn make_move(&mut self, chess_moves: Vec<PyMove>, check_legality: bool) -> PyResult<()> {
        let count = self.boards.len();

        for i in 0..count {
            // Check if draw by fivefold
            if PyBoard::_is_n_repetition(
                self.board_histories[i].as_ref(),
                self.halfmove_clocks[i],
                5,
            ) {
                return Err(PyValueError::new_err(
                    "Game over due to fivefold repetition",
                ));
            }

            // If we are checking legality, check if the move is legal
            if check_legality && !PyBoard::_is_legal_move(&self.boards[i], chess_moves[i]) {
                return Err(PyValueError::new_err("Illegal move"));
            }

            // Make the move onto a new board using the chess crate
            let temp_board: chess::Board = self.boards[i].make_move_new(chess_moves[i].0);

            // Reset the halfmove clock if the move zeroes (is a capture or pawn move and therefore "zeroes" the halfmove clock)
            let is_zeroing = if PyBoard::_is_zeroing(&self.boards[i], chess_moves[i]) {
                self.halfmove_clocks[i] = 0;
                true
            } else {
                self.halfmove_clocks[i] += 1; // Add one if not zeroing
                false
            };

            // Increment fullmove number if black moves
            self.fullmove_numbers[i] += self.boards[i].side_to_move().to_index() as u8; // White is 0, black is 1

            // Add the new board's Zobrist hash to history
            if let Some(history) = &mut self.board_histories[i] {
                if is_zeroing {
                    // Don't need previous history anymore since it is a zeroing move (irreversible)
                    history.clear();
                }
                history.push(temp_board.get_hash());
            }

            // Invalidate the move generator since the board has changed
            self.move_gens[i].take();

            // Update the current board
            self.boards[i] = temp_board;
        }

        Ok(())
    }

    /// Make a respective move onto a new board for each board.
    ///
    /// Defaults to checking move legality, unless the optional legality parameter is `False`.
    /// Not checking move legality will provide a slight performance boost, but crash if the move is invalid.
    /// Checking legality will return an error if the move is illegal.
    ///
    /// ```python
    /// >>> batch = rust_chess.BoardBatch(2)
    /// >>> new_batch = batch.make_move_new([rust_chess.Move("e2e4"), rust_chess.Move("d2d4")])
    /// >>> print(new_batch)
    /// r n b q k b n r
    /// p p p p p p p p
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . P . . .
    /// . . . . . . . .
    /// P P P P . P P P
    /// R N B Q K B N R
    /// <BLANKLINE>
    /// r n b q k b n r
    /// p p p p p p p p
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . P . . . .
    /// . . . . . . . .
    /// P P P . P P P P
    /// R N B Q K B N R
    ///
    /// ```
    #[pyo3(signature = (chess_moves, check_legality = true))]
    #[allow(clippy::needless_pass_by_value)] // Can't pass reference of PyMove to Python
    #[inline]
    fn make_move_new(&self, chess_moves: Vec<PyMove>, check_legality: bool) -> PyResult<Self> {
        let count = self.boards.len();

        let mut new_boards = Vec::with_capacity(count);
        let mut move_gens = Vec::with_capacity(count);
        let mut halfmove_clocks = Vec::with_capacity(count);
        let mut fullmove_numbers = Vec::with_capacity(count);
        let mut board_histories = Vec::with_capacity(count);

        for i in 0..count {
            // Check if draw by fivefold
            if PyBoard::_is_n_repetition(
                self.board_histories[i].as_ref(),
                self.halfmove_clocks[i],
                5,
            ) {
                return Err(PyValueError::new_err(
                    "Game over due to fivefold repetition",
                ));
            }

            // If we are checking legality, check if the move is legal
            if check_legality && !PyBoard::_is_legal_move(&self.boards[i], chess_moves[i]) {
                return Err(PyValueError::new_err("Illegal move"));
            }

            // Make the move onto a new board using the chess crate
            new_boards.push(self.boards[i].make_move_new(chess_moves[i].0));

            // Reset the halfmove clock if the move zeroes (is a capture or pawn move and therefore "zeroes" the halfmove clock)
            let is_zeroing = if PyBoard::_is_zeroing(&self.boards[i], chess_moves[i]) {
                halfmove_clocks.push(0);
                true
            } else {
                halfmove_clocks.push(self.halfmove_clocks[i] + 1); // Add one if not zeroing
                false
            };

            // Increment fullmove number if black moves
            fullmove_numbers
                .push(self.fullmove_numbers[i] + self.boards[i].side_to_move().to_index() as u8); // White is 0, black is 1

            board_histories.push(self.board_histories[i].as_ref().map(|history| {
                let mut history = history.clone();
                if is_zeroing {
                    // Don't need previous history since it's a zeroing move (irreversible)
                    history.clear();
                }
                history.push(new_boards[i].get_hash());
                history
            }));

            move_gens.push(OnceLock::new());
        }

        Ok(Self {
            boards: new_boards,
            move_gens,
            halfmove_clocks,
            fullmove_numbers,
            repetition_detection_mode: self.repetition_detection_mode,
            board_histories,
        })
    }

    /// Get the bitboard of the side to move's pinned pieces for each board.
    ///
    #[inline]
    fn get_pinned_bitboard(&self) -> Vec<PyBitboard> {
        self.boards
            .iter()
            .map(PyBoard::_get_pinned_bitboard)
            .collect()
    }

    /// Get the bitboard of the pieces putting the side to move in check for each board.
    ///
    #[inline]
    fn get_checkers_bitboard(&self) -> Vec<PyBitboard> {
        self.boards
            .iter()
            .map(PyBoard::_get_checkers_bitboard)
            .collect()
    }

    /// Get the bitboard of all the pieces of a certain color for each board.
    ///
    #[inline]
    fn get_color_bitboard(&self, color: PyColor) -> Vec<PyBitboard> {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_color_bitboard(board, color))
            .collect()
    }

    /// Get the bitboard of all the pieces of a certain type for each board.
    ///
    #[inline]
    fn get_piece_type_bitboard(&self, piece_type: PyPieceType) -> Vec<PyBitboard> {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_piece_type_bitboard(board, piece_type))
            .collect()
    }

    /// Get the bitboard of all the pieces of a certain color and type for each board.
    ///
    #[inline]
    fn get_piece_bitboard(&self, piece: PyPiece) -> Vec<PyBitboard> {
        self.boards
            .iter()
            .map(|board| PyBoard::_get_piece_bitboard(board, piece))
            .collect()
    }

    /// Get the bitboard of all the pieces for each board.
    ///
    #[inline]
    fn get_all_bitboard(&self) -> Vec<PyBitboard> {
        self.boards.iter().map(PyBoard::_get_all_bitboard).collect()
    }

    /// Get the number of moves remaining in each move generator.
    /// This is the number of remaining moves that can be generated.
    /// Does not consume any iterations.
    ///
    #[inline]
    fn get_generator_num_remaining(&self) -> Vec<usize> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        self.boards
            .iter()
            .zip(self.move_gens.iter())
            .map(|(board, move_gen)| PyBoard::_get_generator_num_remaining(py, board, move_gen))
            .collect()
    }

    /// Get the sum of all moves remaining in each move generator.
    /// This is the total number of remaining moves that can be generated for the batch.
    /// Does not consume any iterations.
    ///
    #[inline]
    fn get_total_generator_num_remaining(&self) -> usize {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        self.boards
            .iter()
            .zip(self.move_gens.iter())
            .map(|(board, move_gen)| PyBoard::_get_generator_num_remaining(py, board, move_gen))
            .sum()
    }

    /// Reset the move generator for each board.
    ///
    #[inline]
    fn reset_move_generator(&mut self) {
        // // Invalidate each move generator
        self.move_gens.iter_mut().for_each(|lock| {
            lock.take();
        });
    }

    /// Remove a respective move from each move generator.
    /// Prevents the move from being generated by its generator.
    /// Updates the generator mask to exclude the move.
    /// Useful if you already have a certain move and don't need to generate it again.
    ///
    #[allow(clippy::needless_pass_by_value)] // Can't pass reference of PyMove to Python
    #[inline]
    fn remove_generator_move(&mut self, chess_moves: Vec<PyMove>) {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        self.boards
            .iter()
            .zip(self.move_gens.iter())
            .zip(chess_moves.iter())
            .for_each(|((board, move_gen), chess_move)| {
                PyBoard::_remove_generator_move(py, board, move_gen, *chess_move);
            });
    }

    /// Retains only moves whose destination squares are in the given mask respectively.
    ///
    #[allow(clippy::needless_pass_by_value)] // Can't pass reference of PyBitboard to Python
    #[inline]
    fn retain_generator_mask(&mut self, masks: Vec<PyBitboard>) {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        self.boards
            .iter()
            .zip(self.move_gens.iter())
            .zip(masks.iter())
            .for_each(|((board, move_gen), mask)| {
                PyBoard::_retain_generator_mask(py, board, move_gen, *mask);
            });
    }

    /// Excludes moves whose destination squares are in the given mask respectively.
    ///
    /// The mask is a bitboard of forbidden landing squares.
    /// Only moves landing on squares not in the mask will be generated.
    /// See `retain_generator_mask` for the inverse.
    ///
    /// Removed moves stay removed even if you change the mask.
    ///
    #[allow(clippy::needless_pass_by_value)] // Can't pass reference of PyBitboard to Python
    #[inline]
    fn exclude_generator_mask(&mut self, masks: Vec<PyBitboard>) {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        self.boards
            .iter()
            .zip(self.move_gens.iter())
            .zip(masks.iter())
            .for_each(|((board, move_gen), mask)| {
                PyBoard::_exclude_generator_mask(py, board, move_gen, *mask);
            });
    }

    /// Get the next remaining move in each generator.
    /// Updates each move generator to the next move.
    ///
    /// Unless a mask has been set, this will return the next legal move by default for each board.
    ///
    #[inline]
    fn generate_next_move(&mut self) -> Vec<Option<PyMove>> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        self.boards
            .iter()
            .zip(self.move_gens.iter())
            .map(|(board, move_gen)| PyBoard::_generate_next_move(py, board, move_gen))
            .collect()
    }

    /// Get the next remaining legal move in each generator.
    /// Updates each move generator to the next legal move.
    ///
    /// Allows all legal destination squares for each generator.
    ///
    #[inline]
    fn generate_next_legal_move(&mut self) -> Vec<Option<PyMove>> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        self.boards
            .iter()
            .zip(self.move_gens.iter())
            .map(|(board, move_gen)| PyBoard::_generate_next_legal_move(py, board, move_gen))
            .collect()
    }

    /// Get the next remaining legal capture in each generator.
    /// Updates each move generator to the next move.
    ///
    /// Allows only enemy-occupied destination squares for each generator.
    ///
    #[inline]
    fn generate_next_legal_capture(&mut self) -> Vec<Option<PyMove>> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        self.boards
            .iter()
            .zip(self.move_gens.iter())
            .map(|(board, move_gen)| PyBoard::_generate_next_legal_capture(py, board, move_gen))
            .collect()
    }

    // TODO: Generate moves_list (Vec<PyMove>)

    /// Generate the next remaining moves for each board.
    /// Exhausts each move generator if fully iterated over.
    /// Updates each move generator.
    ///
    /// Unless a mask has been set, this will generate the next legal moves by default for each board.
    ///
    #[inline]
    fn generate_moves(&mut self) -> Vec<Py<PyMoveGenerator>> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        self.boards
            .iter()
            .zip(self.move_gens.iter())
            .map(|(board, move_gen)| PyBoard::_generate_moves(py, board, move_gen))
            .collect()
    }

    /// Generate the next remaining legal moves for each board.
    /// Exhausts each move generator if fully iterated over.
    /// Updates each move generator.
    ///
    /// Will not iterate over moves already generated.
    ///
    #[inline]
    fn generate_legal_moves(&mut self) -> Vec<Py<PyMoveGenerator>> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        self.boards
            .iter()
            .zip(self.move_gens.iter())
            .map(|(board, move_gen)| PyBoard::_generate_legal_moves(py, board, move_gen))
            .collect()
    }

    /// Generate the next remaining legal captures for each current board.
    /// Exhausts each move generator if fully iterated over.
    /// Updates each move generator.
    ///
    /// Can iterate over legal captures first and then legal moves without any duplicated moves.
    /// Useful for move ordering, in case you want to check captures first before generating other moves.
    ///
    #[inline]
    fn generate_legal_captures(&mut self) -> Vec<Py<PyMoveGenerator>> {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };

        self.boards
            .iter()
            .zip(self.move_gens.iter())
            .map(|(board, move_gen)| PyBoard::_generate_legal_captures(py, board, move_gen))
            .collect()
    }

    /// Checks if the halfmoves since the last pawn move or capture is >= 100
    /// and the game is ongoing (not checkmate or stalemate) for each board.
    ///
    /// This is a claimable draw according to FIDE rules.
    ///
    #[inline]
    fn is_fifty_moves(&self) -> Vec<bool> {
        self.boards
            .iter()
            .zip(self.halfmove_clocks.iter())
            .map(|(board, halfmove_clock)| PyBoard::_is_fifty_moves(board, *halfmove_clock))
            .collect()
    }

    /// Checks if the halfmoves since the last pawn move or capture is >= 150
    /// and the game is ongoing (not checkmate or stalemate) for each board.
    ///
    /// This is an automatic draw according to FIDE rules.
    ///
    #[inline]
    fn is_seventy_five_moves(&self) -> Vec<bool> {
        self.boards
            .iter()
            .zip(self.halfmove_clocks.iter())
            .map(|(board, halfmove_clock)| PyBoard::_is_seventy_five_moves(board, *halfmove_clock))
            .collect()
    }

    /// Checks if the side to move has insufficient material to checkmate the opponent for each board.
    /// The cases where this is true are:
    ///     1. K vs K
    ///     2. K vs K + N
    ///     3. K vs K + B
    ///     4. K + B vs K + B with the bishops on the same color.
    ///
    #[inline]
    fn is_insufficient_material(&self) -> Vec<bool> {
        self.boards
            .iter()
            .map(PyBoard::_is_insufficient_material)
            .collect()
    }

    /// Checks if the current position is a n-fold repetition for each board.
    ///
    /// TODO: Quick check (only check last few moves since that is common error for engines)
    /// TODO: Add option to use full, or no repetition checks
    #[inline]
    fn is_n_repetition(&self, n: u8) -> Vec<bool> {
        self.board_histories
            .iter()
            .zip(self.halfmove_clocks.iter())
            .map(|(board_history, halfmove_clock)| {
                PyBoard::_is_n_repetition(board_history.as_ref(), *halfmove_clock, n)
            })
            .collect()
    }

    /// Checks if the current position is a threefold repetition for each board.
    /// This is a claimable draw according to FIDE rules.
    ///
    #[inline]
    fn is_threefold_repetition(&self) -> Vec<bool> {
        self.is_n_repetition(3)
    }

    /// Checks if the current position is a fivefold repetition for each board.
    /// This is an automatic draw according to FIDE rules.
    ///
    #[inline]
    fn is_fivefold_repetition(&self) -> Vec<bool> {
        self.is_n_repetition(5)
    }

    /// Checks if the side to move is in check for each board.
    ///
    #[inline]
    fn is_check(&self) -> Vec<bool> {
        self.boards.iter().map(PyBoard::_is_check).collect()
    }

    /// Checks if the side to move is in stalemate for each board.
    ///
    #[inline]
    fn is_stalemate(&self) -> Vec<bool> {
        self.boards.iter().map(PyBoard::_is_stalemate).collect()
    }

    /// Checks if the side to move is in checkmate for each board.
    ///
    #[inline]
    fn is_checkmate(&self) -> Vec<bool> {
        self.boards.iter().map(PyBoard::_is_checkmate).collect()
    }

    /// Get the status of each board (ongoing, draw, or game-ending).
    ///
    #[inline]
    fn get_status(&self) -> Vec<PyBoardStatus> {
        self.boards
            .iter()
            .zip(self.board_histories.iter())
            .zip(self.halfmove_clocks.iter())
            .map(|((board, board_history), halfmove_clock)| {
                PyBoard::_get_status(board, board_history.as_ref(), *halfmove_clock)
            })
            .collect()
    }
}

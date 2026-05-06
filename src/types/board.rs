use std::str::FromStr;
use std::sync::OnceLock;
use std::{fmt::Write, sync::LazyLock};

use pyo3::{exceptions::PyValueError, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pyclass_enum, gen_stub_pymethods};

use crate::types::{
    bitboard::PyBitboard,
    color::{BLACK, PyColor, WHITE},
    r#move::{PyMove, PyMoveGenerator},
    piece::{PyPiece, PyPieceType},
    square::PySquare,
};

pub static DEFAULT_BOARD: LazyLock<PyBoard> = LazyLock::new(|| {
    let board = chess::Board::default();
    let mut history = Vec::with_capacity(256);
    history.push(board.get_hash());
    PyBoard {
        board,
        move_gen: OnceLock::new(),
        halfmove_clock: 0,
        fullmove_number: 1,
        repetition_detection_mode: PyRepetitionDetectionMode::Full,
        board_history: Some(history),
    }
});

/// Board status enum class.
/// Represents the status of a chess board.
/// The status can be one of the following:
///     Ongoing, seventy-five moves, five-fold repetition, insufficient material, stalemate, or checkmate.
/// Supports comparison and equality.
/// TODO: docs
#[gen_stub_pyclass_enum]
#[pyclass(name = "BoardStatus", frozen, eq, ord, from_py_object)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd)]
pub enum PyBoardStatus {
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

/// Castle rights enum class.
/// The castle rights can be one of the following:
///     No rights, king-side, queen-side, both.
/// TODO: docs
#[gen_stub_pyclass_enum]
#[pyclass(name = "CastleRights", frozen, eq, ord, from_py_object)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd)]
pub enum PyCastleRights {
    #[pyo3(name = "NO_RIGHTS")]
    NoRights,
    #[pyo3(name = "QUEEN_SIDE")]
    QueenSide,
    #[pyo3(name = "KING_SIDE")]
    KingSide,
    #[pyo3(name = "BOTH")]
    Both,
}

#[gen_stub_pyclass_enum]
#[pyclass(name = "RepetitionDetectionMode", frozen, eq, ord, from_py_object)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd)]
pub enum PyRepetitionDetectionMode {
    #[pyo3(name = "NONE")]
    None,
    #[pyo3(name = "FULL")]
    Full,
}

/// Board class.
/// Represents the state of a chess board.
///
/// TODO: docs
#[gen_stub_pyclass]
#[pyclass(name = "Board", from_py_object)]
pub struct PyBoard {
    pub(crate) board: chess::Board,

    // Use a OnceLock to lazily initialize the move generator when needed
    pub(crate) move_gen: OnceLock<Py<PyMoveGenerator>>, // Use a Py to be able to share between Python and Rust

    /// Get the halfmove clock.
    ///
    /// ```python
    /// >>> rust_chess.Board().halfmove_clock
    /// 0
    /// ```
    #[pyo3(get)]
    pub(crate) halfmove_clock: u8, // Halfmoves since last pawn move or capture

    /// Get the fullmove number.
    ///
    /// ```python
    /// >>> rust_chess.Board().fullmove_number
    /// 1
    /// ```
    #[pyo3(get)]
    pub(crate) fullmove_number: u8, // Fullmove number; increments after black moves (theoretical max 218, fits in u8)

    /// The repetition dectection mode the board will use.
    #[pyo3(get)]
    pub(crate) repetition_detection_mode: PyRepetitionDetectionMode,

    /// Stores board Zobrist hashes for board history.
    #[pyo3(get)]
    pub(crate) board_history: Option<Vec<u64>>,
}

/// Rust only helpers.
/// Defined here so that `PyBoard` and `PyBoardBatch` can use the same functions (don't want to store `PyBoard`s in `PyBoardBatch`).
/// Documented in the Python section so the stubs can be automatically generated.
impl PyBoard {
    // TODO: Pass by value instead of reference?

    /// Helper to lazily initialize and return a reference to the generator
    #[inline]
    pub(crate) fn ensure_move_gen(
        py: Python<'_>,
        board: &chess::Board,
        move_gen: &OnceLock<Py<PyMoveGenerator>>,
    ) -> Py<PyMoveGenerator> {
        move_gen
            .get_or_init(|| Py::new(py, PyMoveGenerator::new(board)).unwrap())
            .clone_ref(py)
    }

    #[inline]
    pub(crate) fn _get_fen(
        board: &chess::Board,
        halfmove_clock: u8,
        fullmove_number: u8,
    ) -> String {
        let base_fen = board.to_string();

        // 0: board, 1: player, 2: castling, 3: en passant, 4: halfmove clock, 5: fullmove number
        let base_parts: Vec<&str> = base_fen.split_whitespace().collect();

        // The chess crate doesn't handle the halfmove and fullmove values so we need to do it ourselves
        format!(
            "{} {} {} {} {} {}",
            base_parts[0],   // board
            base_parts[1],   // player
            base_parts[2],   // castling
            base_parts[3],   // en passant
            halfmove_clock,  // halfmove clock
            fullmove_number, // fullmove number
        )
    }

    // Third argument included so we can reuse this function in `_display_tiled`
    #[inline]
    pub(crate) fn _display(board: &chess::Board, show_labels: bool, _: bool) -> String {
        let mut s = String::with_capacity(136); // (64 squares * 2 chars per square) + 8 newlines
        for rank in (0..8).rev() {
            if show_labels {
                // Print rank number on the left
                unsafe { write!(s, "{} ", rank + 1).unwrap_unchecked() }; // Safe code is for weaklings
            }

            for file in 0..8 {
                let square = PySquare(unsafe { chess::Square::new(file + (rank * 8)) });
                if let Some(piece) = Self::_get_piece_on(board, square) {
                    s.push_str(&piece.get_string()); // TODO: as_str()?
                    s.push(' ');
                } else {
                    s.push_str(". ")
                }
            }
            s.push('\n');
        }

        if show_labels {
            // Print file letters on the bottom
            s.push_str("  a b c d e f g h \n");
        }

        s.push('\n');
        s
    }

    #[inline]
    pub(crate) fn _display_unicode(
        board: &chess::Board,
        show_labels: bool,
        dark_mode: bool,
    ) -> String {
        let mut s = String::with_capacity(232); // Default board string size
        for rank in (0..8).rev() {
            if show_labels {
                // Print rank number on the left
                unsafe { write!(s, "{} ", rank + 1).unwrap_unchecked() }; // Safe code is for weaklings
            }

            for file in 0..8 {
                let square = PySquare(unsafe { chess::Square::new(file + (rank * 8)) });
                if let Some(piece) = Self::_get_piece_on(board, square) {
                    s.push_str(piece.get_unicode(dark_mode));
                    s.push(' ');
                } else {
                    s.push_str("· ") // This is a unicode middle dot, not a period
                }
            }
            s.push('\n');
        }

        if show_labels {
            // Print file letters on the bottom
            s.push_str("  a b c d e f g h \n");
        }

        s.push('\n');
        s
    }

    // TODO: Make this look less ugly?
    // TODO: Add picture of this to demo
    #[inline]
    pub(crate) fn _display_color(
        board: &chess::Board,
        show_labels: bool,
        green_mode: bool,
    ) -> String {
        // TODO: Make these constants?
        let white_code = "255;255;255";
        let black_code = "0;0;0";

        let (light_square_code, dark_square_code) = match green_mode {
            // Tan/Brown
            false => ("230;207;171", "181;136;99"),
            // Olive/Sand
            true => ("215;220;200", "118;150;86"),
        };

        let mut s = String::with_capacity(2666); // Default board string size with labels
        for rank in (0..8).rev() {
            if show_labels {
                // Print rank number on the left
                unsafe { write!(s, "{} ", rank + 1).unwrap_unchecked() }; // Safe code is for weaklings
            }

            for file in 0..8 {
                let square = PySquare(unsafe { chess::Square::new(file + (rank * 8)) });

                let (symbol_color, symbol) = if let Some(piece) = Self::_get_piece_on(board, square)
                {
                    (
                        match piece.color.0 {
                            chess::Color::White => white_code,
                            chess::Color::Black => black_code,
                        },
                        piece.piece_type.get_solid_unicode(),
                    )
                } else {
                    (white_code, " ") // Color doesn't matter, empty square
                };

                let square_color = match square.get_color() {
                    WHITE => light_square_code,
                    BLACK => dark_square_code,
                };

                // Print the symbol with foreground and background color
                unsafe {
                    write!(
                        s,
                        "\x1b[38;2;{};48;2;{}m{} \x1b[0m",
                        symbol_color, square_color, symbol
                    )
                    .unwrap_unchecked()
                }
            }
            s.push('\n');
        }

        if show_labels {
            // Print file letters on the bottom
            s.push_str("  a b c d e f g h \n");
        }

        s.push('\n');
        s
    }

    #[inline]
    pub(crate) fn _display_tiled<F>(
        display_fn: F,
        board_string_size: usize,
        boards: &Vec<chess::Board>,
        show_labels: bool,
        color_mode: bool,
    ) where
        F: Fn(&chess::Board, bool, bool) -> String,
    {
        let num_boards: usize = boards.len();

        let terminal_width: usize = terminal_size::terminal_size()
            .map(|(terminal_size::Width(w), _)| w)
            .unwrap_or(80) as usize; // Default to 80 if we can't get the terminal size

        let tile_width: usize = if show_labels { 18 } else { 16 }; // (8 squares * 2 chars per square) + (2 chars for labels)
        let tile_height: usize = if show_labels { 9 } else { 8 }; // Bottom labels add an extra line

        let num_tiles_wide: usize = std::cmp::max(1, terminal_width / (tile_width + 1)); // + 1 for spacing; ensure at least 1 wide
        let num_tiles_high: usize = num_boards.div_ceil(num_tiles_wide); // Ceiling division

        // Iterate over each row
        let mut final_string =
            String::with_capacity((num_boards * board_string_size) + num_tiles_high); // + num tiles high for new lines
        let mut displays: Vec<String> = Vec::with_capacity(num_tiles_wide); // Stores each board's string
        for chunk in boards.chunks(num_tiles_wide) {
            // Clear the vector and add the new display strings
            displays.clear();
            displays.extend(chunk.iter().map(|b| display_fn(b, show_labels, color_mode)));

            // Iterate over each line
            for line_idx in 0..tile_height {
                // Iterate over each board (column)
                for (board_idx, display_str) in displays.iter().enumerate() {
                    // Add spacing between boards
                    if board_idx > 0 {
                        final_string.push(' ');
                    }
                    final_string
                        .push_str(unsafe { display_str.lines().nth(line_idx).unwrap_unchecked() }); // nth always succeeds since tile_height == line count
                }
                final_string.push('\n'); // Newline between rows
            }
            final_string.push('\n'); // Newline between board rows
        }

        // dbg!(final_string.len()); // Actual num bytes
        // dbg!((num_boards * board_string_size) + num_tiles_high); // Predicted num bytes

        print!("{}", final_string);
    }

    #[inline]
    pub(crate) fn _get_move_from_san(board: &chess::Board, san: &str) -> PyResult<PyMove> {
        chess::ChessMove::from_san(board, san)
            .map(PyMove)
            .map_err(|_| PyValueError::new_err("Invalid SAN move"))
    }

    #[inline]
    pub(crate) fn _get_zobrist_hash(board: &chess::Board) -> u64 {
        board.get_hash()
    }

    #[inline]
    pub(crate) fn _get_turn(board: &chess::Board) -> chess::Color {
        board.side_to_move()
    }

    #[inline]
    pub(crate) fn _get_king_square(board: &chess::Board, color: PyColor) -> PySquare {
        PySquare(board.king_square(color.0))
    }

    #[inline]
    pub(crate) fn _get_castle_rights(board: &chess::Board, color: chess::Color) -> PyCastleRights {
        match board.castle_rights(color) {
            chess::CastleRights::NoRights => PyCastleRights::NoRights,
            chess::CastleRights::QueenSide => PyCastleRights::QueenSide,
            chess::CastleRights::KingSide => PyCastleRights::KingSide,
            chess::CastleRights::Both => PyCastleRights::Both,
        }
    }

    #[inline]
    pub(crate) fn _get_my_castle_rights(board: &chess::Board) -> PyCastleRights {
        Self::_get_castle_rights(board, board.side_to_move())
    }

    #[inline]
    pub(crate) fn _get_their_castle_rights(board: &chess::Board) -> PyCastleRights {
        Self::_get_castle_rights(board, !board.side_to_move())
    }

    #[inline]
    pub(crate) fn _can_castle(board: &chess::Board, color: PyColor) -> bool {
        board.castle_rights(color.0) != chess::CastleRights::NoRights
    }

    #[inline]
    pub(crate) fn _can_castle_queenside(board: &chess::Board, color: PyColor) -> bool {
        board.castle_rights(color.0).has_queenside()
    }

    #[inline]
    pub(crate) fn _can_castle_kingside(board: &chess::Board, color: PyColor) -> bool {
        board.castle_rights(color.0).has_kingside()
    }

    #[inline]
    pub(crate) fn _is_castling(board: &chess::Board, chess_move: PyMove) -> bool {
        let source = chess_move.0.get_source();

        // Check if the moving piece is a king
        if board
            .piece_on(source)
            .is_some_and(|p| p == chess::Piece::King)
        {
            // Check if the move is two squares horizontally
            let dest = chess_move.0.get_dest();
            return (dest.to_int() as i8 - source.to_int() as i8).abs() == 2;
        }
        false
    }

    #[inline]
    pub(crate) fn _is_castling_queenside(board: &chess::Board, chess_move: PyMove) -> bool {
        let source = chess_move.0.get_source();

        // Check if the moving piece is a king
        if board
            .piece_on(source)
            .is_some_and(|p| p == chess::Piece::King)
        {
            // Check if the move is two squares to the left
            let dest = chess_move.0.get_dest();
            #[allow(clippy::cast_possible_truncation)]
            return dest.to_int() as i8 - source.to_int() as i8 == -2;
        }
        false
    }

    #[inline]
    pub(crate) fn _is_castling_kingside(board: &chess::Board, chess_move: PyMove) -> bool {
        let source = chess_move.0.get_source();

        // Check if the moving piece is a king
        if board
            .piece_on(source)
            .is_some_and(|p| p == chess::Piece::King)
        {
            // Check if the move is two squares to the right
            let dest = chess_move.0.get_dest();
            #[allow(clippy::cast_possible_truncation)]
            return dest.to_int() as i8 - source.to_int() as i8 == 2;
        }
        false
    }

    #[inline]
    pub(crate) fn _get_color_on(board: &chess::Board, square: PySquare) -> Option<PyColor> {
        // Get the color of the piece on the square using the chess crate
        board.color_on(square.0).map(PyColor)
    }

    #[inline]
    pub(crate) fn _get_piece_type_on(
        board: &chess::Board,
        square: PySquare,
    ) -> Option<PyPieceType> {
        // Get the piece on the square using the chess crate
        board.piece_on(square.0).map(PyPieceType)
    }

    #[inline]
    pub(crate) fn _get_piece_on(board: &chess::Board, square: PySquare) -> Option<PyPiece> {
        Self::_get_color_on(board, square).and_then(|color| {
            Self::_get_piece_type_on(board, square).map(|piece_type| PyPiece { piece_type, color })
        })
    }

    #[inline]
    pub(crate) fn _get_en_passant(board: &chess::Board) -> Option<PySquare> {
        // The Rust chess crate doesn't actually compute this right; it returns the square that the pawn was moved to.
        // The actual en passant square is the one that one can move to that would cause en passant.
        // TLDR: The actual en passant square is one above or below the one returned by the chess crate.
        board.en_passant().map(|sq| match board.side_to_move() {
            chess::Color::White => PySquare(sq.up().unwrap()),
            chess::Color::Black => PySquare(sq.down().unwrap()),
        })
    }

    #[inline]
    pub(crate) fn _is_en_passant(board: &chess::Board, chess_move: PyMove) -> bool {
        let source = chess_move.0.get_source();
        let dest = chess_move.0.get_dest();

        // The Rust chess crate doesn't actually compute this right; it returns the square that the pawn was moved to.
        // The actual en passant square is the one that one can move to that would cause en passant.
        // TLDR: The actual en passant square is one above or below the one returned by the chess crate.
        let ep_square = board
            .en_passant()
            .and_then(|sq| match board.side_to_move() {
                chess::Color::White => sq.up(),
                chess::Color::Black => sq.down(),
            });

        ep_square.is_some_and(|ep_sq| ep_sq == dest) // Use our en passant square function since it is accurate
            && board.piece_on(source).is_some_and(|p| p == chess::Piece::Pawn) // Moving pawn
            && {
                // Moving diagonally
                #[allow(clippy::cast_possible_truncation)]
                let diff = (dest.to_int() as i8 - source.to_int() as i8).abs();
                diff == 7 || diff == 9
            }
            && board.piece_on(dest).is_none() // Target square is empty
    }

    #[inline]
    pub(crate) fn _is_capture(board: &chess::Board, chess_move: PyMove) -> bool {
        board.piece_on(chess_move.0.get_dest()).is_some() // Capture (moving piece onto other piece)
            || Self::_is_en_passant(board, chess_move) // Or the move is en passant (also a capture)
    }

    pub(crate) fn _is_zeroing(board: &chess::Board, chess_move: PyMove) -> bool {
        board.piece_on(chess_move.0.get_source()).is_some_and(|p| p == chess::Piece::Pawn) // Pawn move
        || board.piece_on(chess_move.0.get_dest()).is_some() // Capture (moving piece onto other piece)
    }

    #[inline]
    pub(crate) fn _is_legal_move(board: &chess::Board, chess_move: PyMove) -> bool {
        board.legal(chess_move.0)
    }

    #[inline]
    pub(crate) fn _is_legal_generator_move(board: &chess::Board, chess_move: PyMove) -> bool {
        chess::MoveGen::legal_quick(board, chess_move.0)
    }

    #[inline]
    pub(crate) fn _get_pinned_bitboard(board: &chess::Board) -> PyBitboard {
        PyBitboard(*board.pinned())
    }

    #[inline]
    pub(crate) fn _get_checkers_bitboard(board: &chess::Board) -> PyBitboard {
        PyBitboard(*board.checkers())
    }

    #[inline]
    pub(crate) fn _get_color_bitboard(board: &chess::Board, color: PyColor) -> PyBitboard {
        PyBitboard(*board.color_combined(color.0))
    }

    #[inline]
    pub(crate) fn _get_piece_type_bitboard(
        board: &chess::Board,
        piece_type: PyPieceType,
    ) -> PyBitboard {
        PyBitboard(*board.pieces(piece_type.0))
    }

    #[inline]
    pub(crate) fn _get_piece_bitboard(board: &chess::Board, piece: PyPiece) -> PyBitboard {
        PyBitboard(board.pieces(piece.piece_type.0) & board.color_combined(piece.color.0))
    }

    #[inline]
    pub(crate) fn _get_all_bitboard(board: &chess::Board) -> PyBitboard {
        PyBitboard(*board.combined())
    }

    #[inline]
    pub(crate) fn _get_generator_num_remaining(
        py: Python<'_>,
        board: &chess::Board,
        move_gen: &OnceLock<Py<PyMoveGenerator>>,
    ) -> usize {
        Self::ensure_move_gen(py, board, move_gen)
            .borrow(py)
            .__len__()
    }

    #[inline]
    pub(crate) fn _remove_generator_move(
        py: Python<'_>,
        board: &chess::Board,
        move_gen: &OnceLock<Py<PyMoveGenerator>>,
        chess_move: PyMove,
    ) {
        Self::ensure_move_gen(py, board, move_gen)
            .borrow_mut(py)
            .remove_move(chess_move.0);
    }

    #[inline]
    pub(crate) fn _retain_generator_mask(
        py: Python<'_>,
        board: &chess::Board,
        move_gen: &OnceLock<Py<PyMoveGenerator>>,
        mask: PyBitboard,
    ) {
        Self::ensure_move_gen(py, board, move_gen)
            .borrow_mut(py)
            .retain_mask(mask.0);
    }

    #[inline]
    pub(crate) fn _exclude_generator_mask(
        py: Python<'_>,
        board: &chess::Board,
        move_gen: &OnceLock<Py<PyMoveGenerator>>,
        mask: PyBitboard,
    ) {
        Self::ensure_move_gen(py, board, move_gen)
            .borrow_mut(py)
            .exclude_mask(mask.0);
    }

    #[inline]
    pub(crate) fn _generate_next_move(
        py: Python<'_>,
        board: &chess::Board,
        move_gen: &OnceLock<Py<PyMoveGenerator>>,
    ) -> Option<PyMove> {
        Self::ensure_move_gen(py, board, move_gen)
            .borrow_mut(py)
            .__next__()
    }

    #[inline]
    pub(crate) fn _generate_next_legal_move(
        py: Python<'_>,
        board: &chess::Board,
        move_gen: &OnceLock<Py<PyMoveGenerator>>,
    ) -> Option<PyMove> {
        let gen_ref = Self::ensure_move_gen(py, board, move_gen);

        // Allow all destination squares again for iteration
        gen_ref.borrow_mut(py).retain_mask(!chess::EMPTY);

        gen_ref.borrow_mut(py).__next__()
    }

    #[inline]
    pub(crate) fn _generate_next_legal_capture(
        py: Python<'_>,
        board: &chess::Board,
        move_gen: &OnceLock<Py<PyMoveGenerator>>,
    ) -> Option<PyMove> {
        // Get the mask of enemy‐occupied squares
        let targets_mask = board.color_combined(!board.side_to_move());

        let gen_ref = Self::ensure_move_gen(py, board, move_gen);

        // Allow only capture destination squares for iteration
        gen_ref.borrow_mut(py).retain_mask(*targets_mask);

        gen_ref.borrow_mut(py).__next__()
    }

    // TODO: Generate moves_list (Vec<PyMove>)

    #[inline]
    pub(crate) fn _generate_moves(
        py: Python<'_>,
        board: &chess::Board,
        move_gen: &OnceLock<Py<PyMoveGenerator>>,
    ) -> Py<PyMoveGenerator> {
        // Share ownership with Python
        Self::ensure_move_gen(py, board, move_gen)
    }

    #[inline]
    pub(crate) fn _generate_legal_moves(
        py: Python<'_>,
        board: &chess::Board,
        move_gen: &OnceLock<Py<PyMoveGenerator>>,
    ) -> Py<PyMoveGenerator> {
        let gen_ref = Self::ensure_move_gen(py, board, move_gen);

        // Allow all destination squares again for iteration
        gen_ref.borrow_mut(py).retain_mask(!chess::EMPTY);

        // Share ownership with Python
        gen_ref
    }

    #[inline]
    pub(crate) fn _generate_legal_captures(
        py: Python<'_>,
        board: &chess::Board,
        move_gen: &OnceLock<Py<PyMoveGenerator>>,
    ) -> Py<PyMoveGenerator> {
        // Get the mask of enemy‐occupied squares
        let targets_mask = board.color_combined(!board.side_to_move());

        let gen_ref = Self::ensure_move_gen(py, board, move_gen);

        // Allow only capture destination squares for iteration
        gen_ref.borrow_mut(py).retain_mask(*targets_mask);

        // Share ownership with Python
        gen_ref
    }

    #[inline]
    pub(crate) fn _is_fifty_moves(board: &chess::Board, halfmove_clock: u8) -> bool {
        halfmove_clock >= 100 && board.status() == chess::BoardStatus::Ongoing
    }

    #[inline]
    pub(crate) fn _is_seventy_five_moves(board: &chess::Board, halfmove_clock: u8) -> bool {
        halfmove_clock >= 150 && board.status() == chess::BoardStatus::Ongoing
    }

    #[inline]
    pub(crate) fn _is_insufficient_material(board: &chess::Board) -> bool {
        let kings = board.pieces(chess::Piece::King);

        // Get the bitboards of the white and black pieces without the kings
        let white_bb = board.color_combined(chess::Color::White) & !kings;
        let black_bb = board.color_combined(chess::Color::Black) & !kings;
        let combined_bb = white_bb | black_bb;

        // King vs King: Combined bitboard minus kings is empty
        if combined_bb == chess::EMPTY {
            return true;
        }

        let num_remaining_pieces = combined_bb.popcnt();
        if num_remaining_pieces <= 2 {
            let knights = board.pieces(chess::Piece::Knight);
            let bishops = board.pieces(chess::Piece::Bishop);

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

    /// Very efficient repetition detection algorithm.
    /// TODO: Quick check (only check last few moves since that is common error for engines)
    /// TODO: Add option to use full, or no repetition checks
    #[inline]
    pub(crate) fn _is_n_repetition(
        board_history: &Option<Vec<u64>>,
        halfmove_clock: u8,
        n: u8,
    ) -> bool {
        if let Some(history) = board_history {
            // Move history length is one greater than the halfmove clock since when halfmove clock is 0, there is 1 position in history
            let length: i16 = i16::from(halfmove_clock + 1);
            // If checking threefold (n = 3), then it would be (4 * (3-1)) + 1 = 9
            // Fivefold requires 17 positions minimum
            //   Takes 4 halfmoves to return to a position
            let calc_min_pos_req_for_nfold = |n: u8| -> i16 { i16::from((4 * (n - 1)) + 1) };

            // n-fold repetition is not possible when length is less than (n * 4) - 1
            // For example, threefold repetition (n=3) can occur with a move history length minimum of 9
            // A color cannot repeat a position back to back--some move has to be made, and then another to return to the position
            // Example: index 0, 4, 8 are the minimum required for a threefold repetition
            //   (2 and 6 are in-between positions that allow returning to repeated position (0, 4, 8))
            if length < calc_min_pos_req_for_nfold(n) {
                return false;
            }

            #[allow(clippy::cast_sign_loss)]
            let current_hash: u64 = history[length as usize - 1];
            let mut num_repetitions: u8 = 1;

            // (length - 5) since we compare to current, which is at length - 1, and positions can't repeat back-to-back for a color
            let mut i: i16 = length - 5;
            // n-fold still possible if enough positions still left in history
            while i >= calc_min_pos_req_for_nfold(n - num_repetitions) - 1 {
                #[allow(clippy::cast_sign_loss)]
                if history[i as usize] == current_hash {
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

    #[inline]
    pub(crate) fn _is_check(board: &chess::Board) -> bool {
        *board.checkers() != chess::EMPTY
    }

    #[inline]
    pub(crate) fn _is_stalemate(board: &chess::Board) -> bool {
        board.status() == chess::BoardStatus::Stalemate
    }

    #[inline]
    pub(crate) fn _is_checkmate(board: &chess::Board) -> bool {
        board.status() == chess::BoardStatus::Checkmate
    }

    #[inline]
    pub(crate) fn _get_status(
        board: &chess::Board,
        board_history: &Option<Vec<u64>>,
        halfmove_clock: u8,
    ) -> PyBoardStatus {
        match board.status() {
            chess::BoardStatus::Ongoing => {
                if Self::_is_seventy_five_moves(board, halfmove_clock) {
                    PyBoardStatus::SeventyFiveMoves
                } else if Self::_is_insufficient_material(board) {
                    PyBoardStatus::InsufficientMaterial
                } else if Self::_is_n_repetition(board_history, halfmove_clock, 3) {
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

/// Implement clone for PyBoard manually since OnceLock doesn't implement clone
/// Allows us to use a default board constant
impl Clone for PyBoard {
    fn clone(&self) -> Self {
        Self {
            board: self.board,
            move_gen: OnceLock::new(), // Reset move generator when cloned
            halfmove_clock: self.halfmove_clock,
            fullmove_number: self.fullmove_number,
            repetition_detection_mode: self.repetition_detection_mode,
            board_history: self.board_history.clone(),
        }
    }
}

/// Python methods for `PyBoard`.
/// Calls the Rust helpers defined above.
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
        #[allow(clippy::option_if_let_else)]
        match fen {
            // If no FEN string is provided, use the default starting position
            None => match mode {
                // Used cached board if full repetition detection
                PyRepetitionDetectionMode::Full => Ok(DEFAULT_BOARD.clone()),
                PyRepetitionDetectionMode::None => Ok(Self {
                    board: DEFAULT_BOARD.board,
                    move_gen: OnceLock::new(),
                    halfmove_clock: 0,
                    fullmove_number: 1,
                    repetition_detection_mode: PyRepetitionDetectionMode::None,
                    board_history: None,
                }),
            },
            // Otherwise, parse the FEN string using the chess crate
            Some(fen_str) => Self::from_fen(fen_str, mode),
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

        // Create move history vector and add the initial board hash
        let board_history = match mode {
            PyRepetitionDetectionMode::None => None,
            PyRepetitionDetectionMode::Full => {
                let mut history = Vec::with_capacity(256);
                history.push(board.get_hash());
                Some(history)
            }
        };

        Ok(Self {
            board,
            move_gen: OnceLock::new(),
            halfmove_clock,
            fullmove_number,
            repetition_detection_mode: mode,
            board_history,
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
        Self::_get_fen(&self.board, self.halfmove_clock, self.fullmove_number)
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

    /// Print the string representation of the board.
    /// Labels are hidden by default.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.display()
    /// r n b q k b n r
    /// p p p p p p p p
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . . . . .
    /// P P P P P P P P
    /// R N B Q K B N R
    ///
    /// >>> board.display(show_labels=True)
    /// 8 r n b q k b n r
    /// 7 p p p p p p p p
    /// 6 . . . . . . . .
    /// 5 . . . . . . . .
    /// 4 . . . . . . . .
    /// 3 . . . . . . . .
    /// 2 P P P P P P P P
    /// 1 R N B Q K B N R
    ///   a b c d e f g h
    ///
    /// ```
    #[pyo3(signature = (show_labels = false))]
    #[inline]
    fn display(&self, show_labels: bool) {
        print!("{}", Self::_display(&self.board, show_labels, false)) // 3rd paramater doesn't do anything
    }

    /// Get the string representation of the board.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> print(board)
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
        Self::_display(&self.board, false, false) // 3rd parameter unused
    }

    /// Print the unicode string representation of the board.
    /// Labels are hidden by default.
    ///
    /// The dark mode parameter is enabled by default.
    /// This inverts the color of the piece, which looks correct on a dark background.
    /// Unicode assumes black text on white background, where in most terminals, it is the opposite.
    /// Disable if you are a psychopath and use light mode in your terminal/IDE.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.display_unicode() # This looks fine printed to terminal
    /// ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖
    /// ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙
    /// · · · · · · · ·
    /// · · · · · · · ·
    /// · · · · · · · ·
    /// · · · · · · · ·
    /// ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
    /// ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    ///
    /// >>> board.display_unicode(show_labels=True, dark_mode=False)
    /// 8 ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    /// 7 ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
    /// 6 · · · · · · · ·
    /// 5 · · · · · · · ·
    /// 4 · · · · · · · ·
    /// 3 · · · · · · · ·
    /// 2 ♙ ♙ ♙ ♙ ♙ ♙ ♙ ♙
    /// 1 ♖ ♘ ♗ ♕ ♔ ♗ ♘ ♖
    ///   a b c d e f g h
    ///
    /// ```
    #[pyo3(signature = (show_labels = false, dark_mode = true))]
    #[inline]
    fn display_unicode(&self, show_labels: bool, dark_mode: bool) {
        print!(
            "{}",
            Self::_display_unicode(&self.board, show_labels, dark_mode)
        )
    }

    /// Print the unicode string representation of the board with ANSI color codes.
    /// The board is a bit tiny, but it looks pretty good.
    /// Labels are shown by default (different than the other display functions).
    ///
    /// The default board color is tan/brown.
    /// Enable the `green_mode` parameter to change the color to olive/sand.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.display_color()
    /// ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    /// ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
    ///
    ///
    ///
    ///
    /// ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟
    /// ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    ///
    /// >>> board.display_color(show_labels=True, green_mode=True)
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
    #[pyo3(signature = (show_labels = true, green_mode = false))]
    #[inline]
    fn display_color(&self, show_labels: bool, green_mode: bool) {
        print!(
            "{}",
            Self::_display_color(&self.board, show_labels, green_mode)
        )
    } // TODO: Make colors better

    /// Create a new move from a SAN string (e.g. "e4").
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> board.get_move_from_san("e4")
    /// Move(e2, e4, None)
    /// ```
    #[inline]
    fn get_move_from_san(&self, san: &str) -> PyResult<PyMove> {
        Self::_get_move_from_san(&self.board, san)
    }

    // TODO: get_san_from_move

    // Get the Zobrist hash of the board.
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
        Self::_get_zobrist_hash(&self.board)
    }

    /// Get a hash of the board based on its Zobrist hash.
    /// **This is not the same as the `zobrist_hash` field since Python doesn't support unsigned 64-bit integers for this function.**
    /// Use `zobrist_hash` directly for the actual Zobrist hash value.
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
    fn __eq__(&self, other: &Self) -> bool {
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
    fn __ne__(&self, other: &Self) -> bool {
        !self.__eq__(other)
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
        PyColor(Self::_get_turn(&self.board))
    }

    /// Get the king square for a color.
    ///
    /// ```python
    /// >>> rust_chess.Board().get_king_square(rust_chess.WHITE)
    /// e1
    /// >>> rust_chess.Board().get_king_square(rust_chess.BLACK)
    /// e8
    /// ```
    #[inline]
    fn get_king_square(&self, color: PyColor) -> PySquare {
        Self::_get_king_square(&self.board, color)
    }

    /// Get the castle rights for a color.
    /// Returns a `CastleRights` enum type, which has the values: `NO_RIGHTS`, `KING_SIDE`, `QUEEN_SIDE`, `BOTH`.
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
        Self::_get_castle_rights(&self.board, color.0)
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
        Self::_get_my_castle_rights(&self.board)
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
        Self::_get_their_castle_rights(&self.board)
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
        Self::_can_castle(&self.board, color)
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
        Self::_can_castle_queenside(&self.board, color)
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
        Self::_can_castle_kingside(&self.board, color)
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
        Self::_is_castling(&self.board, chess_move)
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
        Self::_is_castling_queenside(&self.board, chess_move)
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
        Self::_is_castling_kingside(&self.board, chess_move)
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
        Self::_get_color_on(&self.board, square)
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
        Self::_get_piece_type_on(&self.board, square)
    }

    /// Get the piece on a square, otherwise None.
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
        Self::_get_piece_on(&self.board, square)
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
        Self::_get_en_passant(&self.board)
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
        Self::_is_en_passant(&self.board, chess_move)
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
        Self::_is_capture(&self.board, chess_move)
    }

    /// Check if a move is a capture or a pawn move.
    /// This type of move "zeros" the halfmove clock (sets it to 0).
    ///
    /// Assumes the move is legal.
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
        Self::_is_zeroing(&self.board, chess_move)
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
        Self::_is_legal_move(&self.board, chess_move)
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
    // TODO
    #[inline]
    fn is_legal_generator_move(&self, chess_move: PyMove) -> bool {
        Self::_is_legal_generator_move(&self.board, chess_move)
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
    fn make_null_move_new(&self) -> Option<Self> {
        // Make a null move onto a new board using the chess crate
        let new_board = self.board.null_move()?;

        Some(Self {
            board: new_board,

            // Create a new uninitialized move generator using the chess crate
            move_gen: OnceLock::new(),

            // Increment the halfmove clock
            halfmove_clock: self.halfmove_clock + 1, // Null moves aren't zeroing, so we can just add 1 here

            // Increment fullmove number if black moves
            #[allow(clippy::cast_possible_truncation)]
            fullmove_number: self.fullmove_number + (self.board.side_to_move().to_index() as u8), // White is 0, black is 1

            repetition_detection_mode: self.repetition_detection_mode,

            // Don't update move history when making a null move
            board_history: self.board_history.clone(),
        })
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
    // TODO: is_generator_move
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
            if let Some(history) = &mut self.board_history {
                history.clear();
            }
        } else {
            self.halfmove_clock += 1; // Add one if not zeroing
        }

        // Increment fullmove number if black moves
        self.fullmove_number += self.board.side_to_move().to_index() as u8; // White is 0, black is 1

        // Add the new board's Zobrist hash to history
        if let Some(history) = &mut self.board_history {
            history.push(temp_board.get_hash());
        }

        // Invalidate the move generator since the board has changed
        self.move_gen.take();

        // Update the current board
        self.board = temp_board;

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
    //TODO: Make move new quick legal?
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

        let is_zeroing: bool = self.is_zeroing(chess_move);

        Ok(Self {
            board: new_board,
            move_gen: OnceLock::new(),
            // Reset the halfmove clock if the move zeroes (is a capture or pawn move and therefore "zeroes" the halfmove clock)
            halfmove_clock: if is_zeroing {
                0
            } else {
                self.halfmove_clock + 1
            },
            // Increment fullmove number if black moves
            #[allow(clippy::cast_possible_truncation)]
            fullmove_number: self.fullmove_number + (self.board.side_to_move().to_index() as u8), // White is 0, black is 1
            repetition_detection_mode: self.repetition_detection_mode,
            // Add the new board's Zobrist hash to history
            board_history: self.board_history.as_ref().map(|history| {
                let mut new_history = if is_zeroing {
                    Vec::with_capacity(256) // Don't need previous history anymore since it is a zeroing move (irreversible)
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
        Self::_get_pinned_bitboard(&self.board)
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
        Self::_get_checkers_bitboard(&self.board)
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
        Self::_get_color_bitboard(&self.board, color)
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
        Self::_get_piece_type_bitboard(&self.board, piece_type)
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
        Self::_get_piece_bitboard(&self.board, piece)
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
        Self::_get_all_bitboard(&self.board)
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
        Self::_get_generator_num_remaining(py, &self.board, &self.move_gen)
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
    fn reset_move_generator(&mut self) {
        // Invalidate the move generator
        self.move_gen.take();
    }

    /// Remove a move from the move generator.
    /// Prevents the move from being generated.
    /// Updates the generator mask to exclude the move.
    /// Useful if you already have a certain move and don't need to generate it again.
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
    #[inline]
    fn remove_generator_move(&mut self, chess_move: PyMove) {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };
        Self::_remove_generator_move(py, &self.board, &self.move_gen, chess_move)
    }

    /// Retains only moves whose destination squares are in the given mask.
    ///
    /// The mask is a bitboard of allowed landing squares.
    /// Only moves landing on squares in the mask will be generated.
    /// See `exclude_generator_mask` for the inverse.
    ///
    /// Moves that have already been iterated over will not be generated again, regardless of the mask value.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> len(board.generate_moves())
    /// 20
    /// >>> board.retain_generator_mask(rust_chess.E4.to_bitboard())
    /// >>> len(board.generate_moves())
    /// 1
    /// >>> board.generate_next_move()
    /// Move(e2, e4, None)
    /// ```
    #[inline]
    fn retain_generator_mask(&mut self, mask: PyBitboard) {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };
        Self::_retain_generator_mask(py, &self.board, &self.move_gen, mask)
    }

    /// Excludes moves whose destination squares are in the given mask.
    ///
    /// The mask is a bitboard of forbidden landing squares.
    /// Only moves landing on squares not in the mask will be generated.
    /// See `retain_generator_mask` for the inverse.
    ///
    /// Removed moves stay removed even if you change the mask.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> len(board.generate_moves())
    /// 20
    /// >>> board.exclude_generator_mask(rust_chess.E4.to_bitboard())
    /// >>> len(board.generate_moves())
    /// 19
    /// >>> rust_chess.Move("e2e4") in board.generate_moves()
    /// False
    /// >>> len(board.generate_moves())
    /// 0
    /// ```
    #[inline]
    fn exclude_generator_mask(&mut self, mask: PyBitboard) {
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };
        Self::_exclude_generator_mask(py, &self.board, &self.move_gen, mask)
    }

    /// Get the next remaining move in the generator.
    /// Updates the move generator to the next move.
    ///
    /// Unless a mask has been set, this will return the next legal move by default.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> len(board.generate_moves())
    /// 20
    /// >>> board.remove_generator_move(rust_chess.Move("a2a3"))
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
        Self::_generate_next_move(py, &self.board, &self.move_gen)
    }

    /// Get the next remaining legal move in the generator.
    /// Updates the move generator to the next legal move.
    ///
    /// Allows all legal destination squares for the generator.
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
        Self::_generate_next_legal_move(py, &self.board, &self.move_gen)
    }

    /// Get the next remaining legal capture in the generator.
    /// Updates the move generator to the next move.
    ///
    /// Allows only enemy-occupied destination squares for the generator.
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
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };
        Self::_generate_next_legal_capture(py, &self.board, &self.move_gen)
    }

    // TODO: Generate moves_list (Vec<PyMove>)

    /// Generate the next remaining moves for the current board.
    /// Exhausts the move generator if fully iterated over.
    /// Updates the move generator.
    ///
    /// Unless a mask has been set, this will generate the next legal moves by default.
    ///
    /// ```python
    /// >>> board = rust_chess.Board()
    /// >>> len(board.generate_moves())
    /// 20
    /// >>> board.retain_generator_mask(rust_chess.Bitboard(402915328))
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
        Self::_generate_moves(py, &self.board, &self.move_gen)
    }

    /// Generate the next remaining legal moves for the current board.
    /// Exhausts the move generator if fully iterated over.
    /// Updates the move generator.
    ///
    /// Will not iterate over moves already generated.
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
        Self::_generate_legal_moves(py, &self.board, &self.move_gen)
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
        // We can assume the GIL is acquired, since this function is only called from Python
        let py = unsafe { Python::assume_attached() };
        Self::_generate_legal_captures(py, &self.board, &self.move_gen)
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
        Self::_is_fifty_moves(&self.board, self.halfmove_clock)
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
        Self::_is_seventy_five_moves(&self.board, self.halfmove_clock)
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
        Self::_is_insufficient_material(&self.board)
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
    /// >>> board.board_history.count(board.zobrist_hash)  # Position appears 4 times
    /// 4
    /// ```
    ///
    /// TODO: Quick check (only check last few moves since that is common error for engines)
    /// TODO: Add option to use full, or no repetition checks
    #[inline]
    fn is_n_repetition(&self, n: u8) -> bool {
        Self::_is_n_repetition(&self.board_history, self.halfmove_clock, n)
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
    /// >>> board.board_history.count(board.zobrist_hash)  # Position has appeared 3 times
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
    /// >>> board.board_history.count(board.zobrist_hash)  # Position has appeared 5 times
    /// 5
    /// ```
    #[inline]
    fn is_fivefold_repetition(&self) -> bool {
        self.is_n_repetition(5)
    }

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
        Self::_is_check(&self.board)
    }

    // TODO: Docs

    /// Checks if the side to move is in stalemate.
    ///
    /// ```python
    /// >>> rust_chess.Board().is_stalemate()
    /// False
    /// ```
    /// TODO
    #[inline]
    fn is_stalemate(&self) -> bool {
        Self::_is_stalemate(&self.board)
    }

    /// Checks if the side to move is in checkmate.
    ///
    /// ```python
    /// >>> rust_chess.Board().is_checkmate()
    /// False
    /// ```
    /// TODO
    #[inline]
    fn is_checkmate(&self) -> bool {
        Self::_is_checkmate(&self.board)
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
        Self::_get_status(&self.board, &self.board_history, self.halfmove_clock)
    }
}

// PyO3 does not support "self" input parameters, only "&self"
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::wrong_self_convention)]
#![allow(clippy::unused_self)]

use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, module_variable};

mod types;

use crate::types::{
    bitboard::{
        PyBitboard, BB_EMPTY, BB_FILES, BB_FILE_A, BB_FILE_B, BB_FILE_C, BB_FILE_D, BB_FILE_E,
        BB_FILE_F, BB_FILE_G, BB_FILE_H, BB_FULL, BB_RANKS, BB_RANK_1, BB_RANK_2, BB_RANK_3,
        BB_RANK_4, BB_RANK_5, BB_RANK_6, BB_RANK_7, BB_RANK_8,
    },
    board::{PyBoard, PyBoardStatus},
    color::{PyColor, BLACK, COLORS, WHITE},
    piece::{
        pieces::*, PyPiece, PyPieceType, BISHOP, KING, KNIGHT, PAWN, PIECE_TYPES, QUEEN, ROOK,
    },
    r#move::{PyMove, PyMoveGenerator},
    square::PySquare,
};

// TODO: Remove inline for Python-called only?
// TODO: Add PSQT table support?
// TODO: Add transposition key
// TODO: Add zobrist hashing
// TODO: Add eq and partial eq for common classes that can be used as int

// Define the Python module
#[pymodule]
fn rust_chess(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyColor>()?;
    module.add_class::<PyPieceType>()?;
    module.add_class::<PyPiece>()?;
    module.add_class::<PySquare>()?;
    module.add_class::<PyBitboard>()?;
    module.add_class::<PyMove>()?;
    module.add_class::<PyMoveGenerator>()?;
    module.add_class::<PyBoardStatus>()?;
    module.add_class::<PyBoard>()?;

    // Define a macro to add constants and their stubs
    macro_rules! add_constant {
        ($name:expr, $value:expr, $type:ty) => {
            module.add($name, $value)?;
            module_variable!("rust_chess", $name, $type);
        };
    }

    // Add the constants and stubs to the module

    // Add the color constants and their stubs
    add_constant!("WHITE", WHITE, PyColor);
    add_constant!("BLACK", BLACK, PyColor);

    add_constant!("COLORS", COLORS, Vec<PyColor>);

    // Add the piece type constants and their stubs
    add_constant!("PAWN", PAWN, PyPieceType);
    add_constant!("KNIGHT", KNIGHT, PyPieceType);
    add_constant!("BISHOP", BISHOP, PyPieceType);
    add_constant!("ROOK", ROOK, PyPieceType);
    add_constant!("QUEEN", QUEEN, PyPieceType);
    add_constant!("KING", KING, PyPieceType);

    add_constant!("PIECE_TYPES", PIECE_TYPES, Vec<PyPieceType>);

    // Add the piece constants and their stubs
    add_constant!("WHITE_PAWN", WHITE_PAWN, PyPiece);
    add_constant!("WHITE_KNIGHT", WHITE_KNIGHT, PyPiece);
    add_constant!("WHITE_BISHOP", WHITE_BISHOP, PyPiece);
    add_constant!("WHITE_ROOK", WHITE_ROOK, PyPiece);
    add_constant!("WHITE_QUEEN", WHITE_QUEEN, PyPiece);
    add_constant!("WHITE_KING", WHITE_KING, PyPiece);

    add_constant!("BLACK_PAWN", BLACK_PAWN, PyPiece);
    add_constant!("BLACK_KNIGHT", BLACK_KNIGHT, PyPiece);
    add_constant!("BLACK_BISHOP", BLACK_BISHOP, PyPiece);
    add_constant!("BLACK_ROOK", BLACK_ROOK, PyPiece);
    add_constant!("BLACK_QUEEN", BLACK_QUEEN, PyPiece);
    add_constant!("BLACK_KING", BLACK_KING, PyPiece);

    add_constant!("PIECES", PIECES, Vec<PyPiece>);

    add_constant!("BB_EMPTY", BB_EMPTY, PyBitboard);
    add_constant!("BB_FULL", BB_FULL, PyBitboard);

    add_constant!("BB_FILE_A", BB_FILE_A, PyBitboard);
    add_constant!("BB_FILE_B", BB_FILE_B, PyBitboard);
    add_constant!("BB_FILE_C", BB_FILE_C, PyBitboard);
    add_constant!("BB_FILE_D", BB_FILE_D, PyBitboard);
    add_constant!("BB_FILE_E", BB_FILE_E, PyBitboard);
    add_constant!("BB_FILE_F", BB_FILE_F, PyBitboard);
    add_constant!("BB_FILE_G", BB_FILE_G, PyBitboard);
    add_constant!("BB_FILE_H", BB_FILE_H, PyBitboard);

    add_constant!("BB_FILES", BB_FILES.to_vec(), Vec<PyBitboard>);

    add_constant!("BB_RANK_1", BB_RANK_1, PyBitboard);
    add_constant!("BB_RANK_2", BB_RANK_2, PyBitboard);
    add_constant!("BB_RANK_3", BB_RANK_3, PyBitboard);
    add_constant!("BB_RANK_4", BB_RANK_4, PyBitboard);
    add_constant!("BB_RANK_5", BB_RANK_5, PyBitboard);
    add_constant!("BB_RANK_6", BB_RANK_6, PyBitboard);
    add_constant!("BB_RANK_7", BB_RANK_7, PyBitboard);
    add_constant!("BB_RANK_8", BB_RANK_8, PyBitboard);

    add_constant!("BB_RANKS", BB_RANKS.to_vec(), Vec<PyBitboard>);

    // Define a macro to add square constants and stubs directly to the module (e.g. A1, A2, etc.)
    macro_rules! add_square_constants {
        ($module:expr, $($name:ident),*) => {
            $(
                $module.add(stringify!($name), PySquare(chess::Square::$name))?;
                module_variable!("rust_chess", stringify!($name), PySquare);
            )*
        }
    }

    // Add all square constants and stubs directly to the module
    #[rustfmt::skip]
    add_square_constants!(module,
        A1, A2, A3, A4, A5, A6, A7, A8,
        B1, B2, B3, B4, B5, B6, B7, B8,
        C1, C2, C3, C4, C5, C6, C7, C8,
        D1, D2, D3, D4, D5, D6, D7, D8,
        E1, E2, E3, E4, E5, E6, E7, E8,
        F1, F2, F3, F4, F5, F6, F7, F8,
        G1, G2, G3, G4, G5, G6, G7, G8,
        H1, H2, H3, H4, H5, H6, H7, H8
    );

    // Add list of square constants
    let squares: Vec<PySquare> = (0..64).map(|i| PySquare::from_index(i).unwrap()).collect();
    add_constant!("SQUARES", squares, Vec<PySquare>);

    Ok(())
}

// Define a function to gather stub information.
define_stub_info_gatherer!(stub_info);

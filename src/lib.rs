// PyO3 does not support "self" input parameters, only "&self"
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::wrong_self_convention)]
#![allow(clippy::unused_self)]

use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, module_variable};

mod types;

use crate::types::{
    bitboard::PyBitboard,
    board::{PyBoard, PyBoardStatus},
    color::{PyColor, BLACK, COLORS, WHITE},
    piece::{PyPiece, PyPieceType, BISHOP, KING, KNIGHT, PAWN, PIECES, QUEEN, ROOK},
    r#move::{PyMove, PyMoveGenerator},
    square::PySquare,
};

// TODO: Remove inline for Python-called only?

// Define the Python module
#[pymodule]
fn rust_chess(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyColor>()?;
    module.add_class::<PyPieceType>()?;
    module.add_class::<PyPiece>()?;
    module.add_class::<PyBitboard>()?;
    module.add_class::<PySquare>()?;
    module.add_class::<PyMove>()?;
    module.add_class::<PyMoveGenerator>()?;
    module.add_class::<PyBoardStatus>()?;
    module.add_class::<PyBoard>()?;

    // Add the constants and stubs to the module

    // Add the color constants and their stubs
    module.add("WHITE", WHITE)?;
    module_variable!("rust_chess", "WHITE", PyColor);
    module.add("BLACK", BLACK)?;
    module_variable!("rust_chess", "BLACK", PyColor);
    module.add("COLORS", COLORS)?;
    module_variable!("rust_chess", "COLORS", Vec<PyColor>);

    // Add the piece constants and their stubs
    module.add("PAWN", PAWN)?;
    module_variable!("rust_chess", "PAWN", PyPieceType);
    module.add("KNIGHT", KNIGHT)?;
    module_variable!("rust_chess", "KNIGHT", PyPieceType);
    module.add("BISHOP", BISHOP)?;
    module_variable!("rust_chess", "BISHOP", PyPieceType);
    module.add("ROOK", ROOK)?;
    module_variable!("rust_chess", "ROOK", PyPieceType);
    module.add("QUEEN", QUEEN)?;
    module_variable!("rust_chess", "QUEEN", PyPieceType);
    module.add("KING", KING)?;
    module_variable!("rust_chess", "KING", PyPieceType);
    module.add("PIECES", PIECES)?;
    module_variable!("rust_chess", "PIECES", Vec<PyPieceType>);

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

    Ok(())
}

// Define a function to gather stub information.
define_stub_info_gatherer!(stub_info);

// use std::str::FromStr;

// use pyo3::{basic::CompareOp, exceptions::PyValueError, prelude::*, types::PyAny};
// use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

// pub(crate) const FILE_A: PyFile = PyFile(chess::File::A);
// pub(crate) const FILE_B: PyFile = PyFile(chess::File::B);
// pub(crate) const FILE_C: PyFile = PyFile(chess::File::C);
// pub(crate) const FILE_D: PyFile = PyFile(chess::File::D);
// pub(crate) const FILE_E: PyFile = PyFile(chess::File::E);
// pub(crate) const FILE_F: PyFile = PyFile(chess::File::F);
// pub(crate) const FILE_G: PyFile = PyFile(chess::File::G);
// pub(crate) const FILE_H: PyFile = PyFile(chess::File::H);

// pub(crate) const FILES: [PyFile; 8] = [
//     FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H,
// ];

// #[gen_stub_pyclass]
// #[pyclass(name = "File", frozen, eq, ord)]
// #[derive(PartialEq, Eq, PartialOrd, Copy, Clone, Hash)]
// pub(crate) struct PyFile(pub(crate) chess::File);

// #[gen_stub_pymethods]
// #[pymethods]
// impl PyFile {
//     #[inline]
//     fn from_index(&self) -> String {
//         self
//     }
// }

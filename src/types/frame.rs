use crate::sealed::Sealed;
#[cfg(not(PyRustPython))]
use crate::types::{PyCode, PyDict};
use crate::PyAny;
#[cfg(not(PyRustPython))]
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::{ffi, Bound, PyResult, Python};
#[cfg(PyRustPython)]
use crate::PyErr;
#[cfg(PyRustPython)]
use crate::types::{PyAnyMethods, PyCode, PyCodeInput, PyCodeMethods, PyDict};
#[cfg(PyRustPython)]
use crate::{sync::PyOnceLock, types::{PyType, PyTypeMethods}, Py};
#[cfg(not(PyRustPython))]
use pyo3_ffi::PyObject;
use std::ffi::CStr;

/// Represents a Python frame.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyFrame>`][crate::Py] or [`Bound<'py, PyFrame>`][crate::Bound].
#[repr(transparent)]
pub struct PyFrame(PyAny);

#[cfg(not(PyRustPython))]
pyobject_native_type_core!(
    PyFrame,
    pyobject_native_static_type_object!(ffi::PyFrame_Type),
    "types",
    "FrameType",
    #checkfunction=ffi::PyFrame_Check
);

#[cfg(PyRustPython)]
pyobject_native_type_core!(
    PyFrame,
    |py| {
        static TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
        TYPE.import(py, "types", "FrameType").unwrap().as_type_ptr()
    },
    "types",
    "FrameType"
);

impl PyFrame {
    /// Creates a new frame object.
    pub fn new<'py>(
        py: Python<'py>,
        file_name: &CStr,
        func_name: &CStr,
        line_number: i32,
    ) -> PyResult<Bound<'py, PyFrame>> {
        #[cfg(PyRustPython)]
        {
            let mut source = String::new();
            for _ in 1..line_number.max(2) - 1 {
                source.push('\n');
            }
            source.push_str("def ");
            source.push_str(func_name.to_str().unwrap());
            source.push_str("():\n    raise RuntimeError()\n");
            source.push_str(func_name.to_str().unwrap());
            source.push_str("()\n");
            let source = std::ffi::CString::new(source).unwrap();
            let code = PyCode::compile(py, source.as_c_str(), file_name, PyCodeInput::File)?;
            let globals = PyDict::new(py);
            match code.run(Some(&globals), Some(&globals)) {
                Err(err) => {
                    let mut tb = err.traceback(py).ok_or_else(|| {
                        PyErr::new::<crate::exceptions::PyRuntimeError, _>(
                            "RustPython failed to produce a traceback for PyFrame::new",
                        )
                    })?;
                    loop {
                        let next = tb.getattr("tb_next")?;
                        if next.is_none() {
                            break;
                        }
                        tb = next.cast_into()?;
                    }
                    return Ok(tb.getattr("tb_frame")?.cast_into()?);
                }
                Ok(_) => {
                    return Err(PyErr::new::<crate::exceptions::PyRuntimeError, _>(
                        "RustPython frame construction unexpectedly succeeded without traceback",
                    ));
                }
            }
        }

        #[cfg(not(PyRustPython))]
        // Safety: Thread is attached because we have a python token
        let state = unsafe { ffi::compat::PyThreadState_GetUnchecked() };
        #[cfg(not(PyRustPython))]
        let code = PyCode::empty(py, file_name, func_name, line_number);
        #[cfg(not(PyRustPython))]
        let globals = PyDict::new(py);
        #[cfg(not(PyRustPython))]
        let locals = PyDict::new(py);

        #[cfg(not(PyRustPython))]
        unsafe {
            Ok(ffi::PyFrame_New(
                state,
                code.into_ptr().cast(),
                globals.as_ptr(),
                locals.as_ptr(),
            )
            .cast::<PyObject>()
            .assume_owned_or_err(py)?
            .cast_into_unchecked::<PyFrame>())
        }
    }
}

/// Implementation of functionality for [`PyFrame`].
///
/// These methods are defined for the `Bound<'py, PyFrame>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyFrame")]
pub trait PyFrameMethods<'py>: Sealed {
    /// Returns the line number of the current instruction in the frame.
    fn line_number(&self) -> i32;
}

impl<'py> PyFrameMethods<'py> for Bound<'py, PyFrame> {
    fn line_number(&self) -> i32 {
        unsafe { ffi::PyFrame_GetLineNumber(self.as_ptr().cast()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_creation() {
        Python::attach(|py| {
            let frame = PyFrame::new(py, c"file.py", c"func", 42).unwrap();
            assert_eq!(frame.line_number(), 42);
        });
    }
}

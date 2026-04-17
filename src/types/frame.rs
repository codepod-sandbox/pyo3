use crate::sealed::Sealed;
use crate::PyAny;
use crate::{ffi, Bound, PyResult, Python};
use std::ffi::CStr;

/// Represents a Python frame.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyFrame>`][crate::Py] or [`Bound<'py, PyFrame>`][crate::Bound].
#[repr(transparent)]
pub struct PyFrame(PyAny);

pyobject_native_type_core!(
    PyFrame,
    |py| crate::backend::current::types::frame_type_object(py),
    "types",
    "FrameType",
    #checkfunction=crate::backend::current::types::frame_check
);

impl PyFrame {
    /// Creates a new frame object.
    pub fn new<'py>(
        py: Python<'py>,
        file_name: &CStr,
        func_name: &CStr,
        line_number: i32,
    ) -> PyResult<Bound<'py, PyFrame>> {
        crate::backend::current::types::new_frame(py, file_name, func_name, line_number)
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

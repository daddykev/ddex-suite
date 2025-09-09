use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};
use pythonize::{pythonize, depythonize};
use ddex_builder::{DdexBuilder as RustBuilder, BuildRequest, BuildOptions};

#[pyclass]
struct DDEXBuilder {
    inner: RustBuilder,
}

#[pymethods]
impl DDEXBuilder {
    #[new]
    fn new() -> PyResult<Self> {
        Ok(DDEXBuilder {
            inner: RustBuilder::new(),
        })
    }

    fn build(&self, request: &PyDict, options: Option<&PyDict>, py: Python) -> PyResult<PyObject> {
        let req: BuildRequest = depythonize(request)?;
        let opts: BuildOptions = if let Some(opt_dict) = options {
            depythonize(opt_dict)?
        } else {
            BuildOptions::default()
        };

        let result = py.allow_threads(|| {
            self.inner.build_sync(req, opts)
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        pythonize(py, &result).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
        })
    }

    fn preflight(&self, request: &PyDict, py: Python) -> PyResult<PyObject> {
        let req: BuildRequest = depythonize(request)?;

        let result = py.allow_threads(|| {
            self.inner.preflight_sync(req)
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        pythonize(py, &result).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
        })
    }

    fn canonicalize(&self, xml: &str) -> PyResult<String> {
        self.inner.canonicalize(xml)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    fn from_dataframe(&self, df: &PyAny, profile: &str, py: Python) -> PyResult<PyObject> {
        // Convert pandas DataFrame to BuildRequest
        // This will need custom implementation based on DataFrame structure
        todo!("DataFrame conversion implementation")
    }
}

#[pymodule]
fn ddex_builder(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<DDEXBuilder>()?;
    Ok(())
}
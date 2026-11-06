//! Thin synchronous native boundary; all template logic lives in the Rust engine.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use wordcraft_engine::template;

pyo3::create_exception!(foliorivet, TemplateError, PyException);

fn failure(e: impl std::fmt::Display) -> PyErr {
    TemplateError::new_err(e.to_string())
}

#[pyfunction]
fn render_template(template_path: &str, context_json: &str, assets_dir: &str, output_path: &str) -> PyResult<String> {
    let context = template::parse_context(context_json).map_err(failure)?;
    template::render_template(template_path, &context, assets_dir, output_path).map_err(failure)
}

#[pyfunction]
fn inspect_template(template_path: &str) -> PyResult<String> {
    let doc = template::load(template_path).map_err(failure)?;
    let result = template::inspect(&doc).map_err(failure)?;
    serde_json::to_string(&result).map_err(failure)
}

#[pyfunction]
#[pyo3(signature = (template_path, context_json, assets_dir=""))]
fn preflight_template(template_path: &str, context_json: &str, assets_dir: &str) -> PyResult<String> {
    let context = template::parse_context(context_json).map_err(failure)?;
    let doc = template::load(template_path).map_err(failure)?;
    let result = template::preflight_with_assets(&doc, &context, assets_dir).map_err(failure)?;
    serde_json::to_string(&result).map_err(failure)
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("TemplateError", m.py().get_type::<TemplateError>())?;
    m.add_function(wrap_pyfunction!(render_template, m)?)?;
    m.add_function(wrap_pyfunction!(inspect_template, m)?)?;
    m.add_function(wrap_pyfunction!(preflight_template, m)?)?;
    Ok(())
}

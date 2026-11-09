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
#[pyo3(signature = (template_path, context_json, assets_dir, output_path, *, scalar_fanout=false))]
fn render_template(template_path: &str, context_json: &str, assets_dir: &str, output_path: &str, scalar_fanout: bool) -> PyResult<String> {
    let context = template::parse_context(context_json).map_err(failure)?;
    template::render_template_with_options(template_path, &context, assets_dir, output_path, template::TemplateOptions { scalar_fanout })
        .map_err(failure)
}

#[pyfunction]
#[pyo3(signature = (template_path, *, scalar_fanout=false))]
fn inspect_template(template_path: &str, scalar_fanout: bool) -> PyResult<String> {
    let doc = template::load(template_path).map_err(failure)?;
    let result = template::inspect_with_options(&doc, template::TemplateOptions { scalar_fanout }).map_err(failure)?;
    serde_json::to_string(&result).map_err(failure)
}

#[pyfunction]
#[pyo3(signature = (template_path, context_json, assets_dir="", *, scalar_fanout=false))]
fn preflight_template(template_path: &str, context_json: &str, assets_dir: &str, scalar_fanout: bool) -> PyResult<String> {
    let context = template::parse_context(context_json).map_err(failure)?;
    let doc = template::load(template_path).map_err(failure)?;
    let result = template::preflight_with_options(&doc, &context, assets_dir, template::TemplateOptions { scalar_fanout }).map_err(failure)?;
    serde_json::to_string(&result).map_err(failure)
}

#[pyfunction]
#[pyo3(signature = (template_path, context_json, *, scalar_fanout=false))]
fn analyze_context(template_path: &str, context_json: &str, scalar_fanout: bool) -> PyResult<String> {
    let context = template::parse_context(context_json).map_err(failure)?;
    let doc = template::load(template_path).map_err(failure)?;
    let result = template::analyze_context(&doc, &context, template::TemplateOptions { scalar_fanout }).map_err(failure)?;
    serde_json::to_string(&result).map_err(failure)
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("TemplateError", m.py().get_type::<TemplateError>())?;
    m.add_function(wrap_pyfunction!(render_template, m)?)?;
    m.add_function(wrap_pyfunction!(inspect_template, m)?)?;
    m.add_function(wrap_pyfunction!(preflight_template, m)?)?;
    m.add_function(wrap_pyfunction!(analyze_context, m)?)?;
    Ok(())
}

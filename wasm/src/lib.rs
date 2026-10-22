//! Stateless JSON bindings. Browser storage is the caller's responsibility.
use cybercore::theme::{
    Appearance, PackConflictPolicy, ThemeCatalog, ThemeDocument, ThemePackDocument,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

fn error(value: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&value.to_string())
}

fn json(value: &impl Serialize) -> Result<String, JsValue> {
    serde_json::to_string(value).map_err(error)
}

fn document(input: &str) -> Result<ThemeDocument, JsValue> {
    let value = ThemeDocument::from_json(input, "imported-theme").map_err(error)?;
    value.validate().map_err(error)?;
    Ok(value)
}

fn catalog(custom_json: &str) -> Result<ThemeCatalog, JsValue> {
    let mut catalog = ThemeCatalog::builtin().map_err(error)?;
    let themes: Vec<ThemeDocument> = serde_json::from_str(custom_json).map_err(error)?;
    for theme in themes {
        catalog.insert_custom(theme).map_err(error)?;
    }
    Ok(catalog)
}

fn pack(input: &str) -> Result<ThemePackDocument, JsValue> {
    let pack: ThemePackDocument = serde_json::from_str(input).map_err(error)?;
    pack.validate().map_err(error)?;
    Ok(pack)
}

/// JSON array of all embedded ThemeDocuments.
#[wasm_bindgen]
pub fn builtin_themes() -> Result<String, JsValue> {
    let catalog = ThemeCatalog::builtin().map_err(error)?;
    json(
        &catalog
            .iter()
            .map(|(_, entry)| &entry.document)
            .collect::<Vec<_>>(),
    )
}

/// Validate a theme and return its quality report as JSON. Invalid input throws.
#[wasm_bindgen]
pub fn validate_theme(theme_json: &str) -> Result<String, JsValue> {
    json(&document(theme_json)?.quality_report())
}

/// Generate CSS; appearance must be "dark" or "light".
#[wasm_bindgen]
pub fn theme_css(theme_json: &str, appearance: &str) -> Result<String, JsValue> {
    let appearance: Appearance =
        serde_json::from_value(serde_json::Value::String(appearance.into())).map_err(error)?;
    Ok(document(theme_json)?.to_css(appearance))
}

/// Export a family from built-ins and the supplied custom ThemeDocument array.
#[wasm_bindgen]
pub fn export_family(custom_json: &str, family: &str) -> Result<String, JsValue> {
    let pack = catalog(custom_json)?
        .export_family(family)
        .ok_or_else(|| error("unknown theme family"))?;
    json(&pack)
}

/// Validate a pack and return its built-in/custom collision lists as JSON.
#[wasm_bindgen]
pub fn pack_conflicts(custom_json: &str, pack_json: &str) -> Result<String, JsValue> {
    json(&catalog(custom_json)?.pack_conflicts(&pack(pack_json)?))
}

/// Import with "reject", "skip", or "replace_custom". Return {report, themes},
/// where themes is the new custom-only collection to persist in browser storage.
#[wasm_bindgen]
pub fn import_pack(custom_json: &str, pack_json: &str, policy: &str) -> Result<String, JsValue> {
    let mut catalog = catalog(custom_json)?;
    let policy: PackConflictPolicy =
        serde_json::from_value(serde_json::Value::String(policy.into())).map_err(error)?;
    let report = catalog
        .import_pack_in_memory(pack(pack_json)?, policy)
        .map_err(error)?;
    let themes: Vec<_> = catalog
        .iter()
        .filter(|(_, entry)| !entry.builtin)
        .map(|(_, entry)| &entry.document)
        .collect();
    json(&serde_json::json!({ "report": report, "themes": themes }))
}

use axum::{
    body::Body,
    extract::{Path, Query},
    http::{header, Request, StatusCode},
    middleware::{self, Next},
    response::sse::{Event, KeepAlive, Sse},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use cybercore::control_plane::ControlPlaneWorkspace;
use cybercore::theme::{
    Appearance, PackConflictPolicy, ThemeCatalog, ThemeDocument, ThemeError, ThemePackDocument,
};
use futures_util::stream::{self, Stream};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{convert::Infallible, net::SocketAddr, time::Duration};

const PAGE: &str = include_str!("../static/index.html");

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/api/control-plane/workspace", get(control_plane_workspace))
        .route("/api/themes", get(list_themes).post(save_theme))
        .route("/api/events", get(theme_events))
        .route("/api/themes/:id", delete(delete_theme))
        .route("/api/themes/validate", post(validate_theme))
        .route("/api/packs", get(list_packs).post(import_pack))
        .route("/api/library", get(curated_library))
        .route("/api/library/:id/install", post(install_curated_pack))
        .route("/api/packs/export", get(export_pack))
        .route("/api/packs/validate", post(validate_pack))
        .route("/api/active/:id", post(select_theme))
        .route("/api/appearance/:mode", post(select_appearance))
        .route("/api/css/:id", get(theme_css))
        .layer(middleware::from_fn(local_host_only));

    let bind = std::env::var("CYBERCORE_THEME_STUDIO_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8761".to_string());
    let address: SocketAddr = bind.parse().expect("invalid CYBERCORE_THEME_STUDIO_BIND");
    if !address.ip().is_loopback() {
        eprintln!("Cybercore Theme Studio only accepts loopback bind addresses");
        std::process::exit(2);
    }
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("could not bind Cybercore Theme Studio");
    println!("Cybercore Theme Studio: http://{address}");
    axum::serve(listener, app).await.expect("server failed");
}

async fn local_host_only(request: Request<Body>, next: Next) -> Response {
    let allowed = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .map(is_local_host)
        .unwrap_or(false);
    if !allowed {
        return (StatusCode::FORBIDDEN, "local host required").into_response();
    }
    next.run(request).await
}

fn is_local_host(value: &str) -> bool {
    let Ok(authority) = value.parse::<axum::http::uri::Authority>() else {
        return false;
    };
    let host = authority.host().to_ascii_lowercase();
    matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1" | "[::1]")
        || host.ends_with(".localhost")
}

async fn index() -> Html<&'static str> {
    Html(PAGE)
}

async fn control_plane_workspace() -> Response {
    let path = std::env::var_os("CYBERCORE_WORKSPACE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("CYBERCORE_CONFIG_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| dirs_fallback_config_dir().join("cybercore"))
                .join("workspace.json")
        });
    match ControlPlaneWorkspace::load(&path) {
        Ok(workspace) => Json(json!({
            "loaded": true,
            "path": path,
            "workspace": workspace,
        }))
        .into_response(),
        Err(error) => Json(json!({
            "loaded": false,
            "path": path,
            "workspace": ControlPlaneWorkspace::new("cybercore-local", "Local Cybercore Workspace"),
            "error": error.to_string(),
        }))
        .into_response(),
    }
}

fn dirs_fallback_config_dir() -> std::path::PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".config"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from(".config"))
}

fn catalog_error(error: impl ToString) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": error.to_string()})),
    )
        .into_response()
}

fn pack_error(error: ThemeError) -> Response {
    let status = match &error {
        ThemeError::PackConflicts { .. } => StatusCode::CONFLICT,
        ThemeError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::BAD_REQUEST,
    };
    (status, Json(json!({"error": error.to_string()}))).into_response()
}

async fn list_themes() -> Response {
    let catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    let themes: Vec<Value> = catalog
        .iter()
        .map(|(id, entry)| {
            json!({
                "id": id,
                "builtin": entry.builtin,
                "document": entry.document,
            })
        })
        .collect();
    Json(json!({
        "active": catalog.active_id(),
        "appearance": catalog.active_appearance(),
        "themes": themes,
    }))
    .into_response()
}

async fn theme_events() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let events = stream::unfold(None, |last_revision| async move {
        loop {
            if let Ok(catalog) = ThemeCatalog::load() {
                let revision = catalog.revision();
                if last_revision != Some(revision) {
                    let event = Event::default()
                        .event("theme-change")
                        .data(revision.to_string());
                    return Some((Ok(event), Some(revision)));
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
    Sse::new(events).keep_alive(KeepAlive::default())
}

async fn list_packs() -> Response {
    let catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    let mut families = std::collections::BTreeMap::<String, usize>::new();
    for (_, entry) in catalog.iter() {
        *families
            .entry(entry.document.metadata.family.clone())
            .or_default() += 1;
    }
    Json(json!({
        "packs": families.into_iter().map(|(family, count)| json!({"family": family, "theme_count": count})).collect::<Vec<_>>()
    }))
    .into_response()
}

/// Curated packs are selected from Cybercore's reviewed embedded families.
/// They are copied into the user's catalog with namespaced IDs on install.
async fn curated_library() -> Response {
    let catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    let mut families = std::collections::BTreeMap::<String, usize>::new();
    for (_, entry) in catalog.iter().filter(|(_, entry)| entry.builtin) {
        *families
            .entry(entry.document.metadata.family.clone())
            .or_default() += 1;
    }
    Json(json!({"packs": families.into_iter().map(|(id, theme_count)| {
        let name = format!("{} collection", id.split('-').map(|word| {
            let mut chars = word.chars();
            chars.next().map(|first| first.to_uppercase().collect::<String>() + chars.as_str()).unwrap_or_default()
        }).collect::<Vec<_>>().join(" "));
        let themes: Vec<_> = catalog.iter()
            .filter(|(_, entry)| entry.builtin && entry.document.metadata.family == id)
            .map(|(_, entry)| entry.document.clone())
            .collect();
        let pack = namespaced_curated_pack(&id, themes);
        json!({
            "id": id,
            "name": name,
            "description": format!("Curated Cybercore themes from the {id} collection."),
            "theme_count": theme_count,
            "metadata": pack.metadata,
        })
    }).collect::<Vec<_>>() })).into_response()
}

async fn install_curated_pack(Path(id): Path<String>) -> Response {
    let mut catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    let themes: Vec<_> = catalog
        .iter()
        .filter(|(_, entry)| entry.builtin && entry.document.metadata.family == id)
        .map(|(_, entry)| entry.document.clone())
        .collect();
    if themes.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error":"curated pack not found"})),
        )
            .into_response();
    }
    let pack = namespaced_curated_pack(&id, themes);
    let count = pack.themes.len();
    match catalog.import_pack(pack, PackConflictPolicy::Skip) {
        Ok(report) => (StatusCode::CREATED, Json(json!({"family":format!("curated-{id}"),"imported":report.imported,"skipped":report.skipped,"theme_count":count}))).into_response(),
        Err(error) => pack_error(error),
    }
}

fn namespaced_curated_pack(id: &str, mut themes: Vec<ThemeDocument>) -> ThemePackDocument {
    for theme in &mut themes {
        let original_id = theme.metadata.id.clone();
        theme.metadata.id = format!("curated-{id}-{original_id}")
            .chars()
            .take(64)
            .collect();
        theme.metadata.family = format!("curated-{id}");
    }
    ThemePackDocument {
        format_version: cybercore::theme::THEME_PACK_FORMAT_VERSION,
        metadata: cybercore::theme::ThemePackMetadata {
            id: format!("curated-{id}"),
            name: format!("Curated {} Collection", id),
            description: format!("A user-installable copy of the reviewed {id} theme family."),
            author: "Cybercore".into(),
            version: "1.0.0".into(),
            license: "MIT".into(),
            homepage: "https://github.com/cybercoretech/cybercore".into(),
            compatibility: "0.8+".into(),
            preview: themes
                .first()
                .map(|theme| {
                    let palette = &theme.palette;
                    [
                        &palette.bg,
                        &palette.acid_green,
                        &palette.hot_pink,
                        &palette.cyan,
                        &palette.purple,
                    ]
                    .into_iter()
                    .map(|color| format!("#{}", color.trim_start_matches('#')))
                    .collect()
                })
                .unwrap_or_default(),
        },
        themes,
    }
}

#[derive(Deserialize)]
struct ExportPackQuery {
    family: String,
}

async fn export_pack(Query(query): Query<ExportPackQuery>) -> Response {
    let catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    match catalog.export_family(&query.family) {
        Some(pack) => Json(pack).into_response(),
        None => (StatusCode::NOT_FOUND, "theme pack not found").into_response(),
    }
}

async fn validate_pack(Json(pack): Json<ThemePackDocument>) -> Response {
    if let Err(error) = pack.validate() {
        return pack_error(error);
    }
    let catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    Json(json!({
        "valid": true,
        "theme_count": pack.themes.len(),
        "metadata": pack.metadata,
        "conflicts": catalog.pack_conflicts(&pack),
    }))
    .into_response()
}

#[derive(Deserialize)]
struct ImportPackRequest {
    pack: ThemePackDocument,
    #[serde(default)]
    policy: PackConflictPolicy,
}

async fn import_pack(Json(request): Json<ImportPackRequest>) -> Response {
    let mut catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    let pack_id = request.pack.metadata.id.clone();
    let pack_name = request.pack.metadata.name.clone();
    match catalog.import_pack(request.pack, request.policy) {
        Ok(report) => (
            StatusCode::CREATED,
            Json(json!({"id": pack_id, "name": pack_name, "imported": report.imported, "skipped": report.skipped})),
        )
            .into_response(),
        Err(error) => pack_error(error),
    }
}

async fn save_theme(Json(document): Json<ThemeDocument>) -> Response {
    let id = document.metadata.id.clone();
    let name = document.metadata.name.clone();
    let mut catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    if let Err(error) = catalog.save(document) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": error.to_string()})),
        )
            .into_response();
    }
    if let Err(error) = catalog.select(&id) {
        return catalog_error(error);
    }
    (StatusCode::CREATED, Json(json!({"id": id, "name": name}))).into_response()
}

async fn validate_theme(Json(document): Json<ThemeDocument>) -> Response {
    if let Err(error) = document.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": error.to_string()})),
        )
            .into_response();
    }
    let light = document
        .variants
        .contains_key(&Appearance::Light)
        .then(|| document.contrast_report(Appearance::Light));
    Json(json!({
        "valid": true,
        "contrast": document.contrast_report(Appearance::Dark),
        "light_contrast": light,
        "quality": document.quality_report(),
    }))
    .into_response()
}

async fn select_theme(Path(id): Path<String>) -> Response {
    let mut catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    match catalog.select(&id) {
        Ok(()) => Json(json!({"active": id})).into_response(),
        Err(error) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": error.to_string()})),
        )
            .into_response(),
    }
}

async fn select_appearance(Path(mode): Path<String>) -> Response {
    let appearance = match mode.as_str() {
        "dark" => Appearance::Dark,
        "light" => Appearance::Light,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "mode must be dark or light"})),
            )
                .into_response()
        }
    };
    let mut catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    match catalog.set_appearance(appearance) {
        Ok(()) => Json(json!({"appearance": appearance})).into_response(),
        Err(error) => catalog_error(error),
    }
}

async fn delete_theme(Path(id): Path<String>) -> Response {
    let mut catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    match catalog.remove_custom(&id) {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": error.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct CssQuery {
    appearance: Option<Appearance>,
}

async fn theme_css(Path(id): Path<String>, Query(query): Query<CssQuery>) -> Response {
    let catalog = match ThemeCatalog::load() {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    let Some(entry) = catalog.get(&id) else {
        return (StatusCode::NOT_FOUND, "unknown theme").into_response();
    };
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        entry
            .document
            .to_css(query.appearance.unwrap_or(catalog.active_appearance())),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::{dirs_fallback_config_dir, is_local_host, namespaced_curated_pack, ThemeDocument};

    #[test]
    fn permits_loopback_hostnames_and_ports() {
        for host in [
            "localhost",
            "127.0.0.1:8761",
            "[::1]:8761",
            "cybercore-tech.localhost:8762",
        ] {
            assert!(is_local_host(host), "expected {host} to be local");
        }
    }

    #[test]
    fn rejects_non_loopback_and_suffix_spoofed_hosts() {
        for host in ["cybercore-tech.localhost.example.com", "example.com:8761"] {
            assert!(!is_local_host(host), "expected {host} to be rejected");
        }
    }

    #[test]
    fn curated_pack_copies_embedded_themes_with_valid_isolated_ids() {
        let schema = cybercore::schema::load();
        let themes = schema
            .themes
            .iter()
            .filter(|(id, _)| {
                schema
                    .theme_family(id)
                    .is_some_and(|family| family == "cyberpunk")
            })
            .map(|(id, palette)| {
                let mut document = ThemeDocument::new(id, id, palette.clone());
                document.metadata.family = "cyberpunk".into();
                document
            })
            .collect();
        let pack = namespaced_curated_pack("cyberpunk", themes);
        pack.validate().unwrap();
        assert_eq!(pack.metadata.id, "curated-cyberpunk");
        assert_eq!(pack.metadata.license, "MIT");
        assert_eq!(pack.metadata.compatibility, "0.8+");
        assert_eq!(pack.metadata.preview.len(), 5);
        assert!(!pack.themes.is_empty());
        assert!(pack.themes.iter().all(|theme| {
            theme.metadata.id.starts_with("curated-cyberpunk-")
                && theme.metadata.family == "curated-cyberpunk"
        }));
    }

    #[test]
    fn config_fallback_is_relative_to_a_user_config_root() {
        let path = dirs_fallback_config_dir();
        assert!(path.ends_with(".config") || path.to_string_lossy().contains("config"));
    }
}

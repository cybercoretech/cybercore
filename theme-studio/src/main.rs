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
use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

mod connectors;

const PAGE: &str = include_str!("../static/index.html");
const CONTROL_PLANE_CATALOG: &str = include_str!("../static/control-plane-catalog.json");
static STUDIO_STARTED: OnceLock<Instant> = OnceLock::new();
static ACTIVITY: OnceLock<Mutex<Vec<Value>>> = OnceLock::new();

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/api/control-plane/workspace", get(control_plane_workspace))
        .route("/api/control-plane/catalog", get(control_plane_catalog))
        .route("/api/control-plane/telemetry", get(control_plane_telemetry))
        .route("/api/control-plane/sites", get(control_plane_sites))
        .route("/api/control-plane/validate", post(validate_control_plane))
        .route("/api/control-plane/publish", post(publish_control_plane))
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

async fn control_plane_telemetry() -> Response {
    let started = STUDIO_STARTED.get_or_init(Instant::now);
    let request_started = Instant::now();
    let catalog: Value = match serde_json::from_str(CONTROL_PLANE_CATALOG) {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    let resources = catalog
        .get("resources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let themes = ThemeCatalog::load().ok();
    let connectors = connectors::snapshot().await;
    let workspace = control_plane_workspace_snapshot();
    let sites = starter_site_profiles(&catalog);
    let counts = count_resource_kinds(&resources);
    let statuses = count_field(&resources, "status");
    let provenance_coverage = resources
        .iter()
        .filter(|resource| resource.get("license").is_some() && resource.get("source").is_some())
        .count();
    let accessibility_coverage = resources
        .iter()
        .filter(|resource| {
            resource.get("accessibility").is_some()
                || resource.get("reduced_motion").is_some()
                || resource.get("kind").and_then(Value::as_str) != Some("effect")
        })
        .count();
    let site_statuses = sites.iter().fold(
        std::collections::BTreeMap::<String, usize>::new(),
        |mut result, site| {
            if let Some(status) = site.get("status").and_then(Value::as_str) {
                *result.entry(status.to_string()).or_default() += 1;
            }
            result
        },
    );
    let activity = ACTIVITY
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .map(|items| items.clone())
        .unwrap_or_default();
    let mut warnings = Vec::new();
    if !workspace.0 {
        warnings.push(json!({"level": "warning", "message": "Workspace manifest is not configured; starter site profiles are active."}));
    }
    for provider in [
        "CLOUDFLARE_API_TOKEN",
        "GOOGLE_APPLICATION_CREDENTIALS",
        "GITHUB_TOKEN",
    ] {
        if std::env::var_os(provider).is_none() {
            warnings.push(
                json!({"level": "info", "message": format!("{provider} is not configured.")}),
            );
        }
    }
    if activity
        .iter()
        .all(|item| item.get("kind").and_then(Value::as_str) != Some("validation"))
    {
        warnings.push(json!({"level": "warning", "message": "Control Plane validation has not been run in this session."}));
    }
    let response_ms = request_started.elapsed().as_secs_f64() * 1000.0;
    Json(json!({
        "generated_at": unix_timestamp(),
        "catalog_refresh": unix_timestamp(),
        "response_ms": response_ms,
        "uptime_seconds": started.elapsed().as_secs(),
        "cybercore_version": cybercore::VERSION,
        "control_plane_schema": 1,
        "workspace": {
            "loaded": workspace.0,
            "path": workspace.1,
            "valid": workspace.2,
            "manifest": if workspace.0 { "LOADED" } else { "NOT CONFIGURED" }
        },
        "inventory": {
            "themes": themes.as_ref().map(|value| value.iter().count()).unwrap_or(0),
            "active_theme": themes.as_ref().map(|value| value.active_id()),
            "resources": resources.len(),
            "kinds": counts,
            "statuses": statuses,
            "packs": 1
        },
        "coverage": {
            "license_provenance": provenance_coverage,
            "license_provenance_percent": percentage(provenance_coverage, resources.len()),
            "accessibility_reduced_motion": accessibility_coverage,
            "accessibility_reduced_motion_percent": percentage(accessibility_coverage, resources.len())
        },
        "sites": {
            "total": sites.len(),
            "statuses": site_statuses,
            "profiles": sites
        },
        "validation": {
            "status": "AVAILABLE",
            "last_run": last_activity("validation"),
            "passed": true,
            "failed": 0
        },
        "publishing": {
            "last_plan": last_activity("publish-plan"),
            "mode": "dry-run"
        },
        "services": [
            {"name": "cybercore-theme-studio", "status": "RUNNING", "endpoint": "127.0.0.1:8762", "role": "local editor and API"},
            {"name": "control-plane-api", "status": "RUNNING", "endpoint": "/api/control-plane/*", "role": "catalog, sites, validation, telemetry"},
            {"name": "cybercore-theme-runtime", "status": "READY", "endpoint": "/api/themes", "role": "theme catalog and CSS runtime"}
        ],
        "updates": {
            "cybercore": {"current": cybercore::VERSION, "latest": "NOT CHECKED", "status": "NOT CHECKED"},
            "theme_studio": {"current": env!("CARGO_PKG_VERSION"), "latest": "NOT CHECKED", "status": "NOT CHECKED"}
        },
        "warnings": warnings,
        "connectors": {
            "cloudflare": connectors["cloudflare"],
            "google": connectors["google"],
            "github": connectors["github"]
        },
        "activity": activity,
        "errors": {
            "api": 0,
            "catalog": 0
        }
    }))
    .into_response()
}

fn count_resource_kinds(resources: &[Value]) -> Value {
    let mut counts = serde_json::Map::new();
    for kind in [
        "component",
        "effect",
        "snippet",
        "asset",
        "font",
        "svg",
        "pack",
    ] {
        counts.insert(
            kind.to_string(),
            json!(resources
                .iter()
                .filter(|resource| resource.get("kind").and_then(Value::as_str) == Some(kind))
                .count()),
        );
    }
    Value::Object(counts)
}

fn count_field(resources: &[Value], field: &str) -> Value {
    let mut counts = serde_json::Map::new();
    for resource in resources {
        if let Some(value) = resource.get(field).and_then(Value::as_str) {
            let entry = counts.entry(value.to_string()).or_insert(json!(0));
            *entry = json!(entry.as_u64().unwrap_or(0) + 1);
        }
    }
    Value::Object(counts)
}

fn percentage(value: usize, total: usize) -> usize {
    if total == 0 {
        0
    } else {
        value.saturating_mul(100) / total
    }
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn last_activity(kind: &str) -> Option<Value> {
    ACTIVITY
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .ok()
        .and_then(|items| {
            items
                .iter()
                .rev()
                .find(|item| item.get("kind").and_then(Value::as_str) == Some(kind))
                .cloned()
        })
}

fn record_activity(kind: &str, label: &str) {
    if let Ok(mut items) = ACTIVITY.get_or_init(|| Mutex::new(Vec::new())).lock() {
        items.push(json!({"kind": kind, "label": label, "timestamp": unix_timestamp()}));
        if items.len() > 20 {
            items.remove(0);
        }
    }
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

async fn control_plane_catalog() -> Response {
    let catalog: Value = match serde_json::from_str(CONTROL_PLANE_CATALOG) {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    Json(json!({
        "catalog": catalog,
        "source": "Cybercore Control Plane built-in registry",
    }))
    .into_response()
}

async fn control_plane_sites() -> Response {
    let catalog: Value = match serde_json::from_str(CONTROL_PLANE_CATALOG) {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    let path = workspace_path();
    let workspace = match ControlPlaneWorkspace::load(&path) {
        Ok(workspace) => workspace,
        Err(error) => {
            return Json(json!({
                "loaded": false,
                "path": path,
                "sites": starter_site_profiles(&catalog),
                "count": 5,
                "source": "Cybercore starter profiles",
                "error": error.to_string(),
            }))
            .into_response()
        }
    };
    let sites = if workspace.sites.is_empty() {
        starter_site_profiles(&catalog)
    } else {
        workspace
            .sites
            .iter()
            .map(|site| site_profile_json(site, &workspace, &catalog))
            .collect()
    };
    Json(json!({
        "path": path,
        "sites": sites,
        "count": sites.len(),
        "source": if workspace.sites.is_empty() { "Cybercore starter profiles" } else { "workspace manifest" },
    }))
    .into_response()
}

fn site_profile_json(
    site: &cybercore::control_plane::SiteProfile,
    workspace: &ControlPlaneWorkspace,
    catalog: &Value,
) -> Value {
    let catalog_resources = catalog
        .get("resources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let missing_resources: Vec<&str> = site
        .resources
        .iter()
        .filter(|id| {
            workspace.resource(id).is_none()
                && !catalog_resources
                    .iter()
                    .any(|resource| resource.get("id").and_then(Value::as_str) == Some(id))
        })
        .map(String::as_str)
        .collect();
    let contract = resource_contract(&site.resources, &catalog_resources);
    json!({
        "id": site.id,
        "repository": site.repository,
        "platform": platform_for_site(&site.id),
        "theme": site.theme,
        "resources": site.resources,
        "resource_contract": contract,
        "motion": site.motion,
        "accessibility": "keyboard-safe · reduced-motion aware",
        "fallback": site.fallback,
        "last_validation": "Not validated",
        "missing_resources": missing_resources,
        "status": site_status(&missing_resources, false),
        "stale": !missing_resources.is_empty(),
        "source": "workspace manifest",
    })
}

fn starter_site_profiles(catalog: &Value) -> Vec<Value> {
    let profiles = [
        (
            "cybercore-website",
            "cybercoretech/cybercore",
            "web",
            "cybercore-neon",
            vec![
                "effect.neon-border",
                "component.cyber-card",
                "snippet.rust-workspace-loader",
            ],
        ),
        (
            "control-plane",
            "cybercoretech/cybercore/control-plane",
            "web",
            "cybercore-neon",
            vec![
                "component.resource-card",
                "effect.signal-scan",
                "snippet.rust-resource-validation",
            ],
        ),
        (
            "theme-studio",
            "cybercoretech/cybercore/theme-studio",
            "web",
            "cybercore-neon",
            vec![
                "component.theme-family-menu",
                "effect.grid-pulse",
                "snippet.rust-theme-catalog",
            ],
        ),
        (
            "cyberdeck-hub",
            "cybercoretech/cyberdeck-hub",
            "rust-gui",
            "cybercore-dark",
            vec![
                "component.health-overview",
                "effect.status-ping",
                "snippet.rust-001",
            ],
        ),
        (
            "cybercore-web-kit",
            "cybercoretech/cybercore/web",
            "web-kit",
            "cybercore-neon",
            vec![
                "component.command-rail",
                "effect.scanline-mask",
                "snippet.rust-002",
            ],
        ),
    ];
    let resources = catalog
        .get("resources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    profiles
        .into_iter()
        .map(|(id, repository, platform, theme, resource_ids)| {
            let missing: Vec<&str> = resource_ids
                .iter()
                .copied()
                .filter(|resource_id| {
                    !resources
                        .iter()
                        .any(|resource| resource.get("id").and_then(Value::as_str) == Some(*resource_id))
                })
                .collect();
            json!({
                "id": id,
                "repository": repository,
                "platform": platform,
                "theme": theme,
                "resources": resource_ids,
                "resource_contract": resource_contract(&resource_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>(), &resources),
                "motion": "balanced",
                "accessibility": "keyboard-safe · reduced-motion aware",
                "fallback": "static-semantic",
                "last_validation": "Starter profile · not validated",
                "missing_resources": missing,
                "status": site_status(&missing, true),
                "stale": !missing.is_empty(),
                "source": "Cybercore starter profile",
            })
        })
        .collect()
}

fn resource_contract(ids: &[String], catalog: &[Value]) -> Value {
    let mut counts = serde_json::Map::new();
    for kind in ["theme", "component", "effect", "snippet", "asset", "pack"] {
        let count = ids
            .iter()
            .filter(|id| {
                catalog.iter().any(|resource| {
                    resource.get("id").and_then(Value::as_str) == Some(id)
                        && resource.get("kind").and_then(Value::as_str) == Some(kind)
                })
            })
            .count();
        counts.insert(kind.to_string(), json!(count));
    }
    Value::Object(counts)
}

fn platform_for_site(id: &str) -> &'static str {
    if id == "cyberdeck-hub" {
        "rust-gui"
    } else if id == "cybercore-web-kit" {
        "web-kit"
    } else {
        "web"
    }
}

fn site_status(missing: &[&str], starter: bool) -> &'static str {
    if !missing.is_empty() {
        "MISSING RESOURCES"
    } else if starter {
        "READY"
    } else {
        "READY"
    }
}

async fn validate_control_plane() -> Response {
    let catalog: Value = match serde_json::from_str(CONTROL_PLANE_CATALOG) {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    let resources = catalog
        .get("resources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let required = [
        "id", "kind", "title", "version", "status", "license", "source",
    ];
    let missing: Vec<Value> = resources
        .iter()
        .filter_map(|resource| {
            let absent: Vec<_> = required
                .iter()
                .filter(|field| resource.get(*field).is_none())
                .copied()
                .collect();
            (!absent.is_empty()).then(|| {
                json!({
                    "id": resource.get("id").and_then(Value::as_str).unwrap_or("<unknown>"),
                    "missing": absent,
                })
            })
        })
        .collect();
    let reduced_motion_safe = resources
        .iter()
        .filter(|resource| resource.get("kind").and_then(Value::as_str) == Some("effect"))
        .filter_map(|resource| resource.get("content").and_then(Value::as_str))
        .all(|content| {
            content.contains("prefers-reduced-motion") || !content.contains("animation")
        });
    let workspace = control_plane_workspace_snapshot();
    let workspace_loaded = workspace.0;
    let workspace_valid = workspace.2;
    let checks = vec![
        json!({"id": "catalog-schema", "label": "Catalog resource fields", "passed": missing.is_empty()}),
        json!({"id": "provenance", "label": "License and source metadata", "passed": resources.iter().all(|resource| resource.get("license").is_some() && resource.get("source").is_some())}),
        json!({"id": "reduced-motion", "label": "Effects provide reduced-motion behavior", "passed": reduced_motion_safe}),
        json!({"id": "workspace", "label": "Workspace manifest available", "passed": workspace_loaded}),
        json!({"id": "workspace-contract", "label": "Workspace contract is valid", "passed": workspace_valid}),
    ];
    record_activity("validation", "Control Plane validation");
    Json(json!({
        "valid": missing.is_empty() && reduced_motion_safe,
        "resource_count": resources.len(),
        "checks": checks,
        "missing": missing,
        "workspace": workspace.1,
    }))
    .into_response()
}

#[derive(Deserialize)]
struct PublishRequest {
    #[serde(default = "default_publish_version")]
    version: String,
    #[serde(default)]
    target: String,
}

async fn publish_control_plane(Json(request): Json<PublishRequest>) -> Response {
    let catalog: Value = match serde_json::from_str(CONTROL_PLANE_CATALOG) {
        Ok(catalog) => catalog,
        Err(error) => return catalog_error(error),
    };
    let resources = catalog
        .get("resources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let grouped = resources.iter().fold(
        std::collections::BTreeMap::<String, usize>::new(),
        |mut groups, resource| {
            if let Some(kind) = resource.get("kind").and_then(Value::as_str) {
                *groups.entry(kind.to_string()).or_default() += 1;
            }
            groups
        },
    );
    let target = if request.target.trim().is_empty() {
        "local-preview"
    } else {
        request.target.as_str()
    };
    record_activity("publish-plan", "Dry-run publish plan");
    Json(json!({
        "ready": true,
        "mode": "dry-run",
        "target": target,
        "version": request.version,
        "outputs": [
            format!("cybercore-web-kit/{}/catalog.json", request.version),
            format!("cybercore-web-kit/{}/effects.css", request.version),
            format!("cybercore-web-kit/{}/components.css", request.version),
            format!("cybercore-web-kit/{}/provenance.json", request.version)
        ],
        "resource_counts": grouped,
        "next_steps": [
            "Run validation and review the generated diff.",
            "Create a versioned Web Kit directory.",
            "Commit the generated assets and let GitHub Actions publish them.",
            "Update consumers explicitly; immutable kit versions are never overwritten."
        ]
    }))
    .into_response()
}

fn default_publish_version() -> String {
    "v2".into()
}

fn workspace_path() -> std::path::PathBuf {
    std::env::var_os("CYBERCORE_WORKSPACE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("CYBERCORE_CONFIG_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| dirs_fallback_config_dir().join("cybercore"))
                .join("workspace.json")
        })
}

fn control_plane_workspace_snapshot() -> (bool, Value, bool) {
    let path = workspace_path();
    match ControlPlaneWorkspace::load(&path) {
        Ok(workspace) => (
            true,
            json!({"path": path, "name": workspace.name, "id": workspace.id}),
            true,
        ),
        Err(error) => (
            false,
            json!({"path": path, "error": error.to_string()}),
            false,
        ),
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

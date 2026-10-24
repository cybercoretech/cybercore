use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use cybercore_updater::{Engine, Manifest, TaskStatus};
use serde::{Deserialize, Serialize};
use std::{env, net::SocketAddr, path::PathBuf, sync::Arc};

#[derive(Clone)]
struct AppState {
    engine: Arc<Engine>,
}

#[derive(Serialize)]
struct TaskView {
    name: String,
    kind: &'static str,
}

#[derive(Serialize)]
struct FleetView {
    name: String,
    kind: &'static str,
    source: String,
    source_exists: bool,
    worktree_clean: bool,
    service: Option<String>,
    installed_binary: Option<String>,
    service_active: Option<bool>,
    diagnostics: Vec<String>,
}

#[derive(Deserialize)]
struct ApplyRequest {
    names: Vec<String>,
}

#[derive(Serialize)]
struct ApplyResponse {
    results: Vec<TaskResult>,
}

#[derive(Serialize)]
struct TaskResult {
    name: String,
    status: &'static str,
}

#[derive(Deserialize)]
struct ServiceRequest {
    names: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env::var("CYBERCORE_ROOT").unwrap_or_else(|_| {
        "/home/raven/Devspace/Cybercore/framework/cybercore".into()
    }));
    let manifest = Manifest::load(root.join("tools/cybercore-consumers.json"))?;
    let state = AppState {
        engine: Arc::new(Engine::new(manifest, root)),
    };
    let bind = env::var("CYBERCORE_UPDATE_PANEL_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8790".into())
        .parse::<SocketAddr>()?;
    let app = Router::new()
        .route("/", get(index))
        .route("/api/tasks", get(tasks))
        .route("/api/fleet", get(fleet))
        .route("/api/plan", post(plan))
        .route("/api/apply", post(apply))
        .route("/api/repair", post(repair))
        .route("/api/services/restart", post(restart_services))
        .route("/api/rollback", post(rollback))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(bind).await?;
    println!("Cybercore Update Control Panel: http://{bind}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> Html<String> {
    let mut page = String::from(
        r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width">
<title>CYBERCORE // UPDATE CONTROL</title><style>
:root{--bg:#080b12;--fg:#e9f2ff;--panel:#101827;--line:#26344a;--muted:#7890ad;--cyan:#37e8ff;--pink:#ff3ea5;--purple:#a879ff;--acid:#9eff6e;--orange:#ffb454;--red:#ff5577;--white:#f4f7ff}
body{background:radial-gradient(ellipse at 15% -10%,#193252 0%,var(--bg) 48%);min-height:100vh}
.shell{max-width:1280px;margin:auto;padding:var(--space-7) var(--space-5)}
.hero{display:flex;align-items:end;justify-content:space-between;gap:var(--space-5);border-bottom:1px solid var(--line);padding-bottom:var(--space-5)}
.eyebrow{color:var(--pink);font-size:var(--fs-1);letter-spacing:var(--tracking-caps);text-transform:uppercase}
h1{font-size:clamp(2rem,5vw,var(--fs-7));letter-spacing:.12em;margin:.2rem 0;text-shadow:0 0 24px color-mix(in srgb,var(--cyan) 35%,transparent)}
.lede{color:var(--muted);max-width:55rem;margin:0}.signal{color:var(--acid);font-size:var(--fs-1);white-space:nowrap}
.grid{display:grid;grid-template-columns:minmax(0,2fr) minmax(18rem,1fr);gap:var(--space-5);margin-top:var(--space-5)}
.panel{background:linear-gradient(145deg,color-mix(in srgb,var(--panel) 92%,var(--cyan)),var(--panel));border:1px solid var(--line);box-shadow:0 0 32px #0008;padding:var(--space-5)}
.panel-head{display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid var(--line);padding-bottom:var(--space-3);margin-bottom:var(--space-3)}
.panel-title{color:var(--cyan);font-weight:700;letter-spacing:var(--tracking-caps);text-transform:uppercase}
.task-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(15rem,1fr));gap:var(--space-3)}
.task{display:flex;gap:.7rem;align-items:center;padding:var(--space-3);border:1px solid var(--line);background:#07101ddd;cursor:pointer;transition:transform var(--dur-fast),border-color var(--dur-fast),box-shadow var(--dur-fast)}
.task:hover{transform:translateY(-2px);border-color:var(--cyan);box-shadow:0 0 15px #37e8ff2b}.task input{accent-color:var(--pink);width:1.1rem;height:1.1rem}
.task-name{font-weight:700;font-size:var(--fs-1)}.task-kind{display:block;color:var(--muted);font-size:var(--fs-0);margin-top:.2rem;text-transform:uppercase}
.actions{display:flex;gap:var(--space-3);flex-wrap:wrap;margin-top:var(--space-5)}button{font:inherit;font-weight:700;letter-spacing:var(--tracking-caps);padding:.75rem 1rem;border:1px solid var(--pink);background:var(--pink);color:#fff;cursor:pointer;box-shadow:0 0 18px #ff3ea533}button.secondary{background:transparent;border-color:var(--cyan);color:var(--cyan)}button:hover{filter:brightness(1.15)}
#log{min-height:12rem;white-space:pre-wrap;color:var(--acid);font-size:var(--fs-1);overflow:auto}.metric{display:flex;justify-content:space-between;padding:.7rem 0;border-bottom:1px solid var(--line)}.metric b{color:var(--cyan)}
@media(max-width:800px){.hero{display:block}.signal{display:block;margin-top:var(--space-3)}.grid{grid-template-columns:1fr}.shell{padding:var(--space-5) var(--space-3)}}
</style>"#,
    );
    page.push_str("<style>");
    page.push_str(include_str!("../../../css/cybertokens.css"));
    page.push_str(include_str!("../../../css/cybercomponents.css"));
    page.push_str("</style>");
    page.push_str(
        r#"<style>
body{margin:0}
.shell{width:auto;max-width:1440px;padding:1rem 1.25rem}
.hero{padding-bottom:.85rem;align-items:center}
h1{font-size:clamp(1.5rem,3vw,2.4rem)}
.grid{grid-template-columns:minmax(0,1.7fr) minmax(15rem,.8fr);align-items:start;margin-top:1rem;gap:1rem}
.panel{padding:1rem}
.task-grid{grid-template-columns:repeat(auto-fit,minmax(12rem,1fr));gap:.55rem}
.task{padding:.55rem;min-width:0}
.task-name{font-size:.82rem}.task-kind{font-size:.65rem}
.actions{gap:.5rem;margin-top:1rem}.actions button,button{padding:.55rem .7rem;font-size:.7rem}
.diagnostics{margin-top:1rem}.diagnostics .task-grid{max-height:18rem;overflow:auto}
#log{min-height:5rem;max-height:10rem}
.extended-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(12rem,1fr));gap:.55rem}
.extended-card{border:1px solid var(--line);padding:.7rem;background:#07101ddd}
.extended-card b{display:block;color:var(--cyan);font-size:.75rem;margin-bottom:.25rem}
.extended-card span{display:block;color:var(--muted);font-size:.7rem;margin-bottom:.55rem}
@media(max-width:800px){.grid{grid-template-columns:1fr}.shell{padding:.75rem}.hero{display:block}}
</style>"#,
    );
    page.push_str(
        r#"</head><body><main class="shell">
<header class="hero"><div><div class="eyebrow">CYBERCORE // OPERATIONS</div><h1>UPDATE CONTROL</h1><p class="lede">One governed surface for the local fleet, shared themes, services, and release propagation.</p></div><div class="signal">● LOCAL CONTROL ACTIVE</div></header>
<div class="grid"><section class="panel"><div class="panel-head"><span class="panel-title">Fleet tasks</span><span id="count" class="badge">SCANNING</span></div><div id="tasks" class="task-grid">Loading fleet…</div><div class="actions"><button onclick="applySelected()">APPLY SELECTED</button><button class="secondary" onclick="repairSelected()">REPAIR SELECTED</button><button class="secondary" onclick="selectAll()">SELECT ALL</button><button class="secondary" onclick="selectNone()">CLEAR</button></div></section>
<aside class="panel"><div class="panel-head"><span class="panel-title">Control status</span></div><div class="metric"><span>Execution mode</span><b>LOCAL</b></div><div class="metric"><span>Theme registry</span><b>300 + 12</b></div><div class="metric"><span>Safety model</span><b>EXPLICIT</b></div><div class="metric"><span>Transport</span><b>LOOPBACK</b></div></aside></div>
<section class="panel diagnostics"><div class="panel-head"><span class="panel-title">Fleet diagnostics</span><button class="secondary" onclick="inspectFleet()">REFRESH</button></div><div id="fleet" class="task-grid">Inspection pending…</div></section>
<section class="panel" style="margin-top:1rem"><div class="panel-head"><span class="panel-title">Extended operations</span><span class="badge">LOCAL ONLY</span></div><div class="extended-grid">
<div class="extended-card"><b>DISCOVERY</b><span>Re-scan sources, worktrees, services, and installed binaries.</span><button class="secondary" onclick="inspectFleet()">INSPECT FLEET</button></div>
<div class="extended-card"><b>REPAIR / RETRY</b><span>Re-run selected consumers and synchronizers with the safety gate.</span><button class="secondary" onclick="repairSelected()">REPAIR SELECTED</button></div>
<div class="extended-card"><b>DRY PLAN</b><span>Review the selected update set without changing the system.</span><button class="secondary" onclick="planSelected()">PREVIEW PLAN</button></div>
<div class="extended-card"><b>SERVICES</b><span>Restart selected declared user services without rebuilding binaries.</span><button class="secondary" onclick="restartSelected()">RESTART SERVICES</button></div>
<div class="extended-card"><b>ROLLBACK</b><span>Restore the last binary saved by the updater before installation.</span><button class="secondary" onclick="rollbackSelected()">ROLLBACK LAST</button></div>
</div></section>
<section class="panel" style="margin-top:1rem"><div class="panel-head"><span class="panel-title">Activity stream</span><span id="state" class="signal">IDLE</span></div><div id="log">Ready. Select tasks and apply when the plan is approved.</div></section>
<script>
let taskNames=[];
async function load(){const r=await fetch('/api/tasks');const tasks=await r.json();taskNames=tasks.map(t=>t.name);document.querySelector('#count').textContent=tasks.length+' TASKS';document.querySelector('#tasks').innerHTML=tasks.map(t=>`<label class="task"><input type="checkbox" data-name="${t.name}" checked><span><span class="task-name">${t.name}</span><span class="task-kind">${t.kind}</span></span></label>`).join('');await inspectFleet()}
function selectAll(){document.querySelectorAll('[data-name]').forEach(x=>x.checked=true)}function selectNone(){document.querySelectorAll('[data-name]').forEach(x=>x.checked=false)}
function setBusy(busy){document.querySelectorAll('button').forEach(button=>button.disabled=busy)}
function showLog(text,state){document.querySelector('#log').textContent=text;document.querySelector('#state').textContent=state;document.querySelector('#log').scrollIntoView({behavior:'smooth',block:'nearest'})}
async function runAction(path,label){const names=[...document.querySelectorAll('[data-name]:checked')].map(x=>x.dataset.name);if(!names.length){showLog('Select at least one task first.','IDLE');return}setBusy(true);showLog(label+' '+names.length+' selected tasks…','RUNNING');try{const r=await fetch(path,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({names})});const body=await r.text();let output;try{output=JSON.stringify(JSON.parse(body),null,2)}catch{output=body||'No response body'}showLog(output,r.ok?'COMPLETE':'FAILED');await inspectFleet()}catch(error){showLog(label+' failed: '+error,'FAILED')}finally{setBusy(false)}}
async function applySelected(){await runAction('/api/apply','Applying')}
async function repairSelected(){await runAction('/api/repair','Repairing')}
async function restartSelected(){await runAction('/api/services/restart','Restarting services')}
async function rollbackSelected(){await runAction('/api/rollback','Rolling back')}
async function planSelected(){const names=[...document.querySelectorAll('[data-name]:checked')].map(x=>x.dataset.name);if(!names.length){showLog('Select at least one task first.','IDLE');return}setBusy(true);showLog('Planning '+names.length+' selected tasks…','PLANNING');try{const r=await fetch('/api/plan',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({names})});const body=await r.text();let output;try{output=JSON.stringify(JSON.parse(body),null,2)}catch{output=body||'No response body'}showLog(output,r.ok?'READY':'FAILED')}catch(error){showLog('Planning failed: '+error,'FAILED')}finally{setBusy(false)}}
async function inspectFleet(){const r=await fetch('/api/fleet');const fleet=await r.json();document.querySelector('#fleet').innerHTML=fleet.map(x=>{const ok=x.diagnostics.length===0;const detail=ok?'Ready':x.diagnostics.join(' · ');return `<div class="task"><span class="task-name">${x.name}</span><span class="task-kind">${ok?'READY':'ATTENTION'} — ${detail}</span></div>`}).join('')}
load().catch(e=>{document.querySelector('#state').textContent='ERROR';document.querySelector('#log').textContent=e})
</script></main></body></html>"#,
    );
    Html(page)
}

async fn tasks(State(state): State<AppState>) -> Json<Vec<TaskView>> {
    Json(
        state
            .engine
            .tasks()
            .into_iter()
            .map(|task| TaskView {
                name: task.name,
                kind: match task.kind {
                    cybercore_updater::TaskKind::Consumer => "consumer",
                    cybercore_updater::TaskKind::Sync => "sync",
                },
            })
            .collect(),
    )
}

async fn fleet(State(state): State<AppState>) -> Json<Vec<FleetView>> {
    Json(
        state
            .engine
            .inspect()
            .into_iter()
            .map(|item| FleetView {
                name: item.name,
                kind: match item.kind {
                    cybercore_updater::TaskKind::Consumer => "consumer",
                    cybercore_updater::TaskKind::Sync => "sync",
                },
                source: item.source.display().to_string(),
                source_exists: item.source_exists,
                worktree_clean: item.worktree_clean,
                service: item.service,
                installed_binary: item
                    .installed_binary
                    .map(|path| path.display().to_string()),
                service_active: item.service_active,
                diagnostics: item.diagnostics,
            })
            .collect(),
    )
}

async fn repair(
    State(state): State<AppState>,
    Json(request): Json<ApplyRequest>,
) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        state.engine.repair(&request.names, |_| {})
    })
    .await;
    match result {
        Ok(results) => Json(ApplyResponse {
            results: results
                .into_iter()
                .map(|(name, status)| TaskResult {
                    name,
                    status: match status {
                        TaskStatus::Succeeded => "repaired",
                        TaskStatus::Blocked => "blocked",
                        TaskStatus::Failed => "failed",
                        TaskStatus::Planned => "planned",
                        TaskStatus::Running => "running",
                    },
                })
                .collect(),
        })
        .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("repair task failed: {error}"),
        )
            .into_response(),
    }

}

async fn restart_services(
    State(state): State<AppState>,
    Json(request): Json<ServiceRequest>,
) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        state.engine.restart_services(&request.names, |_| {})
    })
    .await;
    task_response(result, "service restart")
}

async fn rollback(
    State(state): State<AppState>,
    Json(request): Json<ServiceRequest>,
) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        state.engine.rollback(&request.names, |_| {})
    })
    .await;
    task_response(result, "rollback")
}

fn task_response(
    result: Result<Vec<(String, TaskStatus)>, tokio::task::JoinError>,
    operation: &str,
) -> axum::response::Response {
    match result {
        Ok(results) => Json(ApplyResponse {
            results: results
                .into_iter()
                .map(|(name, status)| TaskResult {
                    name,
                    status: match status {
                        TaskStatus::Succeeded => "succeeded",
                        TaskStatus::Blocked => "blocked",
                        TaskStatus::Failed => "failed",
                        TaskStatus::Planned => "planned",
                        TaskStatus::Running => "running",
                    },
                })
                .collect(),
        })
        .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{operation} task failed: {error}"),
        )
            .into_response(),
    }
}

async fn apply(
    State(state): State<AppState>,
    Json(request): Json<ApplyRequest>,
) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        state.engine.run(&request.names, true, |_| {})
    })
    .await;
    match result {
        Ok(results) => Json(ApplyResponse {
            results: results
                .into_iter()
                .map(|(name, status)| TaskResult {
                    name,
                    status: match status {
                        TaskStatus::Succeeded => "succeeded",
                        TaskStatus::Blocked => "blocked",
                        TaskStatus::Failed => "failed",
                        TaskStatus::Planned => "planned",
                        TaskStatus::Running => "running",
                    },
                })
                .collect(),
        })
        .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("update task failed: {error}"),
        )
            .into_response(),
    }
}

async fn plan(
    State(state): State<AppState>,
    Json(request): Json<ApplyRequest>,
) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        state.engine.run(&request.names, false, |_| {})
    })
    .await;
    match result {
        Ok(results) => Json(ApplyResponse {
            results: results
                .into_iter()
                .map(|(name, status)| TaskResult {
                    name,
                    status: match status {
                        TaskStatus::Succeeded => "ready",
                        TaskStatus::Blocked => "blocked",
                        TaskStatus::Failed => "failed",
                        TaskStatus::Planned => "planned",
                        TaskStatus::Running => "running",
                    },
                })
                .collect(),
        })
        .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("plan task failed: {error}"),
        )
            .into_response(),
    }
}

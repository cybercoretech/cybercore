use cybercore::control_plane::ControlPlaneWorkspace;
use std::{env, path::PathBuf};

fn main() {
    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("examples/control-plane-workspace/.cybercore/workspace.json")
        });
    let workspace = ControlPlaneWorkspace::load(&path)
        .unwrap_or_else(|error| panic!("invalid Control Plane workspace {}: {error}", path.display()));

    println!("workspace: {} ({})", workspace.name, workspace.id);
    println!("resources: {}", workspace.resources.len());
    println!("sites: {}", workspace.sites.len());
    for site in &workspace.sites {
        println!("  {} -> {}", site.id, site.repository);
    }
}

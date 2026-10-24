use serde::Deserialize;
use std::{
    fs,
    fmt,
    io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Json(serde_json::Error),
    Command { program: String, code: Option<i32> },
    MissingSource(PathBuf),
    DirtyWorktree(PathBuf),
    MissingBinary(PathBuf),
}

impl fmt::Display for Error {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(output, "I/O error: {error}"),
            Self::Json(error) => write!(output, "manifest error: {error}"),
            Self::Command { program, code } => {
                write!(output, "{program} failed with exit code {code:?}")
            }
            Self::MissingSource(path) => write!(output, "missing source: {}", path.display()),
            Self::DirtyWorktree(path) => write!(output, "dirty worktree: {}", path.display()),
            Self::MissingBinary(path) => write!(output, "missing binary: {}", path.display()),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub consumers: Vec<Consumer>,
    pub syncs: Vec<SyncTask>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Consumer {
    pub name: String,
    pub source: String,
    pub package: String,
    pub binary: String,
    pub install: String,
    #[serde(default)]
    pub service: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub allowed_dirty: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SyncTask {
    pub name: String,
    pub source: String,
    pub command: Vec<String>,
    #[serde(default)]
    pub allowed_dirty: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskKind {
    Consumer,
    Sync,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub name: String,
    pub kind: TaskKind,
}

#[derive(Debug, Clone)]
pub struct Inspection {
    pub name: String,
    pub kind: TaskKind,
    pub source: PathBuf,
    pub source_exists: bool,
    pub worktree_clean: bool,
    pub service: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Planned,
    Running,
    Succeeded,
    Blocked,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub task: String,
    pub status: TaskStatus,
    pub message: String,
}

#[derive(Clone)]
pub struct Engine {
    manifest: Manifest,
    root: PathBuf,
}

impl Manifest {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
    }
}

impl Engine {
    pub fn new(manifest: Manifest, root: impl Into<PathBuf>) -> Self {
        Self {
            manifest,
            root: root.into(),
        }
    }

    pub fn tasks(&self) -> Vec<Task> {
        self.manifest
            .consumers
            .iter()
            .map(|item| Task {
                name: item.name.clone(),
                kind: TaskKind::Consumer,
            })
            .chain(self.manifest.syncs.iter().map(|item| Task {
                name: item.name.clone(),
                kind: TaskKind::Sync,
            }))
            .collect()
    }

    pub fn inspect(&self) -> Vec<Inspection> {
        self.manifest
            .consumers
            .iter()
            .map(|item| {
                let source = self.resolve(&item.source);
                Inspection {
                    name: item.name.clone(),
                    kind: TaskKind::Consumer,
                    source_exists: source.is_dir(),
                    worktree_clean: source.is_dir()
                        && self.check_worktree(&source, &item.allowed_dirty).is_ok(),
                    source,
                    service: item.service.clone(),
                }
            })
            .chain(self.manifest.syncs.iter().map(|item| {
                let source = self.resolve(&item.source);
                Inspection {
                    name: item.name.clone(),
                    kind: TaskKind::Sync,
                    source_exists: source.is_dir(),
                    worktree_clean: source.is_dir()
                        && self.check_worktree(&source, &item.allowed_dirty).is_ok(),
                    source,
                    service: None,
                }
            }))
            .collect()
    }

    pub fn run(
        &self,
        names: &[String],
        apply: bool,
        mut emit: impl FnMut(Event),
    ) -> Vec<(String, TaskStatus)> {
        let mut results = Vec::new();
        for item in &self.manifest.consumers {
            if !names.is_empty() && !names.iter().any(|name| name == &item.name) {
                continue;
            }
            let result = self.run_consumer(item, apply, &mut emit);
            results.push((item.name.clone(), result));
        }
        for item in &self.manifest.syncs {
            if !names.is_empty() && !names.iter().any(|name| name == &item.name) {
                continue;
            }
            let result = self.run_sync(item, apply, &mut emit);
            results.push((item.name.clone(), result));
        }
        results
    }

    fn run_consumer(
        &self,
        item: &Consumer,
        apply: bool,
        emit: &mut impl FnMut(Event),
    ) -> TaskStatus {
        let source = self.resolve(&item.source);
        if !source.is_dir() {
            return self.finish(item.name.clone(), TaskStatus::Blocked, format!("missing source: {}", source.display()), emit);
        }
        if let Err(error) = self.check_worktree(&source, &item.allowed_dirty) {
            return self.finish(item.name.clone(), TaskStatus::Blocked, error.to_string(), emit);
        }
        if !apply {
            return self.finish(item.name.clone(), TaskStatus::Planned, "ready to build and install".into(), emit);
        }
        emit(Event {
            task: item.name.clone(),
            status: TaskStatus::Running,
            message: "building release binary".into(),
        });
        let manifest = source.join("Cargo.toml");
        if fs::read_to_string(&manifest)
            .map(|contents| contents.contains("cybercore"))
            .unwrap_or(false)
        {
            if let Err(error) = self.command(&source, "cargo", &["update".into(), "-p".into(), "cybercore".into()]) {
                return self.finish(item.name.clone(), TaskStatus::Failed, error.to_string(), emit);
            }
        }
        let mut args = vec![
            "build".to_string(),
            "--release".into(),
            "--locked".into(),
            "--package".into(),
            item.package.clone(),
        ];
        if !item.features.is_empty() {
            args.push("--features".into());
            args.push(item.features.join(","));
        }
        if let Err(error) = self.command(&source, "cargo", &args) {
            return self.finish(item.name.clone(), TaskStatus::Failed, error.to_string(), emit);
        }
        let target = self.target_directory(&source);
        let binary = target.join("release").join(&item.binary);
        if !binary.is_file() {
            return self.finish(item.name.clone(), TaskStatus::Failed, format!("missing binary: {}", binary.display()), emit);
        }
        let destination = expand_home(&item.install);
        if let Err(error) = install_binary(&binary, &destination) {
            return self.finish(item.name.clone(), TaskStatus::Failed, error.to_string(), emit);
        }
        if let Some(service) = &item.service {
            if let Err(error) = self.command(&source, "systemctl", &["--user".into(), "restart".into(), service.clone()]) {
                return self.finish(item.name.clone(), TaskStatus::Failed, error.to_string(), emit);
            }
        }
        self.finish(item.name.clone(), TaskStatus::Succeeded, format!("installed {}", destination.display()), emit)
    }

    fn run_sync(
        &self,
        item: &SyncTask,
        apply: bool,
        emit: &mut impl FnMut(Event),
    ) -> TaskStatus {
        let source = self.resolve(&item.source);
        if !source.is_dir() {
            return self.finish(item.name.clone(), TaskStatus::Blocked, format!("missing source: {}", source.display()), emit);
        }
        if let Err(error) = self.check_worktree(&source, &item.allowed_dirty) {
            return self.finish(item.name.clone(), TaskStatus::Blocked, error.to_string(), emit);
        }
        if !apply {
            return self.finish(item.name.clone(), TaskStatus::Planned, "ready to synchronize".into(), emit);
        }
        if item.command.is_empty() {
            return self.finish(item.name.clone(), TaskStatus::Failed, "empty sync command".into(), emit);
        }
        emit(Event {
            task: item.name.clone(),
            status: TaskStatus::Running,
            message: "synchronizing".into(),
        });
        let error = self
            .command(&source, &item.command[0], &item.command[1..].to_vec())
            .err();
        match error {
            Some(error) => self.finish(item.name.clone(), TaskStatus::Failed, error.to_string(), emit),
            None => self.finish(item.name.clone(), TaskStatus::Succeeded, "synchronized".into(), emit),
        }
    }

    fn finish(
        &self,
        task: String,
        status: TaskStatus,
        message: String,
        emit: &mut impl FnMut(Event),
    ) -> TaskStatus {
        emit(Event {
            task,
            status,
            message,
        });
        status
    }

    fn resolve(&self, raw: &str) -> PathBuf {
        let path = expand_home(raw);
        if path.is_absolute() {
            path
        } else {
            self.root.join(path)
        }
    }

    fn check_worktree(&self, source: &Path, allowed: &[String]) -> Result<(), Error> {
        let output = Command::new("git")
            .args(["-C", &source.to_string_lossy(), "status", "--porcelain", "--untracked-files=all"])
            .output()?;
        for line in String::from_utf8_lossy(&output.stdout).lines().filter(|line| !line.is_empty()) {
            let path = Path::new(line.get(3..).unwrap_or_default());
            if !allowed.iter().map(Path::new).any(|item| path == item || path.starts_with(item)) {
                return Err(Error::DirtyWorktree(source.to_path_buf()));
            }
        }
        Ok(())
    }

    fn target_directory(&self, source: &Path) -> PathBuf {
        let output = Command::new("cargo")
            .args(["metadata", "--manifest-path"])
            .arg(source.join("Cargo.toml"))
            .args(["--format-version", "1", "--no-deps"])
            .output()
            .ok();
        output
            .and_then(|output| serde_json::from_slice::<serde_json::Value>(&output.stdout).ok())
            .and_then(|data| data.get("target_directory")?.as_str().map(PathBuf::from))
            .unwrap_or_else(|| source.join("target"))
    }

    fn command(&self, cwd: &Path, program: &str, args: &[String]) -> Result<(), Error> {
        let status = Command::new(program)
            .args(args)
            .current_dir(cwd)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::Command {
                program: program.into(),
                code: status.code(),
            })
        }
    }
}

fn install_binary(source: &Path, destination: &Path) -> Result<(), Error> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = destination.with_extension("cybercore-update.tmp");
    fs::copy(source, &temporary)?;
    fs::set_permissions(&temporary, fs::metadata(source)?.permissions())?;
    fs::rename(&temporary, destination)?;
    Ok(())
}

fn expand_home(raw: &str) -> PathBuf {
    if let Some(rest) = raw.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(raw)
}

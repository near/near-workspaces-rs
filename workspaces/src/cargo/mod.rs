use anyhow::anyhow;
use async_process::Command;
use cargo_metadata::{Message, Metadata, MetadataCommand};
use std::env;
use std::fmt::Debug;
use std::path::{Component, Path, PathBuf};
use std::process::Stdio;
use tracing::debug;

fn cargo_bin() -> Command {
    match env::var_os("CARGO") {
        Some(cargo) => Command::new(cargo),
        None => Command::new("cargo"),
    }
}

/// Fetch current project's metadata (i.e. project invoking this method, not the one that we are
/// trying to compile).
fn root_cargo_metadata() -> anyhow::Result<Metadata> {
    MetadataCommand::new().exec().map_err(Into::into)
}

async fn build_cargo_project<P: AsRef<Path> + Debug>(
    project_path: P,
) -> anyhow::Result<Vec<Message>> {
    let metadata = root_cargo_metadata()?;
    let output = cargo_bin()
        .args([
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
            "--message-format=json",
            "--target-dir",
            metadata.target_directory.as_str(),
        ])
        .current_dir(&project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;
    debug!(
        target: "workspaces",
        "Building project '{:?}' resulted in status {:?}",
        &project_path, output.status
    );
    if output.status.success() {
        let reader = std::io::BufReader::new(output.stdout.as_slice());
        Ok(Message::parse_stream(reader).map(|m| m.unwrap()).collect())
    } else {
        Err(anyhow!(
            "Failed to build project '{:?}'.\n\
            Stderr:\n\
            {}\n\
            Stdout:\n\
            {}",
            project_path,
            String::from_utf8(output.stderr)?,
            String::from_utf8(output.stdout)?,
        ))
    }
}

/// Builds the cargo project located at `project_path` and returns the generated wasm file contents.
///
/// NOTE: This macro does not check whether the resulting wasm file is a valid smart
/// contract or not.
#[macro_export]
macro_rules! compile_contract {
    ($contract_path:expr) => {
        $crate::__ContractCompiler::__new_dont_call_manually(file!(), $contract_path).compile()
    };
}

#[doc(hidden)]
pub struct __ContractCompiler<P> {
    caller: &'static str,
    project_path: P,
}

impl<P: AsRef<Path>> __ContractCompiler<P> {
    // this is aptly named to induce friction, this structure should not be constructed trivially.
    pub fn __new_dont_call_manually(caller: &'static str, project_path: P) -> Self {
        Self {
            caller,
            project_path,
        }
    }

    pub async fn compile(self) -> anyhow::Result<Vec<u8>> {
        let contract_path = resolve_path(Path::new(self.caller), self.project_path)?;

        let messages = build_cargo_project(contract_path).await?;
        // We find the last compiler artifact message which should contain information about the
        // resulting .wasm file
        let compile_artifact = messages
            .iter()
            .filter_map(|m| match m {
                cargo_metadata::Message::CompilerArtifact(artifact) => Some(artifact),
                _ => None,
            })
            .last()
            .ok_or(anyhow!(
                "Cargo failed to produce any compilation artifacts. \
                     Please check that your project contains a NEAR smart contract."
            ))?;
        // The project could have generated many auxiliary files, we are only interested in .wasm files
        let wasm_files = compile_artifact
            .filenames
            .iter()
            .filter(|f| f.as_str().ends_with(".wasm"))
            .collect::<Vec<_>>();
        match wasm_files.as_slice() {
            [] => Err(anyhow!(
                "Compilation resulted in no '.wasm' target files. \
                     Please check that your project contains a NEAR smart contract."
            )),
            [file] => Ok(tokio::fs::read(file.canonicalize()?).await?),
            _ => Err(anyhow!(
                "Compilation resulted in more than one '.wasm' target file: {:?}",
                wasm_files
            )),
        }
    }
}

/// Resolves paths, relative or absolute, regardless of file existence.
///
/// Assuming this workspace tree for our cargo project structure:
///
/// ```txt
///  /workspace
///  ├── project
///  │   └── src
///  │       └── lib.rs
///  └── contract
///      └── src
///          └── lib.rs
/// ```
///
/// Then we can link to our contract from _`/workspace/project/src/lib.rs`_ using either of these methods:
///
/// - Relative paths:
///
///   ```no_run
///   let wasm = compile_contract!("../../contract");
///   ```
///
/// - Absolute paths:
///
///   ```no_run
///   let wasm = compile_contract!("/workspace/contract");
///   ```
fn resolve_path<P: AsRef<Path>>(caller: &Path, contract_path: P) -> anyhow::Result<PathBuf> {
    let contract_path = match contract_path.as_ref() {
        // if "/workspace/contract", return as-is
        contract_path if contract_path.is_absolute() => contract_path.to_path_buf(),
        contract_path => {
            // if "../../contract";
            let workspace = root_cargo_metadata()?;
            let mut dir = None;
            for pkg in workspace.packages {
                if !workspace.workspace_members.contains(&pkg.id) {
                    continue;
                }
                if let (Some(caller), Some(crate_path)) =
                    (caller.parent(), pkg.manifest_path.parent())
                {
                    // caller's parent: `project/src`
                    // crate_path: `/workspace/project`
                    if let Ok(this_path) = crate_path
                        .strip_prefix(&workspace.workspace_root)
                        .and_then(|crate_path| caller.strip_prefix(crate_path))
                    {
                        // crate_path (after stripping workspace root): `project`
                        // caller's parent after stripping crate_path: `src`
                        dir.replace(this_path);
                    }
                }
            }
            // joined: `src/../../contract` (as seen from /workspace/project)
            dir.ok_or_else(|| anyhow::anyhow!("expected a cargo directory structure"))?
                .join(contract_path)
        }
    };
    // normalized: `/workspace/contract`
    Ok(normalize_path(&contract_path))
}

/// Normalize a path, removing things like `.` and `..`.
///
/// Adapted from [`cargo-util`](https://github.com/rust-lang/cargo/blob/65c82664263feddc5fe2d424be0993c28d46377a/crates/cargo-util/src/paths.rs#L81).
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}

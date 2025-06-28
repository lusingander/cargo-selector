use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

use cargo_metadata::{Metadata as CargoMetadata, MetadataCommand, Target as CargoTarget};

use crate::{Action, Target, TargetKind};

fn convert(metadata: CargoMetadata, current_dir: &Path) -> Vec<Target> {
    let mut targets = Vec::new();
    for p in &metadata.packages {
        for t in &p.targets {
            if is_select_target(t) {
                targets.push(build_target(t, current_dir));
            }
        }
    }
    targets
}

fn is_select_target(t: &CargoTarget) -> bool {
    t.is_bin() || t.is_example()
}

fn build_target(t: &CargoTarget, current_dir: &Path) -> Target {
    let name = t.name.to_owned();
    let kind = if t.is_bin() {
        TargetKind::Bin
    } else {
        TargetKind::Example
    };
    let path = t
        .src_path
        .strip_prefix(current_dir)
        .map(|p| p.to_string())
        .unwrap_or("-".to_string());
    let required_features = t.required_features.clone();

    Target {
        name,
        kind,
        path,
        required_features,
    }
}

fn get_current_dir() -> PathBuf {
    env::current_dir().expect("failed to get current directory")
}

pub fn get_all_targets() -> Vec<Target> {
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .expect("failed to exec metadata command");
    let current_dir = get_current_dir();
    convert(metadata, &current_dir)
}

pub fn exec_cargo_run(
    target: &Target,
    action: &Action,
    additional_args: Option<String>,
) -> ExitStatus {
    let action = match action {
        Action::Run => "run",
        Action::Build => "build",
    };
    let kind = match target.kind {
        TargetKind::Bin => "--bin",
        TargetKind::Example => "--example",
    };
    let name = &target.name;

    let mut cmd = Command::new("cargo");
    cmd.arg(action).arg(kind).arg(name);

    let require_features = !target.required_features.is_empty();

    if require_features {
        let features = target.required_features.join(" ");
        cmd.arg("--features").arg(&features);
    };

    if let Some(args) = additional_args {
        // todo: handle quoted arguments properly
        args.split_whitespace().for_each(|a| {
            cmd.arg(a);
        });
    }

    eprintln!("{}", cmd_str(&cmd));

    cmd.spawn()
        .unwrap_or_else(|_| panic!("failed to spawn cargo {action} command"))
        .wait()
        .unwrap()
}

fn cmd_str(cmd: &Command) -> String {
    let program = cmd.get_program().to_string_lossy();
    let args = cmd
        .get_args()
        .map(|a| {
            let a = a.to_string_lossy();
            if a.contains(char::is_whitespace) {
                format!("\"{a}\"").into()
            } else {
                a
            }
        })
        .collect::<Vec<_>>();
    format!("{} {}", program, args.join(" "))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(
        vec!["cargo", "run", "--bin", "xyz"],
        "cargo run --bin xyz",
    )]
    #[case(
        vec!["cargo", "build", "--example", "xyz", "--features", "feature1 feature2", "--release"],
        "cargo build --example xyz --features \"feature1 feature2\" --release",
    )]
    #[case(
        vec!["cargo", "run", "--bin", "xyz", "--", "-p", "Hello World", "-n", "1"],
        "cargo run --bin xyz -- -p \"Hello World\" -n 1",
    )]
    fn test_cmd_str(#[case] args: Vec<&str>, #[case] expected: &str) {
        let mut cmd = Command::new(args[0]);
        args.iter().skip(1).for_each(|a| {
            cmd.arg(a);
        });
        assert_eq!(cmd_str(&cmd), expected);
    }
}

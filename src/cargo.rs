use std::{
    env,
    path::{Path, PathBuf},
};

use cargo_metadata::{Metadata as CargoMetadata, MetadataCommand, Target as CargoTarget};

use crate::{Target, TargetKind};

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

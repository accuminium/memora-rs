// Copyright 2020 Andreas Kurth
//
// SPDX-License-Identifier: (Apache-2.0 OR MIT)

//! Configuration

extern crate tuple_vec_map;
use crate::cache::{Artifact, Artifacts};
use crate::error::{Error, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// A Memora build artifact cache manifest.
#[derive(Debug)]
pub struct Manifest {
    /// The root directory of the build artifact cache for a Git repository.
    ///
    /// The path can be absolute or relative to the root of the Git repository.
    pub cache_root_dir: PathBuf,
    /// The Artifacts.
    ///
    /// Each Artifact must have a name.  This name is used as `artifact` argument to Memora
    /// subcommands, so it should be kept short.  The name of an Artifact must be unique among all
    /// Artifacts in a Memora manifest.
    ///
    /// See [Artifacts](../cache/type.Artifacts.html) for more details on Artifacts.
    pub artifacts: Artifacts,
    /// Optional name of an environment variable that, if set, disables the cache.
    pub disable_env_var: Option<String>,
}

#[derive(Deserialize)]
struct SerdeArtifact {
    inputs: Vec<PathBuf>,
    outputs: Vec<PathBuf>,
}

#[derive(Deserialize)]
struct SerdeManifest {
    pub cache_root_dir: PathBuf,
    #[serde(with = "tuple_vec_map")]
    pub artifacts: Vec<(String, SerdeArtifact)>,
    pub disable_env_var: Option<String>,
}

impl Manifest {
    /// Load a Manifest from the file at `path`.
    ///
    /// This function deserializes the Manifest file and adds the given `path` as input to each
    /// artifact.
    pub fn from_path(path: &Path) -> Result<Manifest> {
        use std::fs::File;
        let file = File::open(path)
            .map_err(|cause| Error::chain(format!("Cannot open manifest {:?}!", path), cause))?;
        let manifest = {
            let serde_manifest: SerdeManifest = serde_yaml::from_reader(file).map_err(|cause| {
                Error::chain(format!("Syntax error in manifest {:?}!", path), cause)
            })?;
            let mut manifest = Manifest {
                cache_root_dir: serde_manifest.cache_root_dir,
                artifacts: serde_manifest
                    .artifacts
                    .iter()
                    .map(|(name, serde_arti)| Artifact {
                        name: name.to_string(),
                        inputs: serde_arti.inputs.clone(),
                        outputs: serde_arti.outputs.clone(),
                    })
                    .collect(),
                disable_env_var: serde_manifest.disable_env_var,
            };
            // Add path of Manifest to inputs of each Artifact.
            for artifact in &mut manifest.artifacts {
                artifact.inputs.push(path.to_path_buf())
            }
            manifest
        };
        Ok(manifest)
    }
}

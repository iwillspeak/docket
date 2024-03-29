//! Non-Renderable Assets

//!
//! This module models the other files in a given docs directory which
//! need to be copied to the output. We use this to abstract between
//! 'bulitin' assets, such as the CSS which is bundled with Docket
//! itself and on-disk assets from the source directory.

use log::warn;

use super::Result;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

/// Defines a Documentation Asset
///
/// Assets encompas non-markdown files which are copied to the target
/// directory to support the rendered markdown files.
#[derive(Debug)]
pub enum Asset {
    /// On-Disk Asset
    ///
    /// On disk assets come from the source directory and are file-copied
    /// to the output when the asset is rendered.
    Disk(PathBuf),

    /// Internal Asset
    ///
    /// Internal assets represent fixed strings which are copied to a
    /// named file in the output directory when rendered.
    Internal(InternalAsset),
}

/// Internal Asset
///
/// Defines the fixed contents for an internal asset.
#[derive(Debug)]
pub struct InternalAsset {
    /// The contents of the asset
    contents: &'static str,
    /// The file name to create
    name: &'static str,
}

impl Asset {
    /// Create an Internal Asset
    ///
    /// # Parameters
    ///  * `name` - The name of the file to create in the output
    ///  * `contents` - The contents to fill the file with
    pub const fn internal(name: &'static str, contents: &'static str) -> Self {
        Asset::Internal(InternalAsset { name, contents })
    }

    /// Create a Path Asset
    ///
    /// # Parameters
    ///  * `path` - The source path
    pub const fn path(path: PathBuf) -> Self {
        Asset::Disk(path)
    }

    /// Copy To
    ///
    /// This method is called to copy a given asset to the output
    /// directory.
    pub fn copy_to(&self, output: &Path) -> Result<()> {
        match self {
            Asset::Internal(int) => {
                let path = output.join(int.name);
                let mut file = File::create(&path)?;
                write!(file, "{}", int.contents)?;
                Ok(())
            }
            Asset::Disk(path) => copy_single(&path, output),
        }
    }
}

fn copy_single(path: &Path, target: &Path) -> Result<()> {
    if path.is_dir() {
        let mut target = PathBuf::from(target);
        target.push(path.file_name().unwrap());
        if !target.exists() {
            fs::create_dir(&target)?;
        }
        copy_recurse(path, &target)?;
    } else if let Some(name) = path.file_name() {
        fs::copy(path, target.join(name))?;
    } else {
        warn!("Asset at {:?} does not appear to be copyable", path);
    }
    Ok(())
}

fn copy_recurse(source: &Path, target: &Path) -> Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        copy_single(&path, target)?;
    }
    Ok(())
}

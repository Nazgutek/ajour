use crate::{addon::Addon, Result};
use std::fs::remove_dir_all;
use std::path::PathBuf;

/// Deletes an Addon and all dependencies from disk.
pub fn delete_addons(path: &PathBuf, dependencies: &[String]) -> Result<()> {
    for dependency in dependencies {
        let path = path.join(dependency);
        if path.exists() {
            remove_dir_all(path)?;
        }
    }

    Ok(())
}

/// Unzips an `Addon` archive, and once that is done, it moves the content
/// to the `to_directory`.
/// At the end it will cleanup and remove the archive.
/// Returns the paths of the addons in the archive
pub async fn install_addon(
    addon: &Addon,
    from_directory: &PathBuf,
    to_directory: &PathBuf,
) -> Result<Vec<PathBuf>> {
    let zip_path = from_directory.join(addon.id.clone());
    let mut zip_file = std::fs::File::open(&zip_path)?;
    let mut archive = zip::ZipArchive::new(&mut zip_file)?;
    let mut addon_dirs = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let path = to_directory.join(file.sanitized_name());

        // Check if file is a root addon toc file
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            let toc_path = format!("{}/{}.toc", stem, stem);
            if let Some(name) = file.sanitized_name().to_str() {
                if name == toc_path {
                    let mut dir = to_directory.join(file.sanitized_name());
                    dir.pop();
                    addon_dirs.push(dir);
                }
            }
        }

        // If top-level destination folder for addon, delete that folder to remove
        // the previous version so we guarantee a clean copy
        if let Some(parent) = path.parent() {
            if parent == to_directory {
                let _ = std::fs::remove_dir_all(&path);
            }
        }

        if file.is_dir() {
            std::fs::create_dir_all(&path)?;
        } else {
            if let Some(p) = path.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = std::fs::File::create(&path)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    // Cleanup
    std::fs::remove_file(&zip_path)?;

    Ok(addon_dirs)
}

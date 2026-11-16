//! Import user-provided Minecraft mod jars without overwriting existing files.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use crate::{Error, Result};

const MAX_MOD_BYTES: u64 = 512 * 1024 * 1024;
const MOD_METADATA: &[&str] = &[
    "fabric.mod.json",
    "quilt.mod.json",
    "META-INF/mods.toml",
    "META-INF/neoforge.mods.toml",
    "mcmod.info",
];

/// A summary of a local mod import. Files not accepted are reported with a reason.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocalImport {
    pub added: Vec<String>,
    pub skipped: Vec<String>,
}

/// Copy recognized Minecraft mod jars into a game's `mods` directory.
///
/// The file picker may supply any path, so extension, archive structure and mod metadata are
/// checked again here. Existing enabled or disabled files are never overwritten.
pub fn import_local(paths: &[PathBuf], mods_dir: &Path) -> Result<LocalImport> {
    fs::create_dir_all(mods_dir).map_err(|e| Error::io(mods_dir, e))?;
    let mut summary = LocalImport::default();

    for source in paths {
        let label = source
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| source.display().to_string());
        let Some(name) = source.file_name().and_then(|n| n.to_str()) else {
            summary.skipped.push(format!("{label}: invalid file name"));
            continue;
        };
        let safe_name = !name.is_empty()
            && !name.starts_with('.')
            && !name.contains("..")
            && !name
                .chars()
                .any(|c| matches!(c, '/' | '\\' | ':' | '\0') || c.is_control());
        if !safe_name || !name.to_ascii_lowercase().ends_with(".jar") {
            summary
                .skipped
                .push(format!("{label}: choose a .jar Minecraft mod"));
            continue;
        }
        match is_minecraft_mod(source) {
            Ok(true) => {}
            Ok(false) => {
                summary
                    .skipped
                    .push(format!("{name}: not a recognized Minecraft mod JAR"));
                continue;
            }
            Err(reason) => {
                summary.skipped.push(format!("{name}: {reason}"));
                continue;
            }
        }

        let destination = mods_dir.join(name);
        if destination.exists() || mods_dir.join(format!("{name}.disabled")).exists() {
            summary.skipped.push(format!(
                "{name}: a file with this name is already installed"
            ));
            continue;
        }
        match copy_new(source, &destination) {
            Ok(()) => summary.added.push(name.to_owned()),
            Err(e) => summary.skipped.push(format!("{name}: {e}")),
        }
    }

    Ok(summary)
}

fn is_minecraft_mod(path: &Path) -> std::result::Result<bool, String> {
    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    if !metadata.is_file() {
        return Err("not a regular file".to_owned());
    }
    if metadata.len() > MAX_MOD_BYTES {
        return Err("file is larger than the 512 MiB limit".to_owned());
    }
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|_| "not a valid JAR archive".to_owned())?;
    if archive
        .file_names()
        .any(|entry| MOD_METADATA.contains(&entry))
    {
        return Ok(true);
    }
    // Some legacy Forge coremods declare themselves only in the manifest. Do not accept every
    // ordinary Java library just because it has a manifest.
    let Ok(mut manifest) = archive.by_name("META-INF/MANIFEST.MF") else {
        return Ok(false);
    };
    let mut text = String::new();
    manifest
        .by_ref()
        .take(64 * 1024)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    Ok(text.contains("FMLCorePlugin") || text.contains("TweakClass"))
}

fn copy_new(source: &Path, destination: &Path) -> io::Result<()> {
    let result = (|| {
        let mut input = File::open(source)?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(destination)?;
        io::copy(&mut input, &mut output)?;
        output.flush()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(destination);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    fn jar(path: &Path, entry: &str) {
        let file = File::create(path).unwrap();
        let mut zip = ZipWriter::new(file);
        zip.start_file(entry, SimpleFileOptions::default()).unwrap();
        zip.write_all(b"{}").unwrap();
        zip.finish().unwrap();
    }

    #[test]
    fn imports_only_recognized_mod_jars_without_overwriting() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("sample.jar");
        let bad = temp.path().join("not-a-mod.jar");
        let mods = temp.path().join("game/mods");
        jar(&source, "fabric.mod.json");
        jar(&bad, "readme.txt");

        let first = import_local(&[source.clone(), bad.clone()], &mods).unwrap();
        assert_eq!(first.added, ["sample.jar"]);
        assert_eq!(first.skipped.len(), 1);
        assert!(mods.join("sample.jar").is_file());
        assert!(!mods.join("not-a-mod.jar").exists());

        let second = import_local(&[source], &mods).unwrap();
        assert!(second.added.is_empty());
        assert!(second.skipped[0].contains("already installed"));
    }

    #[test]
    fn rejects_non_jar_and_corrupt_files() {
        let temp = tempdir().unwrap();
        let text = temp.path().join("wrong.txt");
        let corrupt = temp.path().join("broken.jar");
        fs::write(&text, b"not a jar").unwrap();
        fs::write(&corrupt, b"not a zip").unwrap();
        let result = import_local(&[text, corrupt], &temp.path().join("mods")).unwrap();
        assert!(result.added.is_empty());
        assert_eq!(result.skipped.len(), 2);
        assert!(result.skipped.iter().any(|s| s.contains("choose a .jar")));
        assert!(result.skipped.iter().any(|s| s.contains("valid JAR")));
    }
}

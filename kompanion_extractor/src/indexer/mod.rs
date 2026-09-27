pub mod epub;
pub mod cbz;
pub mod fb2;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use kompanion_core::indexer::FileIndexer;

const GENERATED_COVER_STEM: &str = "kompanion-cover";

pub fn prepare_cover_sidecar(full_path: &str) -> Result<PathBuf, String> {
    remove_generated_covers(full_path)?;

    let sidecar = Path::new(full_path).with_extension("sdr");
    std::fs::create_dir_all(&sidecar)
        .map_err(|e| format!("Failed to create SDR {}: {e}", sidecar.display()))?;
    Ok(sidecar)
}

pub fn generated_cover_path(sidecar: &Path, extension: &str) -> PathBuf {
    sidecar.join(format!("{GENERATED_COVER_STEM}.{extension}"))
}

pub fn remove_generated_covers(full_path: &str) -> Result<(), String> {
    let sidecar = Path::new(full_path).with_extension("sdr");
    remove_covers_with_stem(&sidecar, GENERATED_COVER_STEM)?;
    remove_dir_if_empty(&sidecar)?;

    let legacy_sidecar = PathBuf::from(format!("{full_path}.sdr"));
    remove_covers_with_stem(&legacy_sidecar, "cover")?;
    remove_dir_if_empty(&legacy_sidecar)?;

    Ok(())
}

fn remove_covers_with_stem(sidecar: &Path, stem: &str) -> Result<(), String> {
    let entries = match std::fs::read_dir(sidecar) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(format!("Failed to read SDR {}: {e}", sidecar.display())),
    };

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read SDR entry: {e}"))?;
        let path = entry.path();
        if path.is_file() && path.file_stem().and_then(|s| s.to_str()) == Some(stem) {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to remove generated cover {}: {e}", path.display()))?;
        }
    }

    Ok(())
}

fn remove_dir_if_empty(path: &Path) -> Result<(), String> {
    let mut entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(format!("Failed to read SDR {}: {e}", path.display())),
    };

    if entries.next().is_none() {
        std::fs::remove_dir(path)
            .map_err(|e| format!("Failed to remove empty SDR {}: {e}", path.display()))?;
    }

    Ok(())
}

static REGISTRY: OnceLock<Vec<Box<dyn FileIndexer>>> = OnceLock::new();

pub fn init_registry() {
    REGISTRY
        .set(vec![
            Box::new(epub::EpubIndexer),
            Box::new(cbz::CbzIndexer),
            Box::new(fb2::Fb2Indexer),
        ])
        .ok();
}

pub fn find_indexer(filename: &str) -> Option<&dyn FileIndexer> {
    REGISTRY
        .get()?
        .iter()
        .find(|i| i.can_handle(filename))
        .map(|b| b.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_cover_cleanup_preserves_koreader_state_and_custom_cover() {
        let root =
            std::env::temp_dir().join(format!("kompanion_cover_cleanup_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        let book = root.join("book.EPUB");
        std::fs::write(&book, b"book").unwrap();

        let sidecar = book.with_extension("sdr");
        std::fs::create_dir_all(&sidecar).unwrap();
        let metadata = sidecar.join("metadata.EPUB.lua");
        let custom_cover = sidecar.join("cover.jpg");
        let generated_cover = sidecar.join("kompanion-cover.jpg");
        std::fs::write(&metadata, b"reading state").unwrap();
        std::fs::write(&custom_cover, b"custom cover").unwrap();
        std::fs::write(&generated_cover, b"generated cover").unwrap();

        let legacy_sidecar = std::path::PathBuf::from(format!("{}.sdr", book.display()));
        std::fs::create_dir_all(&legacy_sidecar).unwrap();
        let legacy_cover = legacy_sidecar.join("cover.jpeg");
        let unrelated_legacy_file = legacy_sidecar.join("keep.txt");
        std::fs::write(&legacy_cover, b"legacy generated cover").unwrap();
        std::fs::write(&unrelated_legacy_file, b"keep").unwrap();

        remove_generated_covers(&book.to_string_lossy()).unwrap();

        assert!(!generated_cover.exists());
        assert!(!legacy_cover.exists());
        assert_eq!(std::fs::read(&metadata).unwrap(), b"reading state");
        assert_eq!(std::fs::read(&custom_cover).unwrap(), b"custom cover");
        assert_eq!(std::fs::read(&unrelated_legacy_file).unwrap(), b"keep");

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn test_finds_correct_indexer() {
        init_registry();
        assert!(find_indexer("script.sh").is_none());
        assert!(find_indexer("book.epub").is_some());
        assert!(find_indexer("book.pdf").is_none());
        assert!(find_indexer("image.png").is_none());
    }

    #[test]
    fn test_epub_indexer_handles_epub() {
        init_registry();
        let idx = find_indexer("book.epub").unwrap();
        assert!(idx.can_handle("book.epub"));
        assert!(!idx.can_handle("book.sh"));
        assert_eq!(idx.mime_type(), "application/epub+zip");
    }

    #[test]
    fn test_cde_type_per_indexer() {
        init_registry();
        assert_eq!(find_indexer("book.epub").unwrap().cde_type(), "EBOK");
        assert_eq!(find_indexer("book.fb2").unwrap().cde_type(), "EBOK");
        // CBZ stays a personal document: there is no comic cdeType in ccat.
        assert_eq!(find_indexer("comic.cbz").unwrap().cde_type(), "EBOK");
    }
}

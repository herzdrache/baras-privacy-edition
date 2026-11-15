use std::{fs, path::PathBuf};
use baras_core::{context::{DirectoryIndex, LogAreaCache}, history::LibraryMeta, library};
use baras_types::history::ArchiveReport;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let profile = PathBuf::from(std::env::var_os("BARAS_PRIVACY_TEST_PROFILE").unwrap());
    assert_eq!(baras_core::paths::app_config_dir(), profile);
    let dir = profile.join("roundtrip");
    fs::create_dir_all(&dir)?;
    let mut state = library::LibraryMut {
        directory: &mut DirectoryIndex::default(), areas: &mut LogAreaCache::default(),
        meta: &mut LibraryMeta::default(), live: None,
    };
    for input in std::env::args_os().skip(1).map(PathBuf::from) {
        let name = input.file_name().unwrap().to_str().unwrap();
        let copy = dir.join(name);
        fs::copy(&input, &copy)?;
        let modified = fs::metadata(&copy)?.modified()?;
        let compressed = library::compress_log(&copy)?;
        let mut report = ArchiveReport::default();
        library::commit(&mut state, compressed, &mut report);
        assert_eq!(report.archived, 1);
        assert!(!copy.exists());
        assert!(library::archived_log(name).is_some());
        assert_eq!(library::restore(&mut state, &dir, name)?, copy);
        assert!(library::archived_log(name).is_none());
        assert_eq!(fs::read(&copy)?, fs::read(&input)?);
        assert_eq!(fs::metadata(&copy)?.modified()?, modified);
        println!("{name} restored byte-for-byte, mtime matched");
    }
    Ok(())
}

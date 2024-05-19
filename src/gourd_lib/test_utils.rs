use anyhow::bail;

use crate::file_system::FileOperations;
use crate::file_system::FileSystemInteractor;

pub const REAL_FS: FileSystemInteractor = FileSystemInteractor { dry_run: false };

// This will come in useful later.
#[allow(dead_code)]
pub const EMPTY_FS: EmptyFilesystem = EmptyFilesystem {};

pub struct EmptyFilesystem;

impl FileOperations for EmptyFilesystem {
    fn read_utf8(&self, _: &std::path::Path) -> anyhow::Result<String> {
        Ok("== Nonsese".to_string())
    }

    fn read_bytes(&self, path: &std::path::Path) -> anyhow::Result<Vec<u8>> {
        bail!("File not found: {path:?}")
    }

    fn try_read_toml<T: serde::de::DeserializeOwned>(
        &self,
        _: &std::path::Path,
    ) -> anyhow::Result<T> {
        bail!("File not found")
    }

    fn try_write_toml<T: serde::Serialize>(
        &self,
        _: &std::path::Path,
        _: &T,
    ) -> anyhow::Result<()> {
        bail!("File not found")
    }

    fn write_utf8_truncate(&self, _: &std::path::Path, _: &str) -> anyhow::Result<()> {
        bail!("File not found")
    }

    fn write_bytes_truncate(&self, _: &std::path::Path, _: &[u8]) -> anyhow::Result<()> {
        bail!("File not found")
    }

    fn truncate_and_canonicalize(&self, _: &std::path::Path) -> anyhow::Result<std::path::PathBuf> {
        bail!("File not found")
    }
}

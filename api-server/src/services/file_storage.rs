use mime_guess;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

use crate::config::FileStorageConfig;

#[derive(Error, Debug)]
pub enum FileStorageError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid file type: {0}")]
    InvalidFileType(String),

    #[error("File too large: {0} bytes (max: {1})")]
    FileTooLarge(u64, u64),

    #[error("Invalid file path: {0}")]
    InvalidPath(String),
}

/// File storage service for handling image uploads and serving
pub struct FileStorageService {
    config: FileStorageConfig,
    base_path: PathBuf,
}

impl FileStorageService {
    pub fn new(config: FileStorageConfig) -> Result<Self, FileStorageError> {
        let base_path = PathBuf::from(&config.base_path);

        // Create base directory if it doesn't exist
        std::fs::create_dir_all(&base_path)?;

        Ok(Self { config, base_path })
    }

    /// Store a file and return the stored file info
    pub async fn store_file(
        &self,
        filename: &str,
        content: &[u8],
        session_id: Uuid,
    ) -> Result<StoredFileInfo, FileStorageError> {
        // Validate file size
        if content.len() as u64 > self.config.max_file_size {
            return Err(FileStorageError::FileTooLarge(
                content.len() as u64,
                self.config.max_file_size,
            ));
        }

        // Guess MIME type from filename
        let mime_type = mime_guess::from_path(filename)
            .first_or_octet_stream()
            .to_string();

        // Validate file type
        if !self.config.allowed_types.contains(&mime_type) {
            return Err(FileStorageError::InvalidFileType(mime_type));
        }

        // Generate unique filename to prevent conflicts
        let file_id = Uuid::new_v4();
        let extension = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("bin");
        let stored_filename = format!("{}_{}.{}", session_id, file_id, extension);

        // Create session directory
        let session_dir = self.base_path.join("sessions").join(session_id.to_string());
        fs::create_dir_all(&session_dir).await?;

        // Full file path
        let file_path = session_dir.join(&stored_filename);

        // Write file to disk
        let mut file = fs::File::create(&file_path).await?;
        file.write_all(content).await?;
        file.flush().await?;

        tracing::info!(
            "Stored file: {} ({} bytes) at {:?}",
            stored_filename,
            content.len(),
            file_path
        );

        Ok(StoredFileInfo {
            id: file_id,
            filename: stored_filename,
            original_filename: filename.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            content_type: mime_type,
            file_size: content.len() as u64,
        })
    }

    /// Read a file from storage
    pub async fn read_file(&self, file_path: &str) -> Result<Vec<u8>, FileStorageError> {
        let path = Path::new(file_path);

        // Security check - ensure path is within base directory
        if !path.starts_with(&self.base_path) {
            return Err(FileStorageError::InvalidPath(file_path.to_string()));
        }

        if !path.exists() {
            return Err(FileStorageError::FileNotFound(file_path.to_string()));
        }

        let mut file = fs::File::open(path).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;

        Ok(contents)
    }

    /// Delete a file from storage
    pub async fn delete_file(&self, file_path: &str) -> Result<(), FileStorageError> {
        let path = Path::new(file_path);

        // Security check - ensure path is within base directory
        if !path.starts_with(&self.base_path) {
            return Err(FileStorageError::InvalidPath(file_path.to_string()));
        }

        if path.exists() {
            fs::remove_file(path).await?;
            tracing::info!("Deleted file: {:?}", path);
        }

        Ok(())
    }

    /// Check if a file exists
    pub async fn file_exists(&self, file_path: &str) -> bool {
        let path = Path::new(file_path);
        path.exists() && path.is_file()
    }

    /// Get file metadata
    pub async fn get_file_metadata(
        &self,
        file_path: &str,
    ) -> Result<FileMetadata, FileStorageError> {
        let path = Path::new(file_path);

        // Security check
        if !path.starts_with(&self.base_path) {
            return Err(FileStorageError::InvalidPath(file_path.to_string()));
        }

        if !path.exists() {
            return Err(FileStorageError::FileNotFound(file_path.to_string()));
        }

        let metadata = fs::metadata(path).await?;
        let mime_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        Ok(FileMetadata {
            size: metadata.len(),
            content_type: mime_type,
            modified: metadata
                .modified()
                .ok()
                .map(|time| chrono::DateTime::<chrono::Utc>::from(time)),
        })
    }

    /// Clean up old files (for maintenance)
    pub async fn cleanup_old_files(&self, days_old: u64) -> Result<usize, FileStorageError> {
        use std::time::{Duration, SystemTime};

        let cutoff = SystemTime::now() - Duration::from_secs(days_old * 24 * 60 * 60);
        let mut deleted_count = 0;

        let mut entries = fs::read_dir(&self.base_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;

            if metadata.is_file() {
                if let Ok(modified) = metadata.modified() {
                    if modified < cutoff {
                        if let Err(e) = fs::remove_file(entry.path()).await {
                            tracing::warn!("Failed to delete old file {:?}: {}", entry.path(), e);
                        } else {
                            deleted_count += 1;
                        }
                    }
                }
            }
        }

        tracing::info!("Cleaned up {} old files", deleted_count);
        Ok(deleted_count)
    }

    /// Get storage statistics
    pub async fn get_storage_stats(&self) -> Result<StorageStats, FileStorageError> {
        let mut total_files = 0;
        let mut total_size = 0;

        fn calculate_dir_size(path: &Path) -> std::io::Result<(u64, u64)> {
            let mut file_count = 0;
            let mut size = 0;

            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let metadata = entry.metadata()?;

                if metadata.is_file() {
                    file_count += 1;
                    size += metadata.len();
                } else if metadata.is_dir() {
                    let (sub_files, sub_size) = calculate_dir_size(&entry.path())?;
                    file_count += sub_files;
                    size += sub_size;
                }
            }

            Ok((file_count, size))
        }

        if self.base_path.exists() {
            let (files, size) = calculate_dir_size(&self.base_path)?;
            total_files = files;
            total_size = size;
        }

        Ok(StorageStats {
            total_files,
            total_size_bytes: total_size,
            available_space_bytes: None, // TODO: Implement disk space check
        })
    }
}

#[derive(Debug, Clone)]
pub struct StoredFileInfo {
    pub id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub file_path: String,
    pub content_type: String,
    pub file_size: u64,
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub content_type: String,
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct StorageStats {
    pub total_files: u64,
    pub total_size_bytes: u64,
    pub available_space_bytes: Option<u64>,
}

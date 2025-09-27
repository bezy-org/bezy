use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tokio::process::Command;
use tokio::fs;

pub struct FontCompiler {
    #[allow(dead_code)]
    temp_dir: PathBuf,
    cache_dir: PathBuf,
}

impl FontCompiler {
    pub fn new() -> Self {
        let temp_dir = Self::get_qa_temp_dir();
        let cache_dir = temp_dir.join("compiled");

        Self {
            temp_dir,
            cache_dir,
        }
    }

    fn get_qa_temp_dir() -> PathBuf {
        let config_dir = if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config").join("bezy").join("qa").join("temp")
        } else {
            PathBuf::from("/tmp").join("bezy-qa")
        };

        // Create directory if it doesn't exist
        if let Err(_e) = std::fs::create_dir_all(&config_dir) {
            // Silently fallback to /tmp if config dir creation fails
            return PathBuf::from("/tmp").join("bezy-qa");
        }

        config_dir
    }

    pub async fn compile_for_qa(&self, ufo_path: &Path) -> Result<PathBuf> {
        // Create cache directory
        fs::create_dir_all(&self.cache_dir).await?;

        // Generate hash for cache key
        let font_hash = self.calculate_font_hash(ufo_path).await?;
        let cached_font = self.cache_dir.join(format!("{}.ttf", font_hash));

        // Check if cached version exists and is newer than source
        if cached_font.exists() {
            if let Ok(cache_meta) = fs::metadata(&cached_font).await {
                if let Ok(source_meta) = fs::metadata(ufo_path).await {
                    if let (Ok(cache_time), Ok(source_time)) = (cache_meta.modified(), source_meta.modified()) {
                        if cache_time >= source_time {
                            return Ok(cached_font);
                        }
                    }
                }
            }
        }

        // Compile using FontC
        self.compile_with_fontc(ufo_path, &cached_font).await?;

        Ok(cached_font)
    }

    async fn compile_with_fontc(&self, ufo_path: &Path, output_path: &Path) -> Result<()> {
        let mut cmd = Command::new("fontc");
        cmd.arg(ufo_path)
            .arg("--output")
            .arg(output_path);

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("FontC compilation failed: {}", stderr));
        }

        Ok(())
    }

    async fn calculate_font_hash(&self, ufo_path: &Path) -> Result<String> {
        let mut hasher = DefaultHasher::new();

        // Hash the UFO path
        ufo_path.hash(&mut hasher);

        // Hash the modification time
        let metadata = fs::metadata(ufo_path).await?;
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                duration.as_secs().hash(&mut hasher);
            }
        }

        Ok(format!("{:x}", hasher.finish()))
    }

    pub async fn cleanup_old_cache(&self, max_files: usize) -> Result<()> {
        let mut entries = Vec::new();

        let mut dir = fs::read_dir(&self.cache_dir).await?;
        while let Some(entry) = dir.next_entry().await? {
            if let Ok(metadata) = entry.metadata().await {
                if let Ok(modified) = metadata.modified() {
                    entries.push((entry.path(), modified));
                }
            }
        }

        if entries.len() <= max_files {
            return Ok(());
        }

        // Sort by modification time (newest first)
        entries.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove oldest files
        for (path, _) in entries.iter().skip(max_files) {
            if let Err(_e) = fs::remove_file(path).await {
                // Silently ignore cache cleanup errors
            }
        }

        Ok(())
    }
}
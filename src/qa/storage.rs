use crate::qa::QAReport;
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use std::path::{Path, PathBuf};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub struct ReportStorage {
    reports_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredQAReport {
    pub report: QAReport,
    pub font_hash: String,
    pub storage_timestamp: std::time::SystemTime,
}

impl ReportStorage {
    pub fn new() -> Result<Self> {
        let reports_dir = Self::get_reports_dir();
        std::fs::create_dir_all(&reports_dir)?;

        Ok(Self { reports_dir })
    }

    fn get_reports_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config").join("bezy").join("qa").join("reports")
        } else {
            PathBuf::from("/tmp").join("bezy-qa-reports")
        }
    }

    pub async fn store_report(&self, report: &QAReport) -> Result<()> {
        let font_hash = self.calculate_font_hash(&report.font_path)?;
        let font_dir = self.reports_dir.join(&font_hash);

        // Create font-specific directory
        fs::create_dir_all(&font_dir).await?;

        let stored_report = StoredQAReport {
            report: report.clone(),
            font_hash: font_hash.clone(),
            storage_timestamp: std::time::SystemTime::now(),
        };

        // Store as latest.json
        let latest_path = font_dir.join("latest.json");
        self.write_report_to_file(&stored_report, &latest_path).await?;

        // Store timestamped copy
        let timestamp = stored_report.storage_timestamp
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| anyhow!("Time error: {}", e))?
            .as_secs();
        let timestamped_path = font_dir.join(format!("{}.json", timestamp));
        self.write_report_to_file(&stored_report, &timestamped_path).await?;

        // Update summary
        self.update_summary(&font_hash, &stored_report).await?;

        // Cleanup old reports
        self.cleanup_old_reports(&font_dir, 10).await?;

        Ok(())
    }

    async fn write_report_to_file(&self, report: &StoredQAReport, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(report)?;
        let mut file = fs::File::create(path).await?;
        file.write_all(json.as_bytes()).await?;
        file.flush().await?;
        Ok(())
    }

    async fn update_summary(&self, font_hash: &str, report: &StoredQAReport) -> Result<()> {
        let font_dir = self.reports_dir.join(font_hash);
        let summary_path = font_dir.join("summary.json");

        let summary = ReportSummary {
            font_hash: font_hash.to_string(),
            font_path: report.report.font_path.clone(),
            latest_timestamp: report.storage_timestamp,
            latest_summary: report.report.summary.clone(),
            report_count: self.count_reports(&font_dir).await?,
        };

        let json = serde_json::to_string_pretty(&summary)?;
        let mut file = fs::File::create(summary_path).await?;
        file.write_all(json.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }

    async fn count_reports(&self, font_dir: &Path) -> Result<usize> {
        let mut count = 0;
        let mut dir = fs::read_dir(font_dir).await?;

        while let Some(entry) = dir.next_entry().await? {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            // Count timestamped JSON files (exclude latest.json and summary.json)
            if file_name.ends_with(".json") &&
               file_name != "latest.json" &&
               file_name != "summary.json" {
                count += 1;
            }
        }

        Ok(count)
    }

    async fn cleanup_old_reports(&self, font_dir: &Path, max_reports: usize) -> Result<()> {
        let mut reports = Vec::new();
        let mut dir = fs::read_dir(font_dir).await?;

        while let Some(entry) = dir.next_entry().await? {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Only consider timestamped reports
            if file_name_str.ends_with(".json") &&
               file_name_str != "latest.json" &&
               file_name_str != "summary.json" {

                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(modified) = metadata.modified() {
                        reports.push((entry.path(), modified));
                    }
                }
            }
        }

        if reports.len() <= max_reports {
            return Ok(());
        }

        // Sort by modification time (newest first)
        reports.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove oldest reports
        for (path, _) in reports.iter().skip(max_reports) {
            if let Err(_e) = fs::remove_file(path).await {
                // Silently ignore cleanup errors
            }
        }

        Ok(())
    }

    pub async fn load_latest_report(&self, font_path: &Path) -> Result<Option<StoredQAReport>> {
        let font_hash = self.calculate_font_hash(font_path)?;
        let latest_path = self.reports_dir.join(&font_hash).join("latest.json");

        if !latest_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(latest_path).await?;
        let report: StoredQAReport = serde_json::from_str(&content)?;
        Ok(Some(report))
    }

    pub async fn load_report_history(&self, font_path: &Path, limit: Option<usize>) -> Result<Vec<StoredQAReport>> {
        let font_hash = self.calculate_font_hash(font_path)?;
        let font_dir = self.reports_dir.join(&font_hash);

        if !font_dir.exists() {
            return Ok(Vec::new());
        }

        let mut reports = Vec::new();
        let mut entries = Vec::new();
        let mut dir = fs::read_dir(&font_dir).await?;

        while let Some(entry) = dir.next_entry().await? {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if file_name_str.ends_with(".json") &&
               file_name_str != "latest.json" &&
               file_name_str != "summary.json" {

                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(modified) = metadata.modified() {
                        entries.push((entry.path(), modified));
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        entries.sort_by(|a, b| b.1.cmp(&a.1));

        // Limit results if requested
        let entries_to_load = if let Some(limit) = limit {
            entries.into_iter().take(limit).collect()
        } else {
            entries
        };

        for (path, _) in entries_to_load {
            if let Ok(content) = fs::read_to_string(&path).await {
                if let Ok(report) = serde_json::from_str::<StoredQAReport>(&content) {
                    reports.push(report);
                }
            }
        }

        Ok(reports)
    }

    fn calculate_font_hash(&self, font_path: &Path) -> Result<String> {
        let mut hasher = DefaultHasher::new();
        font_path.hash(&mut hasher);
        Ok(format!("{:x}", hasher.finish()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub font_hash: String,
    pub font_path: PathBuf,
    pub latest_timestamp: std::time::SystemTime,
    pub latest_summary: crate::qa::QASummary,
    pub report_count: usize,
}
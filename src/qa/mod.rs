pub mod fontspector;
pub mod compiler;
pub mod trigger;
pub mod storage;

use anyhow::Result;
use std::path::PathBuf;
use std::time::SystemTime;

pub struct QAEngine {
    fontspector: fontspector::FontspectorRunner,
    compiler: compiler::FontCompiler,
    trigger: trigger::QASaveTrigger,
    storage: storage::ReportStorage,
}

impl QAEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            fontspector: fontspector::FontspectorRunner::new()?,
            compiler: compiler::FontCompiler::new(),
            trigger: trigger::QASaveTrigger::new(),
            storage: storage::ReportStorage::new()?,
        })
    }

    pub async fn run_qa_on_save(&mut self, ufo_path: &PathBuf) -> Result<QAReport> {
        // 1. Compile UFO to TTF/OTF
        let compiled_font = self.compiler.compile_for_qa(ufo_path).await?;

        // 2. Run Fontspector analysis
        let report = self.fontspector.analyze(&compiled_font).await?;

        // 3. Store report
        self.storage.store_report(&report).await?;

        Ok(report)
    }
}

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAReport {
    pub font_path: PathBuf,
    #[serde(with = "crate::qa::time_serde")]
    pub timestamp: SystemTime,
    pub issues: Vec<QAIssue>,
    pub summary: QASummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QAIssue {
    pub severity: Severity,
    pub category: Category,
    pub check_id: String,
    pub message: String,
    pub location: Option<Location>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Category {
    Outlines,
    Metadata,
    Hinting,
    Kerning,
    Spacing,
    Unicode,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub glyph_name: Option<String>,
    pub table_name: Option<String>,
    pub position: Option<(f32, f32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QASummary {
    pub total_checks: usize,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub info: usize,
    pub skipped: usize,
}

// Custom serde module for SystemTime
pub mod time_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(UNIX_EPOCH).map_err(serde::ser::Error::custom)?;
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + std::time::Duration::from_secs(secs))
    }
}
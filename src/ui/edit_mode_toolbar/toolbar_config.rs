//! Centralized Toolbar Configuration
//!
//! This file contains ALL toolbar tool definitions in one place.
//! To modify the toolbar:
//! 1. Change the order number to reposition tools
//! 2. Set enabled: false to hide a tool
//! 3. Change icon to use different Unicode character
//! 4. Modify shortcut key or set to None
//!
//! This is the SINGLE SOURCE OF TRUTH for the edit mode toolbar.

use bevy::prelude::*;

/// Complete configuration for a single toolbar tool
#[derive(Debug, Clone)]
pub struct ToolConfig {
    /// Display order in toolbar (lower = earlier, e.g. 10, 20, 30...)
    pub order: i32,

    /// Unique identifier for the tool
    pub id: &'static str,

    /// Human-readable name shown in tooltips
    pub name: &'static str,

    /// Unicode icon character (from Bezy Grotesk font)
    pub icon: &'static str,

    /// Optional keyboard shortcut
    pub shortcut: Option<char>,

    /// Whether this tool appears in the toolbar
    pub enabled: bool,

    /// What the tool does when active
    pub behavior: ToolBehavior,

    /// Description for tooltips and help
    pub description: &'static str,
}

/// Defines what each tool does when activated
#[derive(Debug, Clone)]
pub enum ToolBehavior {
    Select,
    Pan,
    Pen,
    Text,
    Shapes,
    Knife,
    Hyper,
    Measure,
    Metaballs,
    Ai,
}

/// ============================================================================
/// TOOLBAR CONFIGURATION - EDIT THIS TO CHANGE THE TOOLBAR
/// ============================================================================
///
/// This is where you configure the entire toolbar.
/// - Change `order` to reposition tools
/// - Set `enabled: false` to hide tools  
/// - Change `icon` to use different symbols
/// - Modify `shortcut` for different keys
///
/// Order guidelines:
/// - 10-19: Primary tools (Select, Pan)
/// - 20-29: Drawing tools (Pen)
/// - 30-39: Shape tools
/// - 40-49: Text tools
/// - 50-59: Editing tools (Knife)
/// - 60-69: Advanced tools (Hyper)
/// - 70-79: Utility tools (Measure)
/// - 80-89: Experimental tools (Metaballs)
pub const TOOLBAR_TOOLS: &[ToolConfig] = &[
    ToolConfig {
        order: 10,
        id: "select",
        name: "Select",
        icon: "\u{E010}", // Arrow cursor icon (FIXED - was E001, now E010)
        shortcut: Some('v'),
        enabled: true,
        behavior: ToolBehavior::Select,
        description: "Select and move points, handles, and components",
    },
    ToolConfig {
        order: 15,
        id: "pan",
        name: "Pan",
        icon: "\u{E014}",    // Hand icon (FIXED - was E015, now E014)
        shortcut: Some(' '), // Spacebar
        enabled: true,
        behavior: ToolBehavior::Pan,
        description: "Pan the view (hold spacebar for temporary mode)",
    },
    ToolConfig {
        order: 20,
        id: "pen",
        name: "Pen",
        icon: "\u{E011}", // Pen nib icon (FIXED - was E002, now E011)
        shortcut: Some('p'),
        enabled: true,
        behavior: ToolBehavior::Pen,
        description: "Draw and edit Bézier curves (Tab for modes)",
    },
    ToolConfig {
        order: 30,
        id: "shapes",
        name: "Shapes",
        icon: "\u{E016}", // Square icon (CORRECT)
        shortcut: Some('s'),
        enabled: true, // ✅ This is the square button you want to keep
        behavior: ToolBehavior::Shapes,
        description: "Create geometric shapes like rectangles and ellipses",
    },
    ToolConfig {
        order: 16,
        id: "text",
        name: "Text",
        icon: "\u{E017}", // T icon (FIXED - was E003, now E017)
        shortcut: Some('t'),
        enabled: true,
        behavior: ToolBehavior::Text,
        description: "Place text and create sorts (Tab for modes)",
    },
    ToolConfig {
        order: 50,
        id: "knife",
        name: "Knife",
        icon: "\u{E013}", // Knife icon
        shortcut: Some('k'),
        enabled: true,
        behavior: ToolBehavior::Knife,
        description: "Cut contours at specific points",
    },
    ToolConfig {
        order: 60,
        id: "hyper",
        name: "Hyper",
        icon: "\u{E012}", // Spiral icon
        shortcut: Some('h'),
        enabled: false, // ❌ Moved to pen tool submenu
        behavior: ToolBehavior::Hyper,
        description: "Draw smooth hyperbezier curves",
    },
    ToolConfig {
        order: 70,
        id: "measure",
        name: "Measure",
        icon: "\u{E015}", // Ruler icon (FIXED - was E014, now E015)
        shortcut: Some('m'),
        enabled: true,
        behavior: ToolBehavior::Measure,
        description: "Measure distances and show guides",
    },
    ToolConfig {
        order: 80,
        id: "metaballs",
        name: "Metaballs",
        icon: "\u{E019}", // Circle icon
        shortcut: Some('b'),
        enabled: false, // ❌ This is the circle button you want to disable
        behavior: ToolBehavior::Metaballs,
        description: "Create organic shapes with metaball effects",
    },
    ToolConfig {
        order: 90,
        id: "ai",
        name: "AI",
        icon: "\u{E012}", // Spiral icon (same as hyperbezier)
        shortcut: Some('a'),
        enabled: true,
        behavior: ToolBehavior::Ai,
        description: "AI-powered font editing tools (Tab for submenu)",
    },
];

/// ============================================================================
/// CONFIGURATION HELPERS
/// ============================================================================
impl ToolConfig {
    /// Get all enabled tools sorted by order
    pub fn get_enabled_tools() -> Vec<&'static ToolConfig> {
        let mut tools: Vec<_> = TOOLBAR_TOOLS.iter().filter(|tool| tool.enabled).collect();

        tools.sort_by_key(|tool| tool.order);
        tools
    }

    /// Get a specific tool by ID
    pub fn get_tool(id: &str) -> Option<&'static ToolConfig> {
        TOOLBAR_TOOLS.iter().find(|tool| tool.id == id)
    }

    /// Get all tool configurations (enabled and disabled)
    pub fn get_all_tools() -> Vec<&'static ToolConfig> {
        let mut tools: Vec<_> = TOOLBAR_TOOLS.iter().collect();
        tools.sort_by_key(|tool| tool.order);
        tools
    }
}

/// Print the current toolbar configuration for debugging
pub fn print_toolbar_config() {
    debug!("=== CURRENT TOOLBAR CONFIGURATION ===");
    for tool in ToolConfig::get_all_tools() {
        let status = if tool.enabled { "✅" } else { "❌" };
        let shortcut = tool
            .shortcut
            .map(|c| format!("'{c}'"))
            .unwrap_or_else(|| "None".to_string());

        debug!(
            "{} Order:{:2} | {} | {} | Key:{} | {}",
            status, tool.order, tool.id, tool.icon, shortcut, tool.name
        );
    }
    debug!("=====================================");
}

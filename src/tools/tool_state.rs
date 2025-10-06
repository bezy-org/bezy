//! Unified tool state management - single source of truth for active tool
//!
//! This replaces the scattered SelectModeActive, PenModeActive, etc. resources
//! with a single, consistent state management system.

use bevy::prelude::*;

/// The single source of truth for which tool is currently active
#[derive(Resource, Debug, Default)]
pub struct ToolState {
    /// Currently active tool
    pub active: ToolId,

    /// Track if active tool changed this frame (for cleanup systems)
    active_changed: bool,

    /// Previous tool (for undo/temporary modes)
    previous: Option<ToolId>,

    /// Stack for temporary tool modes (e.g., spacebar pan)
    temporary_stack: Vec<ToolId>,
}

impl ToolState {
    /// Switch to a new tool
    pub fn activate(&mut self, tool: ToolId) {
        if self.active != tool {
            self.previous = Some(self.active);
            self.active = tool;
            self.active_changed = true;
            info!("Tool switched: {:?} -> {:?}", self.previous, self.active);
        }
    }

    /// Push a temporary tool (like spacebar pan)
    pub fn push_temporary(&mut self, tool: ToolId) {
        self.temporary_stack.push(self.active);
        self.active = tool;
        self.active_changed = true;
        debug!("Pushed temporary tool: {:?}", tool);
    }

    /// Pop temporary tool and return to previous
    pub fn pop_temporary(&mut self) -> bool {
        if let Some(previous) = self.temporary_stack.pop() {
            self.active = previous;
            self.active_changed = true;
            debug!("Popped temporary tool, returned to: {:?}", previous);
            true
        } else {
            false
        }
    }

    /// Check if active tool changed this frame
    pub fn just_changed(&self) -> bool {
        self.active_changed
    }

    /// Reset the changed flag (called at end of frame)
    pub fn clear_changed(&mut self) {
        self.active_changed = false;
    }

    /// Check if a specific tool is active
    pub fn is_active(&self, tool: ToolId) -> bool {
        self.active == tool
    }

    /// Get the previous tool
    pub fn previous(&self) -> Option<ToolId> {
        self.previous
    }
}

/// Tool identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToolId {
    #[default]
    Select,
    Pen,
    Knife,
    Pan,
    Text,
    Shapes,
    Measure,
    Hyper,
    Metaballs,
    Ai,
}

impl ToolId {
    /// Get the tool's display name
    pub fn name(&self) -> &'static str {
        match self {
            ToolId::Select => "Select",
            ToolId::Pen => "Pen",
            ToolId::Knife => "Knife",
            ToolId::Pan => "Pan",
            ToolId::Text => "Text",
            ToolId::Shapes => "Shapes",
            ToolId::Measure => "Measure",
            ToolId::Hyper => "Hyper",
            ToolId::Metaballs => "Metaballs",
            ToolId::Ai => "AI",
        }
    }

    /// Convert from string ID (used in UI)
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "select" => Some(ToolId::Select),
            "pen" => Some(ToolId::Pen),
            "knife" => Some(ToolId::Knife),
            "pan" => Some(ToolId::Pan),
            "text" => Some(ToolId::Text),
            "shapes" => Some(ToolId::Shapes),
            "measure" => Some(ToolId::Measure),
            "hyper" => Some(ToolId::Hyper),
            "metaballs" => Some(ToolId::Metaballs),
            "ai" => Some(ToolId::Ai),
            _ => None,
        }
    }

    /// Convert to string ID (used in UI)
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolId::Select => "select",
            ToolId::Pen => "pen",
            ToolId::Knife => "knife",
            ToolId::Pan => "pan",
            ToolId::Text => "text",
            ToolId::Shapes => "shapes",
            ToolId::Measure => "measure",
            ToolId::Hyper => "hyper",
            ToolId::Metaballs => "metaballs",
            ToolId::Ai => "ai",
        }
    }
}

/// Event to request tool switch
#[derive(Event, Debug)]
pub struct SwitchToolEvent {
    pub tool: ToolId,
    pub temporary: bool,
}

/// System to handle tool switching
pub fn handle_tool_switch(
    mut tool_state: ResMut<ToolState>,
    mut events: EventReader<SwitchToolEvent>,
    mut input_mode: ResMut<crate::io::input::InputMode>,
) {
    for event in events.read() {
        debug!("SwitchToolEvent received: {:?}", event);

        if event.temporary {
            tool_state.push_temporary(event.tool);
        } else {
            tool_state.activate(event.tool);
        }

        // Update input mode to match tool
        let new_mode = match event.tool {
            ToolId::Select => crate::io::input::InputMode::Select,
            ToolId::Pen => crate::io::input::InputMode::Pen,
            ToolId::Knife => crate::io::input::InputMode::Knife,
            ToolId::Pan => crate::io::input::InputMode::Pan,
            ToolId::Text => crate::io::input::InputMode::Text,
            ToolId::Shapes => crate::io::input::InputMode::Shape,
            ToolId::Measure => crate::io::input::InputMode::Measure,
            ToolId::Hyper => crate::io::input::InputMode::Hyper,
            ToolId::Metaballs => crate::io::input::InputMode::Metaball,
            ToolId::Ai => crate::io::input::InputMode::Normal,
        };

        *input_mode = new_mode;
        debug!("Tool switched to {:?}, InputMode set to {:?}", event.tool, new_mode);
    }
}

/// System to sync CurrentTool resource with ToolState (for compatibility)
pub fn sync_current_tool(
    tool_state: Res<ToolState>,
    mut current_tool: ResMut<crate::ui::edit_mode_toolbar::CurrentTool>,
) {
    if tool_state.just_changed() {
        current_tool.switch_to(tool_state.active.as_str());
    }
}

/// System to clear the changed flag at end of frame
pub fn clear_tool_changed(mut tool_state: ResMut<ToolState>) {
    tool_state.clear_changed();
}

/// Run condition for systems that only run when a specific tool is active
pub fn tool_is_active(tool: ToolId) -> impl Fn(Res<ToolState>) -> bool {
    move |state: Res<ToolState>| state.is_active(tool)
}

/// Plugin to add unified tool state management
pub struct ToolStatePlugin;

impl Plugin for ToolStatePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ToolState>()
            .add_event::<SwitchToolEvent>()
            // Initialize InputMode with default value
            .insert_resource(crate::io::input::InputMode::Normal)
            .add_systems(
                PreUpdate,
                handle_tool_switch
            )
            .add_systems(
                Update,
                sync_current_tool
            )
            .add_systems(
                PostUpdate,
                clear_tool_changed
            );
    }
}
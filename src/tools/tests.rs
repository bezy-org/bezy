//! Tests for the unified tool state system

#[cfg(test)]
mod tests {
    use super::super::*;
    use bevy::prelude::*;

    #[test]
    fn test_tool_state_activation() {
        let mut tool_state = ToolState::default();

        // Default should be Select
        assert_eq!(tool_state.active, ToolId::Select);
        assert!(!tool_state.just_changed());

        // Activate Pen tool
        tool_state.activate(ToolId::Pen);
        assert_eq!(tool_state.active, ToolId::Pen);
        assert!(tool_state.just_changed());
        assert_eq!(tool_state.previous(), Some(ToolId::Select));

        // Clear changed flag
        tool_state.clear_changed();
        assert!(!tool_state.just_changed());

        // Activate same tool (no change)
        tool_state.activate(ToolId::Pen);
        assert!(!tool_state.just_changed());
    }

    #[test]
    fn test_temporary_tool_stack() {
        let mut tool_state = ToolState::default();

        // Start with Select
        assert_eq!(tool_state.active, ToolId::Select);

        // Push temporary Pan tool
        tool_state.push_temporary(ToolId::Pan);
        assert_eq!(tool_state.active, ToolId::Pan);
        assert!(tool_state.just_changed());

        // Pop back to Select
        let popped = tool_state.pop_temporary();
        assert!(popped);
        assert_eq!(tool_state.active, ToolId::Select);
        assert!(tool_state.just_changed());

        // Pop when empty returns false
        tool_state.clear_changed();
        let popped = tool_state.pop_temporary();
        assert!(!popped);
        assert!(!tool_state.just_changed());
    }

    #[test]
    fn test_tool_id_conversion() {
        // Test string conversion
        assert_eq!(ToolId::from_str("select"), Some(ToolId::Select));
        assert_eq!(ToolId::from_str("pen"), Some(ToolId::Pen));
        assert_eq!(ToolId::from_str("invalid"), None);

        // Test to string
        assert_eq!(ToolId::Select.as_str(), "select");
        assert_eq!(ToolId::Pen.as_str(), "pen");

        // Test display name
        assert_eq!(ToolId::Select.name(), "Select");
        assert_eq!(ToolId::Pen.name(), "Pen");
    }

    #[test]
    fn test_tool_is_active() {
        let mut tool_state = ToolState::default();

        assert!(tool_state.is_active(ToolId::Select));
        assert!(!tool_state.is_active(ToolId::Pen));

        tool_state.activate(ToolId::Pen);
        assert!(!tool_state.is_active(ToolId::Select));
        assert!(tool_state.is_active(ToolId::Pen));
    }
}
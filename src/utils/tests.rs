#![allow(clippy::assertions_on_constants)]

#[cfg(test)]
mod ufo_tests {
    use crate::data::ufo;

    #[test]
    fn test_load_ufo_from_path() {
        let test_path = "assets/fonts/bezy-grotesk-regular.ufo";
        let result = ufo::load_ufo_from_path(test_path);

        assert!(result.is_ok(), "Failed to load UFO file");

        let ufo = result.unwrap();

        // Test basic font info
        let font_info = &ufo.font_info;

        assert_eq!(
            font_info.family_name.as_deref(),
            Some("Bezy Grotesk"),
            "Family name should match"
        );
        assert_eq!(
            font_info.style_name.as_deref(),
            Some("Regular"),
            "Style name should match"
        );
    }
}

#[cfg(test)]
mod workspace_tests {
    use crate::core::state::AppState;
    use crate::data::ufo;
    use std::path::PathBuf;

    #[test]
    fn test_workspace_loads_ufo() {
        // First load the UFO file
        let test_path = "assets/fonts/bezy-grotesk-regular.ufo";
        let _ufo = ufo::load_ufo_from_path(test_path)
            .expect("Failed to load UFO file");

        // Create a new app state and load the font
        let mut app_state = AppState::default();
        let path = PathBuf::from(test_path);

        // Load the font into app state
        app_state
            .load_font_from_path(path)
            .expect("Failed to load font into app state");

        // Verify the workspace state
        assert_eq!(
            app_state.workspace.info.family_name, "Bezy Grotesk",
            "Workspace family name should match"
        );
        assert_eq!(
            app_state.workspace.info.style_name, "Regular",
            "Workspace style name should match"
        );

        // Test that the display name is correct
        assert_eq!(
            app_state.get_font_display_name(),
            "Bezy Grotesk Regular",
            "App state should display correct font name"
        );
    }
}

#[cfg(test)]
mod nudge_tests {
    use crate::core::settings::BezySettings;
    use crate::editing::selection::nudge::{EditEvent, NudgeState};

    #[test]
    fn test_nudge_amounts() {
        // Test that nudge amounts are reasonable
        let settings = BezySettings::default();
        assert!(
            settings.nudge.default > 0.0,
            "Default nudge amount should be positive"
        );
        assert!(
            settings.nudge.shift > settings.nudge.default,
            "Shift nudge should be larger than default"
        );
        assert!(
            settings.nudge.cmd > settings.nudge.shift,
            "Cmd nudge should be larger than shift"
        );
    }

    #[test]
    fn test_nudge_state_default() {
        let nudge_state = NudgeState::default();
        assert!(
            !nudge_state.is_nudging,
            "Default nudge state should not be nudging"
        );
        assert_eq!(
            nudge_state.last_nudge_time, 0.0,
            "Default last nudge time should be 0"
        );
    }

    #[test]
    fn test_edit_event_creation() {
        use crate::editing::edit_type::EditType;

        let event = EditEvent {
            edit_type: EditType::NudgeLeft,
        };

        assert!(
            matches!(event.edit_type, EditType::NudgeLeft),
            "Edit event should have correct type"
        );
    }
}

// Add more test modules here as needed, for example:
// mod grid_tests { ... }
// mod font_info_tests { ... }
// etc.

#![allow(clippy::assertions_on_constants)]

#[cfg(test)]
mod ufo_tests {
    use crate::data::ufo;

    #[test]
    #[ignore = "Requires test UFO file to be provided - skipped as test fixtures were removed"]
    fn test_load_ufo_from_path() {
        // This test requires a test UFO file that was removed from the repository
        // To run this test, provide a UFO file path via environment variable or test fixtures
        let test_path = std::env::var("TEST_UFO_PATH")
            .unwrap_or_else(|_| "path/to/test.ufo".to_string());

        // Skip test if file doesn't exist
        if !std::path::Path::new(&test_path).exists() {
            println!("Test UFO not found at {}, skipping test", test_path);
            return;
        }

        let result = ufo::load_ufo_from_path(&test_path);
        assert!(result.is_ok(), "Failed to load UFO file");

        let ufo = result.unwrap();
        // Test basic font info exists (don't check specific values)
        let font_info = &ufo.font_info;
        assert!(font_info.family_name.is_some(), "Should have family name");
    }
}

#[cfg(test)]
mod workspace_tests {
    use crate::core::state::AppState;
    use crate::data::ufo;
    use std::path::PathBuf;

    #[test]
    #[ignore = "Requires test UFO file to be provided - skipped as test fixtures were removed"]
    fn test_workspace_loads_ufo() {
        // This test requires a test UFO file that was removed from the repository
        let test_path = std::env::var("TEST_UFO_PATH")
            .unwrap_or_else(|_| "path/to/test.ufo".to_string());

        // Skip test if file doesn't exist
        if !std::path::Path::new(&test_path).exists() {
            println!("Test UFO not found at {}, skipping test", test_path);
            return;
        }

        // First load the UFO file
        let _ufo = ufo::load_ufo_from_path(&test_path).expect("Failed to load UFO file");

        // Create a new app state and load the font
        let mut app_state = AppState::default();
        let path = PathBuf::from(&test_path);

        // Load the font into app state
        app_state
            .load_font_from_path(path)
            .expect("Failed to load font into app state");

        // Verify the workspace state has been populated
        assert!(!app_state.workspace.info.family_name.is_empty(),
                "Workspace should have a family name");
        assert!(!app_state.workspace.info.style_name.is_empty(),
                "Workspace should have a style name");

        // Test that the display name is not empty
        assert!(!app_state.get_font_display_name().is_empty(),
                "App state should have a display name");
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

        let event = EditEvent {
        };

        assert!(
            "Edit event should have correct type"
        );
    }
}

// Add more test modules here as needed, for example:
// mod grid_tests { ... }
// mod font_info_tests { ... }
// etc.

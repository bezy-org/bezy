//! AI Tool - Advanced AI-powered font editing features
//!
//! The AI tool provides intelligent assistance for font editing tasks including:
//! - Automatic kerning adjustments
//! - Language support expansion
//! - Optical corrections and harmonization
//! - Weight consistency fixes
//! - Curve smoothness optimization

use super::{EditTool, ToolInfo};
use bevy::prelude::*;

/// AI operations that can be performed
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Resource)]
pub enum AiOperation {
    #[default]
    Kerning,
    LanguageSupport,
    OpticalAdjustment,
    WeightFix,
    CurveSmoothing,
}

impl AiOperation {
    /// Get the icon for each AI operation
    pub fn get_icon(&self) -> &'static str {
        match self {
            AiOperation::Kerning => "\u{E023}",
            AiOperation::LanguageSupport => "\u{E024}",
            AiOperation::OpticalAdjustment => "\u{E025}",
            AiOperation::WeightFix => "\u{E026}",
            AiOperation::CurveSmoothing => "\u{E027}",
        }
    }

    /// Get a human-readable name for this operation
    pub fn display_name(&self) -> &'static str {
        match self {
            AiOperation::Kerning => "Auto Kerning",
            AiOperation::LanguageSupport => "Language Support",
            AiOperation::OpticalAdjustment => "Optical Correction",
            AiOperation::WeightFix => "Weight Consistency",
            AiOperation::CurveSmoothing => "Curve Smoothing",
        }
    }

    /// Get description for tooltip
    pub fn description(&self) -> &'static str {
        match self {
            AiOperation::Kerning => "Automatically adjust character spacing for optimal readability",
            AiOperation::LanguageSupport => "Expand font support for additional languages and scripts", 
            AiOperation::OpticalAdjustment => "Make optical corrections to improve visual harmony",
            AiOperation::WeightFix => "Fix weight inconsistencies across characters",
            AiOperation::CurveSmoothing => "Optimize curve smoothness and mathematical precision",
        }
    }
}

/// Component to mark AI submenu buttons
#[derive(Component)]
pub struct AiSubMenuButton;

/// Component to associate a button with its AI operation
#[derive(Component)]
pub struct AiOperationButton {
    pub operation: AiOperation,
}

/// Resource to track the current AI operation
#[derive(Resource, Default)]
pub struct CurrentAiOperation(pub AiOperation);

/// Resource to track if AI mode is active
#[derive(Resource, Default)]
pub struct AiModeActive(pub bool);

/// The AI tool implementation
pub struct AiTool;

impl EditTool for AiTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "ai",
            display_name: "AI",
            icon: "\u{E022}", // AI brain icon
            tooltip: "AI-powered font editing tools",
            shortcut: Some(KeyCode::KeyA),
        }
    }

    fn on_activate(&mut self, _commands: &mut Commands) {
        info!("AI tool activated - Enhanced features:");
        info!("• Auto Kerning: Intelligent character spacing");
        info!("• Language Support: Expand script coverage");
        info!("• Optical Correction: Visual harmony adjustments");
        info!("• Weight Consistency: Fix thickness variations");
        info!("• Curve Smoothing: Mathematical optimization");
        info!("• Tab to switch between AI operations");
    }

    fn on_deactivate(&mut self, _commands: &mut Commands) {
        info!("AI tool deactivated");
    }
}

/// Plugin for the AI tool
pub struct AiToolPlugin;

impl Plugin for AiToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AiModeActive>()
            .init_resource::<CurrentAiOperation>()
            .add_systems(PostStartup, spawn_ai_submenu)
            .add_systems(
                Update,
                (
                    update_ai_mode_active,
                    handle_ai_tool_shortcuts,
                    handle_ai_operation_selection,
                    toggle_ai_submenu_visibility,
                    execute_ai_operations,
                )
                    .chain(),
            );
    }
}

// --------- UI Systems -----------

/// Helper function to spawn a single AI operation button using the unified system
fn spawn_ai_operation_button(
    parent: &mut ChildSpawnerCommands,
    operation: AiOperation,
    asset_server: &Res<AssetServer>,
    theme: &Res<crate::ui::themes::CurrentTheme>,
) {
    // Use the unified toolbar button creation system for consistent styling
    crate::ui::toolbars::edit_mode_toolbar::create_toolbar_button(
        parent,
        operation.get_icon(),
        (AiSubMenuButton, AiOperationButton { operation }),
        asset_server,
        theme,
    );
}

pub fn spawn_ai_submenu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    theme: Res<crate::ui::themes::CurrentTheme>,
) {
    use crate::ui::theme::*;

    let operations = [
        AiOperation::Kerning,
        AiOperation::LanguageSupport,
        AiOperation::OpticalAdjustment,
        AiOperation::WeightFix,
        AiOperation::CurveSmoothing,
    ];

    // Create the parent submenu node (positioned to the right of the main toolbar)
    let submenu_node = Node {
        position_type: PositionType::Absolute,
        top: Val::Px(TOOLBAR_CONTAINER_MARGIN + 74.0),
        left: Val::Px(TOOLBAR_CONTAINER_MARGIN),
        flex_direction: FlexDirection::Row,
        padding: UiRect::all(Val::Px(TOOLBAR_PADDING)),
        margin: UiRect::all(Val::ZERO),
        row_gap: Val::Px(TOOLBAR_PADDING),
        display: Display::None,
        ..default()
    };

    // Spawn the submenu with all AI operation buttons
    commands
        .spawn((submenu_node, Name::new("AiSubMenu")))
        .with_children(|parent| {
            for operation in operations {
                spawn_ai_operation_button(parent, operation, &asset_server, &theme);
            }
        });

    info!("Spawned AI submenu with {} operations", operations.len());
}

pub fn handle_ai_operation_selection(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &AiOperationButton,
            Entity,
        ),
        With<AiSubMenuButton>,
    >,
    mut current_operation: ResMut<CurrentAiOperation>,
    children_query: Query<&Children>,
    mut text_query: Query<&mut TextColor>,
) {
    for (interaction, mut color, mut border_color, operation_button, entity) in
        &mut interaction_query
    {
        let is_current_operation = current_operation.0 == operation_button.operation;

        if *interaction == Interaction::Pressed && !is_current_operation {
            current_operation.0 = operation_button.operation;
            info!("Switched to AI operation: {:?}", operation_button.operation);
        }

        // Use the unified button color system for consistent appearance with main toolbar
        crate::ui::toolbars::edit_mode_toolbar::update_toolbar_button_colors(
            *interaction,
            is_current_operation,
            &mut color,
            &mut border_color,
        );
        
        // Use the unified text color system for consistent icon colors with main toolbar
        crate::ui::toolbars::edit_mode_toolbar::update_toolbar_button_text_colors(
            entity,
            is_current_operation,
            &children_query,
            &mut text_query,
        );
    }
}

pub fn toggle_ai_submenu_visibility(
    mut submenu_query: Query<(&mut Node, &Name)>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
) {
    let is_ai_tool_active = current_tool.get_current() == Some("ai");
    for (mut style, name) in submenu_query.iter_mut() {
        if name.as_str() == "AiSubMenu" {
            style.display = if is_ai_tool_active {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
}

pub fn update_ai_mode_active(
    mut ai_mode_active: ResMut<AiModeActive>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
) {
    let is_ai_mode = current_tool.get_current() == Some("ai");
    if ai_mode_active.0 != is_ai_mode {
        ai_mode_active.0 = is_ai_mode;
        debug!("AI mode active state changed: {}", is_ai_mode);
    }
}

// -- Input and AI Operation Logic --

pub fn handle_ai_tool_shortcuts(
    mut keyboard_input: ResMut<ButtonInput<KeyCode>>,
    mut current_tool: ResMut<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    mut current_operation: ResMut<CurrentAiOperation>,
    text_mode_active: Option<Res<crate::ui::toolbars::edit_mode_toolbar::text::TextModeActive>>,
    current_text_placement_mode: Option<Res<crate::ui::toolbars::edit_mode_toolbar::text::CurrentTextPlacementMode>>,
) {
    // Check if single-char hotkeys should be disabled for text input
    let should_disable = crate::ui::toolbars::edit_mode_toolbar::keyboard_utils::should_disable_single_char_hotkeys(
        text_mode_active.as_ref(),
        current_text_placement_mode.as_ref(),
    );
    
    // Activate AI tool with 'A' key (but not when in text insert mode)
    if keyboard_input.just_pressed(KeyCode::KeyA)
        && current_tool.get_current() != Some("ai")
        && !should_disable
    {
        current_tool.switch_to("ai");
        info!("Activated AI tool via keyboard shortcut");
        keyboard_input.clear_just_pressed(KeyCode::KeyA);
    }

    // Cycle through AI operations with Tab when AI tool is active
    if current_tool.get_current() == Some("ai")
        && keyboard_input.just_pressed(KeyCode::Tab)
    {
        let new_operation = match current_operation.0 {
            AiOperation::Kerning => AiOperation::LanguageSupport,
            AiOperation::LanguageSupport => AiOperation::OpticalAdjustment,
            AiOperation::OpticalAdjustment => AiOperation::WeightFix,
            AiOperation::WeightFix => AiOperation::CurveSmoothing,
            AiOperation::CurveSmoothing => AiOperation::Kerning,
        };
        current_operation.0 = new_operation;
        info!("Switched AI operation to: {:?}", new_operation);
        keyboard_input.clear_just_pressed(KeyCode::Tab);
    }

    // Show help with F1
    if current_tool.get_current() == Some("ai")
        && keyboard_input.just_pressed(KeyCode::F1)
    {
        info!("=== AI TOOL HELP ===");
        info!("A - Activate AI tool");
        info!("Tab - Switch between AI operations");
        info!("Enter - Execute current AI operation");
        info!("AI OPERATIONS:");
        info!("  • Auto Kerning - Intelligent character spacing");
        info!("  • Language Support - Expand script coverage");
        info!("  • Optical Correction - Visual harmony adjustments");
        info!("  • Weight Consistency - Fix thickness variations");
        info!("  • Curve Smoothing - Mathematical optimization");
        info!("Escape - Exit AI tool");
        info!("F1 - Show this help");
        info!("==================");
    }

    // Exit AI tool with Escape
    if current_tool.get_current() == Some("ai")
        && keyboard_input.just_pressed(KeyCode::Escape)
    {
        if let Some(previous_tool) = current_tool.get_previous() {
            current_tool.switch_to(previous_tool);
            info!(
                "Exited AI tool via Escape key, returned to: {}",
                previous_tool
            );
        } else {
            current_tool.switch_to("select");
            info!("Exited AI tool via Escape key, returned to select tool");
        }
    }
}

pub fn execute_ai_operations(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_tool: Res<crate::ui::toolbars::edit_mode_toolbar::CurrentTool>,
    current_operation: Res<CurrentAiOperation>,
    ai_mode_active: Res<AiModeActive>,
) {
    if !ai_mode_active.0 || current_tool.get_current() != Some("ai") {
        return;
    }

    // Execute AI operation with Enter key
    if keyboard_input.just_pressed(KeyCode::Enter) {
        match current_operation.0 {
            AiOperation::Kerning => {
                info!("🤖 Executing Auto Kerning...");
                info!("   Analyzing character pairs for optimal spacing");
                info!("   [PLACEHOLDER] This feature will be implemented later");
            }
            AiOperation::LanguageSupport => {
                info!("🤖 Executing Language Support Expansion...");
                info!("   Analyzing missing glyphs for target languages");
                info!("   [PLACEHOLDER] This feature will be implemented later");
            }
            AiOperation::OpticalAdjustment => {
                info!("🤖 Executing Optical Corrections...");
                info!("   Analyzing visual balance and harmony");
                info!("   [PLACEHOLDER] This feature will be implemented later");
            }
            AiOperation::WeightFix => {
                info!("🤖 Executing Weight Consistency Fixes...");
                info!("   Analyzing stroke thickness variations");
                info!("   [PLACEHOLDER] This feature will be implemented later");
            }
            AiOperation::CurveSmoothing => {
                info!("🤖 Executing Curve Smoothing...");
                info!("   Optimizing mathematical precision of curves");
                info!("   [PLACEHOLDER] This feature will be implemented later");
            }
        }
    }
}
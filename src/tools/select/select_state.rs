//! State management for the select tool

use bevy::prelude::*;

/// State for marquee selection (drag to select)
#[derive(Resource, Default, Debug)]
pub struct SelectToolDragState {
    pub is_dragging: bool,
    pub start_position: Vec2,
    pub current_position: Vec2,
    pub marquee_entity: Option<Entity>,
}

impl SelectToolDragState {
    pub fn start_drag(&mut self, position: Vec2) {
        self.is_dragging = true;
        self.start_position = position;
        self.current_position = position;
        debug!("🔍 SELECT: Started marquee selection at {:?}", position);
    }

    pub fn update_drag(&mut self, position: Vec2) {
        if self.is_dragging {
            self.current_position = position;
        }
    }

    pub fn end_drag(&mut self) {
        if self.is_dragging {
            debug!("🔍 SELECT: Ended marquee selection");
            self.is_dragging = false;
        }
    }

    pub fn get_bounds(&self) -> (Vec2, Vec2) {
        let min_x = self.start_position.x.min(self.current_position.x);
        let max_x = self.start_position.x.max(self.current_position.x);
        let min_y = self.start_position.y.min(self.current_position.y);
        let max_y = self.start_position.y.max(self.current_position.y);

        (Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        let (min, max) = self.get_bounds();
        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    }
}
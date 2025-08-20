//! Design space coordinate system for font editing
//!
//! This module provides the core coordinate types and transformations for the font editor.
//! Design space is the fixed coordinate system where glyphs, guides, and other entities
//! are described. When drawing to the screen or handling mouse input, we need to translate
//! from 'screen space' to design space, taking into account things like the current
//! scroll offset and zoom level.

use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use bevy::math::Vec2;
use bevy::prelude::*;

/// A point in design space.
///
/// This type represents a point in the canonical font coordinate system.
/// The origin (0,0) is at the intersection of the baseline and the left sidebearing.
/// Ascenders are in positive Y, and descenders are in negative Y.
#[derive(Clone, Copy, Component, PartialEq)]
pub struct DPoint {
    pub x: f32,
    pub y: f32,
}

/// A vector in design space, used for nudging & dragging
#[derive(Debug, Clone, Copy, Component, PartialEq)]
pub struct DVec2 {
    pub x: f32,
    pub y: f32,
}

impl DPoint {
    pub const ZERO: DPoint = DPoint { x: 0.0, y: 0.0 };

    /// Create a new `DPoint` with the given coordinates.
    /// Should only be used with inputs already in design space, such as when
    /// loaded from file.
    pub(crate) fn new(x: f32, y: f32) -> DPoint {
        DPoint { x, y }
    }

    /// Create a new `DPoint` from a `Vec2` in design space. This should only
    /// be used to convert back to a `DPoint` after using `Vec2` to do vector
    /// math in design space.
    pub fn from_raw(point: impl Into<Vec2>) -> DPoint {
        let point = point.into();
        DPoint::new(point.x, point.y)
    }

    /// Convert a design point directly to a Vec2, without taking screen geometry
    /// into account.
    pub fn to_raw(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Convert this `DPoint` to a `DVec2`.
    pub fn to_dvec2(self) -> DVec2 {
        let DPoint { x, y } = self;
        DVec2 { x, y }
    }

    pub fn lerp(self, other: DPoint, t: f32) -> DPoint {
        DPoint::from_raw(Vec2::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        ))
    }
}

impl DVec2 {
    pub const ZERO: DVec2 = DVec2 { x: 0.0, y: 0.0 };

    fn new(x: f32, y: f32) -> DVec2 {
        DVec2 { x, y }
    }

    pub fn from_raw(vec2: impl Into<Vec2>) -> DVec2 {
        let vec2 = vec2.into();
        DVec2::new(vec2.x, vec2.y)
    }

    #[inline]
    pub(super) fn to_raw(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.to_raw().length()
    }

    /// The vector snapped to the closest axis.
    pub fn axis_locked(self) -> DVec2 {
        if self.x.abs() > self.y.abs() {
            DVec2::new(self.x, 0.0)
        } else {
            DVec2::new(0.0, self.y)
        }
    }
}

impl Add<DVec2> for DPoint {
    type Output = DPoint;

    fn add(self, other: DVec2) -> Self {
        DPoint::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub<DVec2> for DPoint {
    type Output = DPoint;

    fn sub(self, other: DVec2) -> Self {
        DPoint::new(self.x - other.x, self.y - other.y)
    }
}

impl Sub<DPoint> for DPoint {
    type Output = DVec2;

    fn sub(self, other: DPoint) -> DVec2 {
        DVec2::new(self.x - other.x, self.y - other.y)
    }
}

impl Add for DVec2 {
    type Output = DVec2;

    fn add(self, other: DVec2) -> DVec2 {
        DVec2::new(self.x + other.x, self.y + other.y)
    }
}

impl AddAssign for DVec2 {
    fn add_assign(&mut self, rhs: DVec2) {
        *self = *self + rhs;
    }
}

impl Sub for DVec2 {
    type Output = DVec2;

    fn sub(self, other: DVec2) -> DVec2 {
        DVec2::new(self.x - other.x, self.y - other.y)
    }
}

impl SubAssign for DVec2 {
    fn sub_assign(&mut self, rhs: DVec2) {
        *self = *self - rhs;
    }
}

impl From<(f32, f32)> for DPoint {
    fn from(src: (f32, f32)) -> DPoint {
        DPoint { x: src.0, y: src.1 }
    }
}

impl From<Vec2> for DPoint {
    fn from(src: Vec2) -> DPoint {
        DPoint::new(src.x, src.y)
    }
}

impl fmt::Debug for DPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DPoint<{} {}>", self.x, self.y)
    }
}

impl fmt::Display for DPoint {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "x: {:.1}, y: {:.1}", self.x, self.y)
    }
}

impl fmt::Display for DVec2 {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "x: {:.1}, y: {:.1}", self.x, self.y)
    }
}

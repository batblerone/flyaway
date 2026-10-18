//! The two hit tests the game needs. Everything in here works in plain world
//! coordinates, so none of it cares about sprites or scale.

use bevy::math::{Mat2, Vec2};

pub fn circles_touch(a: Vec2, a_radius: f32, b: Vec2, b_radius: f32) -> bool {
    a.distance_squared(b) < (a_radius + b_radius).powi(2)
}

/// Does a circle overlap a box that's been rotated by `angle` radians?
///
/// Rather than rotating the box, we rotate the circle's centre into the box's
/// own frame, where it's axis-aligned again. Then it's the usual trick: clamp
/// the centre to the box to find the nearest point on it, and see if that
/// point is inside the circle.
pub fn circle_hits_box(
    circle: Vec2,
    radius: f32,
    box_centre: Vec2,
    half_size: Vec2,
    angle: f32,
) -> bool {
    let local = Mat2::from_angle(-angle) * (circle - box_centre);
    let nearest = local.clamp(-half_size, half_size);
    local.distance_squared(nearest) < radius * radius
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn circles() {
        assert!(circles_touch(Vec2::ZERO, 32.0, Vec2::new(60.0, 0.0), 32.0));
        assert!(!circles_touch(Vec2::ZERO, 32.0, Vec2::new(70.0, 0.0), 32.0));
    }

    #[test]
    fn rotated_box() {
        // A long flat box, and a small circle sitting 50px above its middle.
        let half = Vec2::new(100.0, 10.0);
        let circle = Vec2::new(0.0, 50.0);
        assert!(!circle_hits_box(circle, 5.0, Vec2::ZERO, half, 0.0));
        // Stand the box on its end and the circle is now inside it.
        assert!(circle_hits_box(circle, 5.0, Vec2::ZERO, half, FRAC_PI_2));
    }

    #[test]
    fn box_corners_are_respected() {
        // Just off the corner diagonally: inside the bounding square of the
        // circle, but not actually touching the box.
        let half = Vec2::new(10.0, 10.0);
        assert!(!circle_hits_box(
            Vec2::new(14.0, 14.0),
            5.0,
            Vec2::ZERO,
            half,
            0.0
        ));
        assert!(circle_hits_box(
            Vec2::new(13.0, 13.0),
            5.0,
            Vec2::ZERO,
            half,
            0.0
        ));
    }
}

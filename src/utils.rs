use bevy::prelude::*;

pub fn create_movement_vector(orientation: Quat, velocity: f32) -> Vec2 {
    let rotation = orientation.to_axis_angle().1;
    Vec2 {
        x: rotation.cos() * velocity,
        y: rotation.sin() * velocity,
    }
}

pub fn transform_vector(position: &mut Vec2, orientation: Quat, velocity: f32) {
    *position += create_movement_vector(orientation, velocity);
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use bevy::prelude::*;

    use super::*;

    #[test]
    fn create_movement_vector_test() {
        assert_eq!(
            create_movement_vector(Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), 0.0), 1.0),
            Vec2::new(1.0, 0.0)
        );
        let mut mv =
            create_movement_vector(Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), PI), 1.0);
        assert!((mv.x - -1.0).abs() < 0.000001);
        assert!((mv.y - -0.0).abs() < 0.000001);
        mv = create_movement_vector(
            Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), PI / 2.0),
            1.0,
        );
        assert!((mv.x - -0.0).abs() < 0.000001);
        assert!((mv.y - 1.0).abs() < 0.000001);
        mv = create_movement_vector(
            Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), PI / 2.0 * 3.0),
            1.0,
        );
        assert!((mv.x - -0.0).abs() < 0.000001);
        assert!((mv.y - -1.0).abs() < 0.000001);
        mv = create_movement_vector(
            Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), PI / 4.0 * 5.0),
            1.0,
        );
        assert!((mv.x - -(0.5_f32.powf(0.5))).abs() < 0.000001);
        assert!((mv.y - -(0.5_f32.powf(0.5))).abs() < 0.000001);
    }
    fn transform_vector_test() {
        let mut orig = Vec2::new(10., 10.);
        let t = transform_vector(
            &mut orig,
            Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), PI / 4.0 * 5.0),
            1.0,
        );
        assert!((orig.x - -(0.5_f32.powf(0.5)) + 10.).abs() < 0.000001);
        assert!((orig.y - -(0.5_f32.powf(0.5)) + 10.).abs() < 0.000001);
    }
}

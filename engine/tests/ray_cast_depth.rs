use engine::{ray_cast, Vec2f};

/// A square grid of floor enclosed by a one-tile wall border.
fn walled_grid(size: usize) -> impl Fn(Vec2f) -> Option<()> {
    let last = (size - 1) as f32;
    move |point: Vec2f| {
        let is_wall = point.x <= 0.0 || point.y <= 0.0 || point.x >= last || point.y >= last;
        is_wall.then_some(())
    }
}

/// Fired from just inside one wall, straight at the opposite one.
fn cast_across(size: usize, max_steps: usize) -> Option<()> {
    let grid = walled_grid(size);
    ray_cast(Vec2f::new(1.5, 1.5), 0.0, max_steps, &grid).value
}

#[test]
fn a_bound_matching_the_grid_reaches_the_far_wall() {
    assert!(cast_across(50, 50).is_some());
    assert!(cast_across(120, 120).is_some());
}

#[test]
fn a_bound_smaller_than_the_grid_misses_silently() {
    // the failure this guards against: no error, just "no hit", which shows
    // up as invisible distant walls and shots passing through them
    assert!(cast_across(50, 20).is_none());

    // a grid grown past the step limit that used to be a shared constant
    assert!(cast_across(120, 50).is_none());
}

#[test]
fn a_bound_of_zero_takes_no_step_at_all() {
    assert!(cast_across(50, 0).is_none());
}

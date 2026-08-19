//! Pins the interleaved walk to the two-sweep algorithm it replaced: for the
//! same ray they must agree on what was hit, how far away it is and where
//! across the tile it landed.

use engine::{ray_cast, Float, Vec2f, RAY_CASTER_TOL};

const SIZE: usize = 40;

/// Scattered walls plus a solid border, so both sweeps have work to do.
fn grid(x: Float, y: Float) -> Option<u32> {
    if !x.is_finite() || !y.is_finite() || x < 0.0 || y < 0.0 {
        return None;
    }
    let (col, row) = (x as usize, y as usize);
    if col >= SIZE || row >= SIZE {
        return None;
    }
    let border = col == 0 || row == 0 || col == SIZE - 1 || row == SIZE - 1;
    // a sparse, irregular scatter of blocks
    let block = (col * 7 + row * 13) % 23 == 0 && col > 2 && row > 2;
    (border || block).then_some((row * SIZE + col) as u32)
}

/// The algorithm as it stood before: each axis cast independently to its own
/// wall, then the nearer of the two results kept.
fn reference(pos: Vec2f, angle: Float, max_steps: usize) -> (Option<u32>, Float, Float) {
    let check = |p: Vec2f| grid(p.x, p.y);
    let (sin, cos) = (angle.sin(), angle.cos());
    let tile = pos.floor();

    let sweep = |mut point: Vec2f, step: Vec2f, mut depth: Float, depth_step: Float| {
        let mut val = None;
        for _ in 0..max_steps {
            val = check(point);
            if val.is_some() {
                break;
            }
            point += step;
            depth += depth_step;
        }
        (val, depth, point)
    };

    let (h_val, h_depth, h_point) = {
        let (y, dy) = if sin > 0.0 {
            (tile.y + 1.0, 1.0)
        } else {
            (tile.y - RAY_CASTER_TOL, -1.0)
        };
        let depth = (y - pos.y) / sin;
        let depth_step = dy / sin;
        let point = Vec2f::new(pos.x + depth * cos, y);
        sweep(point, Vec2f::new(depth_step * cos, dy), depth, depth_step)
    };
    let (v_val, v_depth, v_point) = {
        let (x, dx) = if cos > 0.0 {
            (tile.x + 1.0, 1.0)
        } else {
            (tile.x - RAY_CASTER_TOL, -1.0)
        };
        let depth = (x - pos.x) / cos;
        let depth_step = dx / cos;
        let point = Vec2f::new(x, pos.y + depth * sin);
        sweep(point, Vec2f::new(dx, depth_step * sin), depth, depth_step)
    };

    let vertical = {
        let vy = v_point.y % 1.0;
        (v_val, v_depth, if cos > 0.0 { vy } else { 1.0 - vy })
    };
    let horizontal = {
        let hx = h_point.x % 1.0;
        (h_val, h_depth, if sin > 0.0 { 1.0 - hx } else { hx })
    };
    if sin == 0.0 {
        return vertical;
    }
    if cos == 0.0 {
        return horizontal;
    }
    if v_depth < h_depth {
        vertical
    } else {
        horizontal
    }
}

#[test]
fn matches_the_two_sweep_algorithm_it_replaced() {
    let check = |p: Vec2f| grid(p.x, p.y);
    let mut seed: u64 = 0x5eed;
    let mut compared = 0;
    let mut hits = 0;

    for _ in 0..4000 {
        // deterministic pseudo-random positions and angles
        let mut next = || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((seed >> 33) as f64 / (u32::MAX as f64)) as Float
        };
        let pos = Vec2f::new(
            2.0 + next() * (SIZE as Float - 4.0),
            2.0 + next() * (SIZE as Float - 4.0),
        );
        if grid(pos.x, pos.y).is_some() {
            continue; // starting inside a wall is not a case the game produces
        }
        let angle = (next() - 0.5) * 8.0;

        let got = ray_cast(pos, angle, SIZE, &check);
        let (want_value, want_depth, want_offset) = reference(pos, angle, SIZE);

        compared += 1;
        assert_eq!(
            got.value, want_value,
            "different hit at {pos} angle {angle}"
        );
        if got.value.is_some() {
            hits += 1;
            assert!(
                (got.depth - want_depth).abs() <= 1e-3 * want_depth.abs().max(1.0),
                "depth {} vs {} at {pos} angle {angle}",
                got.depth,
                want_depth
            );
            assert!(
                (got.offset - want_offset).abs() <= 1e-3,
                "offset {} vs {} at {pos} angle {angle}",
                got.offset,
                want_offset
            );
        }
    }
    assert!(compared > 3000, "only {compared} rays compared");
    assert!(hits > 3000, "only {hits} rays hit a wall");
}

#[test]
fn a_check_that_answers_infinities_cannot_break_an_axis_aligned_ray() {
    // A ray parallel to one axis never crosses that axis's grid lines, and
    // its sweep would otherwise walk a line of infinities. A `check` written
    // as a simple `x <= 0.0` bounds test calls that a hit -- at infinite
    // depth, which must not be allowed to cut the other sweep short.
    let permissive =
        |p: Vec2f| (p.x <= 0.0 || p.x >= 39.0 || p.y <= 0.0 || p.y >= 39.0).then_some(1u32);
    let pos = Vec2f::new(1.5, 1.5);
    for angle in [
        0.0,
        std::f32::consts::FRAC_PI_2,
        std::f32::consts::PI,
        -std::f32::consts::FRAC_PI_2,
    ] {
        let result = ray_cast(pos, angle, SIZE, &permissive);
        assert!(result.value.is_some(), "angle {angle} found nothing");
        assert!(
            result.depth.is_finite(),
            "angle {angle} gave depth {}",
            result.depth
        );
        assert!(
            result.depth > 0.0,
            "angle {angle} gave depth {}",
            result.depth
        );
    }
}

#[test]
fn axis_aligned_rays_still_hit() {
    let check = |p: Vec2f| grid(p.x, p.y);
    let pos = Vec2f::new(1.5, 1.5);
    // sin == 0 and cos == 0 are the cases the sweeps have to opt out of
    for angle in [0.0, std::f32::consts::FRAC_PI_2, std::f32::consts::PI] {
        let result = ray_cast(pos, angle, SIZE, &check);
        assert!(result.value.is_some(), "angle {angle} found nothing");
        assert!(
            result.depth.is_finite(),
            "angle {angle} gave depth {}",
            result.depth
        );
    }
}

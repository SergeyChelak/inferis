use crate::{Float, Vec2f};

pub const RAY_CASTER_TOL: Float = 1e-5;

pub struct RayCastResult<T> {
    pub value: Option<T>,
    pub depth: Float,
    pub offset: Float,
}

/// Casts a ray from `pos` until `check` reports a hit, giving up after
/// `max_steps` tile steps.
///
/// A ray advances exactly one row (for the horizontal sweep) or one column
/// (for the vertical one) per step, so a caller casting over a grid must
/// allow at least the larger of that grid's two dimensions. A smaller bound
/// does not fail loudly: the ray simply reports no hit, which reads as
/// distant walls turning invisible and shots passing through them. Derive
/// the value from the grid being cast over rather than fixing it.
pub fn ray_cast<T>(
    pos: Vec2f,
    ray_angle: Float,
    max_steps: usize,
    check: &impl Fn(Vec2f) -> Option<T>,
) -> RayCastResult<T> {
    let sin = ray_angle.sin();
    let cos = ray_angle.cos();
    let tile = pos.floor();
    let (h_val, h_depth, h_vec) = cast_horizontal(pos, tile, sin, cos, max_steps, check);
    let (v_val, v_depth, v_vec) = cast_vertical(pos, tile, sin, cos, max_steps, check);
    let vertical_result = {
        let vertical_y = v_vec.y % 1.0;
        let offset = if cos > 0.0 {
            vertical_y
        } else {
            1.0 - vertical_y
        };
        RayCastResult {
            value: v_val,
            depth: v_depth,
            offset,
        }
    };
    let horizontal_result = {
        let horizontal_x = h_vec.x % 1.0;
        let offset = if sin > 0.0 {
            1.0 - horizontal_x
        } else {
            horizontal_x
        };
        RayCastResult {
            value: h_val,
            depth: h_depth,
            offset,
        }
    };
    if sin == 0.0 {
        return vertical_result;
    }
    if cos == 0.0 {
        return horizontal_result;
    }
    if v_depth < h_depth {
        vertical_result
    } else {
        horizontal_result
    }
}

fn cast_horizontal<T>(
    pos: Vec2f,
    tile: Vec2f,
    sin: Float,
    cos: Float,
    max_steps: usize,
    check: impl Fn(Vec2f) -> Option<T>,
) -> (Option<T>, Float, Vec2f) {
    let (mut y, dy) = if sin > 0.0 {
        (tile.y + 1.0, 1.0)
    } else {
        (tile.y - RAY_CASTER_TOL, -1.0)
    };
    let mut depth = (y - pos.y) / sin;
    let mut x = pos.x + depth * cos;
    let depth_delta = dy / sin;
    let dx = depth_delta * cos;
    let mut val: Option<T> = None;
    for _ in 0..max_steps {
        val = check(Vec2f::new(x, y));
        if val.is_some() {
            break;
        }
        x += dx;
        y += dy;
        depth += depth_delta;
    }
    (val, depth, Vec2f::new(x, y))
}

fn cast_vertical<T>(
    pos: Vec2f,
    tile: Vec2f,
    sin: Float,
    cos: Float,
    max_steps: usize,
    check: impl Fn(Vec2f) -> Option<T>,
) -> (Option<T>, Float, Vec2f) {
    let (mut x, dx) = if cos > 0.0 {
        (tile.x + 1.0, 1.0)
    } else {
        (tile.x - RAY_CASTER_TOL, -1.0)
    };
    let mut depth = (x - pos.x) / cos;
    let mut y = pos.y + depth * sin;
    let depth_delta = dx / cos;
    let dy = depth_delta * sin;
    let mut val: Option<T> = None;
    for _ in 0..max_steps {
        val = check(Vec2f::new(x, y));
        if val.is_some() {
            break;
        }
        x += dx;
        y += dy;
        depth += depth_delta;
    }
    (val, depth, Vec2f::new(x, y))
}

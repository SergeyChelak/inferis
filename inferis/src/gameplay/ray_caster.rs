use engine::{Float, Vec2f};

const TOL: Float = 1e-5;
const MAX_DEPTH: usize = 50;

pub struct RayCastResult<T> {
    pub value: Option<T>,
    pub depth: Float,
    pub offset: Float,
}

pub fn ray_cast<T>(
    pos: Vec2f,
    ray_angle: Float,
    check: &impl Fn(Vec2f) -> Option<T>,
) -> RayCastResult<T> {
    let sin = ray_angle.sin();
    let cos = ray_angle.cos();
    let tile = pos.floor();
    let (h_val, h_depth, h_vec) = cast_horizontal(pos, tile, sin, cos, check);
    let (v_val, v_depth, v_vec) = cast_vertical(pos, tile, sin, cos, check);

    if v_depth < h_depth {
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
    } else {
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
    }
}

fn cast_horizontal<T>(
    pos: Vec2f,
    tile: Vec2f,
    sin: Float,
    cos: Float,
    check: impl Fn(Vec2f) -> Option<T>,
) -> (Option<T>, Float, Vec2f) {
    let (mut y, dy) = if sin > 0.0 {
        (tile.y + 1.0, 1.0)
    } else {
        (tile.y - TOL, -1.0)
    };
    let mut depth = (y - pos.y) / sin;
    let mut x = pos.x + depth * cos;
    let depth_delta = dy / sin;
    let dx = depth_delta * cos;
    let mut val: Option<T> = None;
    for _ in 0..MAX_DEPTH {
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
    check: impl Fn(Vec2f) -> Option<T>,
) -> (Option<T>, Float, Vec2f) {
    let (mut x, dx) = if cos > 0.0 {
        (tile.x + 1.0, 1.0)
    } else {
        (tile.x - TOL, -1.0)
    };
    let mut depth = (x - pos.x) / cos;
    let mut y = pos.y + depth * sin;
    let depth_delta = dx / cos;
    let dy = depth_delta * sin;
    let mut val: Option<T> = None;
    for _ in 0..MAX_DEPTH {
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

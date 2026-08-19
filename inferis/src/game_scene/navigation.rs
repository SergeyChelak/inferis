//! Grid navigation for the soldiers: where they can walk, and how to get
//! there.
//!
//! Everything here works on the maze's tile grid rather than on continuous
//! positions. One breadth-first flood out from a soldier answers all three
//! questions the AI asks -- somewhere to wander to, the way to a remembered
//! spot, and the nearest place the player cannot see -- so a soldier that
//! re-plans pays for the flood once.

use std::collections::{HashMap, VecDeque};

use engine::{ray_cast, Float, Vec2f, RAY_CASTER_TOL};

use super::{components::Maze, generator::matrix::Position};

/// The tile a continuous position falls in.
pub fn cell_at(point: Vec2f) -> Option<Position> {
    if point.x < 0.0 || point.y < 0.0 {
        return None;
    }
    Some(Position {
        row: point.y as usize,
        col: point.x as usize,
    })
}

/// The middle of a tile. Soldiers walk centre to centre so that their
/// bounding box clears the walls on either side.
pub fn cell_center(cell: Position) -> Vec2f {
    Vec2f::new(cell.col as Float + 0.5, cell.row as Float + 0.5)
}

/// Whether `from` can see `to`, using the same wall cast the renderer uses.
pub fn has_line_of_sight(maze: &Maze, from: Vec2f, to: Vec2f) -> bool {
    let vector = to - from;
    let distance = vector.length();
    if distance < RAY_CASTER_TOL {
        return true;
    }
    let angle = vector.y.atan2(vector.x);
    let wall = |point: Vec2f| maze.is_wall(point).then_some(());
    let result = ray_cast(from, angle, maze.ray_cast_steps(), &wall);
    match result.value {
        // a wall counts only if it stands between the two points
        Some(()) => result.depth >= distance,
        None => true,
    }
}

fn is_walkable(maze: &Maze, cell: Position) -> bool {
    !maze.is_wall(cell_center(cell))
}

/// Walkable tiles reachable from an origin, nearest first.
///
/// Steps are limited to the four orthogonal neighbours: a diagonal step
/// between two wall corners fits through the gap on the grid but not once a
/// soldier's bounding box is taken into account, and the soldier would stick
/// on the corner.
pub struct Flood {
    origin: Position,
    /// Each reached tile and the tile it was reached from.
    came_from: HashMap<Position, Position>,
    /// Reached tiles in breadth-first order, so the first match of any
    /// predicate is also the closest one.
    reached: Vec<Position>,
}

impl Flood {
    pub fn new(maze: &Maze, origin: Position, max_cells: usize) -> Self {
        let mut flood = Self {
            origin,
            came_from: HashMap::new(),
            reached: Vec::new(),
        };
        if !is_walkable(maze, origin) || max_cells == 0 {
            return flood;
        }
        let mut queue = VecDeque::new();
        queue.push_back(origin);
        flood.came_from.insert(origin, origin);
        while let Some(cell) = queue.pop_front() {
            flood.reached.push(cell);
            if flood.reached.len() >= max_cells {
                break;
            }
            for next in neighbours(cell) {
                if flood.came_from.contains_key(&next) || !is_walkable(maze, next) {
                    continue;
                }
                flood.came_from.insert(next, cell);
                queue.push_back(next);
            }
        }
        flood
    }

    /// Reached tiles, nearest first. The origin is the first entry.
    pub fn reached(&self) -> &[Position] {
        &self.reached
    }

    /// Waypoints from the origin to `target`, excluding the origin itself.
    /// `None` if the flood never reached `target`.
    pub fn route_to(&self, target: Position) -> Option<Vec<Vec2f>> {
        if !self.came_from.contains_key(&target) {
            return None;
        }
        let mut cells = Vec::new();
        let mut cell = target;
        while cell != self.origin {
            cells.push(cell);
            cell = *self.came_from.get(&cell)?;
        }
        cells.reverse();
        Some(cells.into_iter().map(cell_center).collect())
    }

    /// The closest reached tile satisfying `predicate`, skipping the tile the
    /// soldier is already standing on.
    pub fn nearest(&self, predicate: impl Fn(Position) -> bool) -> Option<Position> {
        self.reached
            .iter()
            .skip(1)
            .copied()
            .find(|cell| predicate(*cell))
    }
}

fn neighbours(cell: Position) -> impl Iterator<Item = Position> {
    let Position { row, col } = cell;
    [
        (row > 0).then(|| Position { row: row - 1, col }),
        Some(Position { row: row + 1, col }),
        (col > 0).then(|| Position { row, col: col - 1 }),
        Some(Position { row, col: col + 1 }),
    ]
    .into_iter()
    .flatten()
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    /// `#` wall, `.` floor.
    fn maze(rows: &[&str]) -> Maze {
        let matrix = rows
            .iter()
            .map(|row| {
                row.chars()
                    .map(|c| if c == '#' { 1 } else { 0 })
                    .collect::<Vec<i32>>()
            })
            .collect();
        Maze {
            matrix,
            contour: HashSet::new(),
        }
    }

    fn at(row: usize, col: usize) -> Position {
        Position { row, col }
    }

    #[test]
    fn route_follows_a_corridor_around_a_wall() {
        let maze = maze(&["#####", "#...#", "#.#.#", "#...#", "#####"]);
        let flood = Flood::new(&maze, at(1, 1), 100);
        let route = flood.route_to(at(3, 3)).expect("no route");
        // four steps around the pillar, never through it
        assert_eq!(route.len(), 4);
        for step in &route {
            assert!(!maze.is_wall(*step), "route walks into a wall at {step}");
        }
        assert_eq!(*route.last().unwrap(), cell_center(at(3, 3)));
    }

    #[test]
    fn walled_off_tiles_are_unreachable() {
        let maze = maze(&["#####", "#.#.#", "#.#.#", "#####"]);
        let flood = Flood::new(&maze, at(1, 1), 100);
        assert!(flood.route_to(at(1, 3)).is_none());
        assert!(flood.reached().iter().all(|cell| cell.col == 1));
    }

    #[test]
    fn reached_tiles_come_out_nearest_first() {
        let maze = maze(&["#####", "#...#", "#...#", "#####"]);
        let flood = Flood::new(&maze, at(1, 1), 100);
        let steps = |cell| flood.route_to(cell).map(|r| r.len()).unwrap_or(usize::MAX);
        let distances = flood
            .reached()
            .iter()
            .map(|c| steps(*c))
            .collect::<Vec<_>>();
        assert!(
            distances.windows(2).all(|w| w[0] <= w[1]),
            "not in breadth-first order: {distances:?}"
        );
        assert_eq!(flood.reached().first().copied(), Some(at(1, 1)));
    }

    #[test]
    fn nearest_skips_the_tile_underfoot() {
        let maze = maze(&["#####", "#...#", "#####"]);
        let flood = Flood::new(&maze, at(1, 1), 100);
        // the origin satisfies this too, but a soldier already standing there
        // needs somewhere else to go
        assert_eq!(flood.nearest(|_| true), Some(at(1, 2)));
    }

    #[test]
    fn a_soldier_inside_a_wall_reaches_nothing() {
        let maze = maze(&["###", "###", "###"]);
        let flood = Flood::new(&maze, at(1, 1), 100);
        assert!(flood.reached().is_empty());
        assert!(flood.route_to(at(1, 1)).is_none());
    }

    #[test]
    fn line_of_sight_is_blocked_by_a_wall_between_two_points() {
        let open = cell_center(at(1, 1));
        let far = cell_center(at(1, 3));
        let corridor = maze(&["#####", "#...#", "#####"]);
        assert!(has_line_of_sight(&corridor, open, far));

        let split = maze(&["#####", "#.#.#", "#####"]);
        assert!(!has_line_of_sight(&split, open, far));
    }

    #[test]
    fn a_wall_behind_the_target_does_not_block_it() {
        // the far wall is past the target, so it must not count
        let maze = maze(&["#####", "#...#", "#####"]);
        assert!(has_line_of_sight(
            &maze,
            cell_center(at(1, 1)),
            cell_center(at(1, 2))
        ));
    }

    #[test]
    fn the_flood_stops_at_its_cap() {
        let maze = maze(&["######", "#....#", "#....#", "######"]);
        let flood = Flood::new(&maze, at(1, 1), 3);
        assert_eq!(flood.reached().len(), 3);
    }
}

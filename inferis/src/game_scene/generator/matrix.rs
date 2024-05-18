use std::collections::{HashSet, VecDeque};

use rand::prelude::*;

pub type MatrixElement = i32;

pub type Matrix = Vec<Vec<MatrixElement>>;

#[derive(Default, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    fn up(&self, _dims: &Dimension) -> Option<Position> {
        if self.row > 0 {
            Some(Position {
                row: self.row - 1,
                col: self.col,
            })
        } else {
            None
        }
    }

    fn down(&self, dims: &Dimension) -> Option<Position> {
        if self.row < dims.rows - 1 {
            Some(Position {
                row: self.row + 1,
                col: self.col,
            })
        } else {
            None
        }
    }

    fn left(&self, _dims: &Dimension) -> Option<Position> {
        if self.col > 0 {
            Some(Position {
                row: self.row,
                col: self.col - 1,
            })
        } else {
            None
        }
    }

    fn right(&self, dims: &Dimension) -> Option<Position> {
        if self.col < dims.cols - 1 {
            Some(Position {
                row: self.row,
                col: self.col + 1,
            })
        } else {
            None
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct Dimension {
    pub rows: usize,
    pub cols: usize,
}

pub fn size(matrix: &Matrix) -> Option<Dimension> {
    let rows = matrix.len();
    if rows == 0 {
        return None;
    }
    let cols = matrix.first().map(|row| row.len())?;
    Some(Dimension { rows, cols })
}

pub fn noise_matrix(
    matrix: &mut Matrix,
    noise_density: u8,
    val_on: MatrixElement,
    val_off: MatrixElement,
) {
    let mut rng = thread_rng();
    for row in matrix.iter_mut() {
        for elem in row {
            let val: u8 = rng.gen_range(1..=100);
            *elem = if val < noise_density { val_on } else { val_off }
        }
    }
}

pub fn fill_borders(matrix: &mut Matrix, fill: MatrixElement) {
    for row in matrix.iter_mut() {
        *row.first_mut().unwrap() = fill;
        *row.last_mut().unwrap() = fill;
    }
    matrix
        .first_mut()
        .unwrap()
        .iter_mut()
        .for_each(|x| *x = fill);
    matrix
        .last_mut()
        .unwrap()
        .iter_mut()
        .for_each(|x| *x = fill);
}

#[allow(clippy::needless_range_loop)]
pub fn moore_neighborhood(
    input: &Matrix,
    val_on: MatrixElement,
    val_off: MatrixElement,
) -> Option<Matrix> {
    let size = size(input)?;
    let mut output = input.clone();
    for i in 1..size.rows - 1 {
        for j in 1..size.cols - 1 {
            let mut wall_count = 0;
            for adj_i in i - 1..=i + 1 {
                for adj_j in j - 1..=j + 1 {
                    if adj_i == i && adj_j == j {
                        continue;
                    }
                    if input[adj_i][adj_j] == val_on {
                        wall_count += 1;
                    }
                }
            }
            output[i][j] = if wall_count > 4 { val_on } else { val_off };
        }
    }
    Some(output)
}

pub fn contours(matrix: &Matrix, value: MatrixElement) -> HashSet<Position> {
    let mut positions = HashSet::<Position>::new();
    let rows = matrix.len();
    for (i, row) in matrix.iter().enumerate() {
        let cols = row.len();
        for (j, val) in row.iter().enumerate() {
            if *val != value {
                continue;
            }
            let mut adj = Vec::with_capacity(4);
            if i > 0 {
                adj.push(matrix[i - 1][j]);
            }
            if i < rows - 1 {
                adj.push(matrix[i + 1][j]);
            }
            if j > 0 {
                adj.push(matrix[i][j - 1]);
            }
            if j < cols - 1 {
                adj.push(matrix[i][j + 1]);
            }
            if adj.iter().all(|x| *x == *val) {
                continue;
            }
            positions.insert(Position { row: i, col: j });
        }
    }
    positions
}

pub fn regions(matrix: &Matrix, value: MatrixElement) -> Vec<HashSet<Position>> {
    let mut regions = Vec::<HashSet<Position>>::new();
    let mut visited = HashSet::<Position>::new();
    let Some(dims) = size(matrix) else {
        return regions;
    };
    for (i, row) in matrix.iter().enumerate() {
        for (j, val) in row.iter().enumerate() {
            if *val != value {
                continue;
            }
            let mut region = HashSet::new();
            let mut deque = VecDeque::<Position>::new();
            deque.push_back(Position { row: i, col: j });
            while let Some(pos) = deque.pop_front() {
                if visited.contains(&pos) {
                    continue;
                }
                region.insert(pos);
                visited.insert(pos);
                [
                    pos.up(&dims),
                    pos.down(&dims),
                    pos.left(&dims),
                    pos.right(&dims),
                ]
                .iter()
                .filter_map(|x| *x)
                .filter(|p| matrix[p.row][p.col] == value)
                .for_each(|p| {
                    deque.push_back(p);
                });
            }
            if !region.is_empty() {
                regions.push(region);
            }
        }
    }
    regions
}

pub fn generate_matrix(
    rows: usize,
    cols: usize,
    val_on: MatrixElement,
    val_off: MatrixElement,
    threshold: usize,
) -> Option<Matrix> {
    let noise_density = 58;
    let iterations = 3;
    let mut matrix = vec![vec![0; cols]; rows];
    noise_matrix(&mut matrix, noise_density, val_on, val_off);
    fill_borders(&mut matrix, val_on);
    for _ in 0..iterations {
        matrix = moore_neighborhood(&matrix, val_on, val_off)?;
    }
    // fill isolated gaps
    let mut regions = self::regions(&matrix, val_off);
    let max = regions.iter().map(|x| x.len()).max().unwrap_or_default();
    for region in regions.iter() {
        if region.len() == max {
            continue;
        }
        for pos in region.iter() {
            matrix[pos.row][pos.col] = val_on;
        }
    }
    // filter small wall regions
    regions = self::regions(&matrix, val_on);
    for region in regions.iter() {
        if region.len() > threshold {
            continue;
        }
        for pos in region.iter() {
            matrix[pos.row][pos.col] = val_off;
        }
    }
    Some(matrix)
}

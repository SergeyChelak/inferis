use std::{fmt::Display, fs, path::Path};

pub type PBMColorType = u8;

pub struct PBMImage {
    rows: usize,
    cols: usize,
    content: Vec<PBMColorType>,
}

impl PBMImage {
    pub fn with_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_str(&content)
    }

    fn from_str(s: &str) -> Result<Self, String> {
        let Some((header, mut rest)) = s.split_once('\n') else {
            return Err("[PBM] Bad input".to_string());
        };
        if header != "P1" {
            return Err("[PBM] Wrong header".to_string());
        }
        while rest.starts_with('#') {
            if let Some((_, sub_str)) = rest.split_once('\n') {
                rest = sub_str;
            }
        }
        let Some((size, content)) = rest.split_once('\n') else {
            return Err("[PBM] Invalid format".to_string());
        };

        let Some((cols, rows)) = size.split_once(' ') else {
            return Err("[PBM] Failed to determine image size".to_string());
        };
        let Some(cols) = cols.parse::<usize>().ok() else {
            return Err("[PBM] width isn't integer value".to_string());
        };

        let Some(rows) = rows.parse::<usize>().ok() else {
            return Err("[PBM] height isn't integer value".to_string());
        };

        let content = content
            .chars()
            .filter_map(|ch| ch.to_digit(10))
            .map(|x| x as PBMColorType)
            .collect::<Vec<_>>();
        if rows * cols != content.len() {
            return Err("[PBM] rows/cols count don't match content length".to_string());
        }
        Ok(Self {
            rows,
            cols,
            content,
        })
    }

    pub fn get(&self, row: usize, col: usize) -> PBMColorType {
        self.content[self.cols * row + col]
    }
}

impl Display for PBMImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..self.rows {
            for col in 0..self.cols {
                let ch = if self.get(row, col) == 0 { ' ' } else { '#' };
                write!(f, "{}", ch)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl PBMImage {
    pub fn transform_to_array<T, Transform>(&self, transform: Transform) -> Vec<Vec<T>>
    where
        Transform: Fn(PBMColorType) -> T,
    {
        let mut arr = Vec::with_capacity(self.rows);
        for r in 0..self.rows {
            let mut row = Vec::with_capacity(self.cols);
            for c in 0..self.cols {
                let val = self.get(r, c);
                row.push(transform(val));
            }
            arr.push(row);
        }
        arr
    }
}

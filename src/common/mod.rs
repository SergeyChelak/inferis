use std::{fs::read_to_string, io, path::Path};

/// Common utils

// Dims
#[derive(Copy, Clone, Debug)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

pub type U32Size = Size<u32>;

// Files
pub fn read_file_as_lines<P: AsRef<Path>>(path: P) -> io::Result<Vec<String>> {
    let contents = read_to_string(path)?;
    let lines = contents
        .lines()
        .collect::<Vec<&str>>()
        .iter()
        .map(|v| v.to_string())
        .collect();
    Ok(lines)
}

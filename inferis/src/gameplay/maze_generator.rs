use engine::{EngineError, EngineResult};

use crate::pbm::PBMImage;

use super::components::Maze;

#[derive(Default)]
pub struct MazeGenerator {
    // don't know if struct is required
}

impl MazeGenerator {
    pub fn generate(&self) -> EngineResult<Maze> {
        let image = PBMImage::with_file("assets/level.pbm")
            .map_err(|err| EngineError::MazeGenerationFailed(err.to_string()))?;
        let maze = image.transform_to_array(|x| x as i32);
        Ok(Maze(maze))
    }
}

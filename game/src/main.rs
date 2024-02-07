use ecs::{self, common::EcsResult};

use crate::game::Game;

mod components;
mod game;
mod types;
mod vec2;

fn main() -> EcsResult<()> {
    println!("Inferis Project");
    let mut game = Game::new();
    game.setup()?;
    game.run()
}

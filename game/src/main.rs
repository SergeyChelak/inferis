use ecs::{self, common::EcsResult};

use crate::game::Game;

mod components;
mod game;
mod types;
mod vec2;

fn main() -> EcsResult<()> {
    println!("Inferis Project");
    let mut game = Game::new();
    game.run();
    // let mut provider = EntityManager::default();
    // provider
    //     .register_component::<Position>()?
    //     .register_component::<Angle>()?
    //     .register_component::<HitPoints>()?
    //     .register_component::<MovementSpeed>()?
    //     .register_component::<RotationSpeed>()?;

    // let player = provider
    //     .new_entity()?
    //     .add_component(Position(Vec2f::new(1.0, 1.0)))?
    //     .add_component(Angle(0.1))?
    //     .add_component(MovementSpeed)?
    //     .add_component(HitPoints(100))?
    //     .as_id();

    // let npc = provider
    //     .new_entity()?
    //     .add_component(Position(Vec2f::new(3.0, 3.0)))?
    //     .add_component(MovementSpeed)?
    //     .add_component(HitPoints(100))?
    //     .as_id();

    // let hero = provider
    //     .new_entity()?
    //     .add_component(HitPoints(200))?
    //     .add_component(Position(Vec2f::new(2.0, 2.0)))?
    //     .as_id();
    // println!("{:20}{player}", "Player id");
    // println!("{:20}{npc}", "NPC id");
    // println!("{:20}{hero}", "Unknown hero id");
    Ok(())
}

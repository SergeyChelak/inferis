use ecs::{self, common::EcsResult, Ecs};

use crate::components::{Angle, HitPoints, MovementSpeed, Position, RotationSpeed};

mod components;
mod types;
mod vec2;

fn main() -> EcsResult<()> {
    println!("Inferis Project");
    let mut world = Ecs::new();

    world
        .state()
        .register_component::<Position>()?
        .register_component::<Angle>()?
        .register_component::<HitPoints>()?
        .register_component::<MovementSpeed>()?
        .register_component::<RotationSpeed>()?;

    let player = world
        .entity()?
        .add_component(Position)?
        .add_component(Angle(0.1))?
        .add_component(MovementSpeed)?
        .add_component(HitPoints(100))?
        .as_id();

    let npc = world
        .entity()?
        .add_component(Position)?
        .add_component(MovementSpeed)?
        .add_component(HitPoints(100))?
        .as_id();

    let hero = world
        .entity()?
        .add_component(HitPoints(200))?
        .add_component(Position)?
        .as_id();
    println!("Player id {player}");
    println!("NPC id {npc}");
    println!("Unknown hero id {hero}");
    Ok(())
}

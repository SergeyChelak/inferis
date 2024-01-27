use ecs::{self, Ecs, EcsResult};

use crate::components::{Angle, HitPoints, MovementSpeed, Position, RotationSpeed};

mod components;
mod types;
mod vec2;

fn main() -> EcsResult<()> {
    let mut world = Ecs::new();
    world.register_component::<Position>()?;
    world.register_component::<Angle>()?;
    world.register_component::<HitPoints>()?;
    world.register_component::<MovementSpeed>()?;
    world.register_component::<RotationSpeed>()?;

    {
        let player = world.create_entity()?;
        world.entity_add_component(player, Position)?;
        world.entity_add_component(player, Angle(0.1))?;
        world.entity_add_component(player, MovementSpeed)?;
        world.entity_add_component(player, HitPoints(100))?;
    }

    {
        let npc = world.create_entity()?;
        world.entity_add_component(npc, Position)?;
        // should fail because argument is considered as a type
        world.entity_add_component(npc, Angle)?;
        world.entity_add_component(npc, MovementSpeed)?;
        world.entity_add_component(npc, HitPoints(100))?;
    }

    println!("Inferis Project");
    Ok(())
}

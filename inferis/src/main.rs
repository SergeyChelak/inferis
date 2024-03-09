use engine::prelude::*;

fn main() -> EcsResult<()> {
    let mut world = GameWorld::new();
    world.run();
    Ok(())
}

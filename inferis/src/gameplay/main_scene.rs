use std::borrow::BorrowMut;

use engine::{pixels::Color, rect::Rect, *};

use super::{components::*, controller::ControllerState};

pub struct GameScene {
    storage: ComponentStorage,
    controller: ControllerState,
    player_id: EntityID,
}

impl GameScene {
    pub fn new() -> EngineResult<Self> {
        let mut storage = ComponentStorage::new();
        storage.register_component::<PlayerTag>()?;
        storage.register_component::<NpcTag>()?;
        storage.register_component::<Health>()?;
        storage.register_component::<Position>()?;
        storage.register_component::<Velocity>()?;
        storage.register_component::<RotationSpeed>()?;
        storage.register_component::<Maze>()?;

        let player_id = storage.add_entity();
        EntityHandler::new(player_id, &mut storage)
            .with_component(PlayerTag)
            .with_component(Health(100))
            .with_component(Velocity(5.0))
            .with_component(RotationSpeed(2.0))
            .with_component(Position(Vec2f::new(300.0, 150.0)));
        Ok(Self {
            storage,
            controller: ControllerState::default(),
            player_id,
        })
    }

    fn render(&mut self, engine: &mut dyn Engine, assets: &AssetManager) -> EngineResult<()> {
        let canvas = engine.canvas();
        let Some(&color) = assets.color("floor") else {
            return Err(EngineError::ResourceNotFound("floor".to_string()));
        };
        canvas.set_draw_color(color);
        canvas.clear();
        // draw player rect
        {
            let Some(pos) = self.storage.get::<Position>(self.player_id) else {
                return Err(EngineError::ComponentNotFound("Position".to_string()));
            };
            // let scale = 10.0;
            let rect = Rect::new(pos.0.x as i32, pos.0.y as i32, 10, 10);
            canvas.set_draw_color(Color::RED);
            canvas
                .fill_rect(rect)
                .map_err(|e| EngineError::Sdl(e.to_string()))?
        }
        canvas.present();
        Ok(())
    }

    fn update_player_position(&mut self) -> EngineResult<()> {
        let (mut dx, mut dy) = (0.0, 0.0);
        let Some(vel_comp) = self.storage.get::<Velocity>(self.player_id) else {
            return Err(EngineError::ComponentNotFound("".to_string()));
        };
        let velocity = vel_comp.0;
        if self.controller.forward_pressed {
            dy = -velocity;
        }
        if self.controller.backward_pressed {
            dy = velocity;
        }
        if self.controller.left_pressed {
            dx = -velocity;
        }
        if self.controller.right_pressed {
            dx = velocity;
        }
        let Some(mut pos_comp) = self.storage.get_mut::<Position>(self.player_id) else {
            return Err(EngineError::ComponentNotFound("".to_string()));
        };
        let position = pos_comp.borrow_mut();
        position.0.x += dx;
        position.0.y += dy;
        Ok(())
    }
}

impl Scene for GameScene {
    fn teak(
        &mut self,
        engine: &mut dyn Engine,
        events: &[InputEvent],
        assets: &AssetManager,
    ) -> EngineResult<()> {
        self.controller.update(events);
        // TODO: call systems here
        // update player position
        self.update_player_position()?;
        // update NPC position
        // find & resolve collisions

        // call renderer system
        self.render(engine, assets)?;
        self.controller.clean_up();
        Ok(())
    }

    fn id(&self) -> String {
        "game_scene".to_string()
    }
}

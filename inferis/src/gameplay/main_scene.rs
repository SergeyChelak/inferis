use std::{
    borrow::{Borrow, BorrowMut},
    f32::consts::PI,
};

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
        storage.register_component::<Angle>()?;

        let bundle = EntityBundle::new()
            .add(PlayerTag)
            .add(Health(100))
            .add(Velocity(5.0))
            .add(RotationSpeed(2.0))
            .add(Position(Vec2f::new(300.0, 150.0)))
            .add(Angle(0.0));

        let player_id = storage.add_from_bundle(&bundle);
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
        let delta_time = 1.0;
        let Some(vel_comp) = self.storage.get::<Velocity>(self.player_id) else {
            return Err(EngineError::ComponentNotFound("".to_string()));
        };
        let Some(mut angle_comp) = self.storage.get_mut::<Angle>(self.player_id) else {
            return Err(EngineError::ComponentNotFound("".to_string()));
        };
        let angle = angle_comp.borrow().0;
        let sin_a = angle.sin();
        let cos_a = angle.cos();

        let velocity = vel_comp.0;
        let dist = velocity * delta_time;
        let dist_cos = dist * cos_a;
        let dist_sin = dist * sin_a;

        let (mut dx, mut dy) = (0.0, 0.0);

        if self.controller.forward_pressed {
            dx = dist_cos;
            dy = dist_sin;
        }
        if self.controller.backward_pressed {
            dx = -dist_cos;
            dy = -dist_sin;
        }
        if self.controller.left_pressed {
            dx = dist_sin;
            dy = -dist_cos;
        }
        if self.controller.right_pressed {
            dx = -dist_sin;
            dy = dist_cos;
        }
        let Some(mut pos_comp) = self.storage.get_mut::<Position>(self.player_id) else {
            return Err(EngineError::ComponentNotFound("".to_string()));
        };
        let position = pos_comp.borrow_mut();
        position.0.x += dx;
        position.0.y += dy;
        // rotation
        let Some(rot_speed_comp) = self.storage.get::<RotationSpeed>(self.player_id) else {
            return Err(EngineError::ComponentNotFound("".to_string()));
        };
        let rotation_speed = rot_speed_comp.0;
        let angle = angle_comp.borrow_mut();
        if self.controller.rotate_left_pressed {
            angle.0 -= rotation_speed * delta_time;
        }
        if self.controller.rotate_right_pressed {
            angle.0 += rotation_speed * delta_time;
        }
        angle.0 %= 2.0 * PI;
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
        self.controller.reset_relative();
        Ok(())
    }

    fn id(&self) -> String {
        "game_scene".to_string()
    }
}

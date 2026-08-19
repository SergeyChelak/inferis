use components::SoundFx;
use engine::{
    game_scene::SceneParameters, refresh_cached_entity, systems::GameSystem, ComponentStorage,
    EngineError, EngineResult, EntityID, Float, Query, Vec2f,
};
use log::info;
use rand::RngExt;

use crate::{
    game_scene::subsystems::{can_shoot, get_actor_state, update_weapon_state},
    resource::{
        NPC_SOLDIER_ATTACK, NPC_SOLDIER_DAMAGE, NPC_SOLDIER_DEATH, NPC_SOLDIER_IDLE,
        NPC_SOLDIER_WALK, SCENE_MAIN_MENU, SCENE_PARAM_WIN, SOUND_NPC_ATTACK, SOUND_NPC_DEATH,
        SOUND_NPC_PAIN,
    },
};

use super::{
    components::{self, ActorState, HealthType, NpcIntent, Sprite},
    generator::{matrix::Position as MatrixPosition, NPC_SOLDIER_HEALTH},
    navigation::{cell_at, cell_center, has_line_of_sight, Flood},
    subsystems::{is_actor_dead, ray_cast_from_entity, replace_actor_state, updated_state},
};

pub const NPC_SOLDIER_SHOT_DEADLINE: usize = 10;
pub const NPC_SOLDIER_DAMAGE_RECOVER: usize = 20;
/// Distance at which a soldier stops closing in and starts shooting.
const NPC_SOLDIER_ATTACK_DISTANCE: Float = 5.0;

/// Health at or below which a soldier breaks off and looks for cover.
const NPC_SOLDIER_CRITICAL_HEALTH: HealthType = NPC_SOLDIER_HEALTH / 4;
/// How long a wounded soldier stays out of sight before coming back.
/// Steps are a fixed 1/60s, so these counts are honest durations.
const NPC_SOLDIER_HIDE_FRAMES: usize = 4 * 60;
/// How long a soldier looks around on reaching the player's last known spot.
const NPC_SOLDIER_LOOK_AROUND_FRAMES: usize = 45;
/// Range of pauses between wander legs, so a group does not move in lockstep.
const NPC_SOLDIER_WANDER_PAUSE: std::ops::Range<usize> = 30..150;
/// How far a soldier steps aside after being hit. A dodge is a sidestep,
/// not a sprint: far enough not to be a stationary target, near enough to
/// stay in the fight and in view.
const NPC_SOLDIER_DODGE_RANGE: std::ops::Range<Float> = 1.0..2.5;
/// A dodge is hurried but not a flat run.
const NPC_SOLDIER_DODGE_SPEED: Float = 0.6;
/// How rarely a soldier may sidestep. It takes four shotgun hits to kill a
/// soldier; dodging every one of them means re-acquiring the target four
/// times, which is what makes them feel impossible to finish off.
const NPC_SOLDIER_DODGE_COOLDOWN: usize = 150;
/// How rarely a soldier may break off to hide. Without this a wounded one
/// hides on every hit and can never be finished off.
const NPC_SOLDIER_HIDE_COOLDOWN: usize = 15 * 60;
/// Cap on the tiles one re-plan considers. A soldier only ever wanders
/// locally, and the cap keeps the flood off the whole maze.
const NPC_NAV_FLOOD_CELLS: usize = 400;
/// How close to a waypoint counts as having reached it.
const NPC_WAYPOINT_TOLERANCE: Float = 0.15;
/// Ground covered in a step below which a soldier counts as not moving.
const NPC_MIN_PROGRESS: Float = 0.01;
/// How long a soldier shoves at a blockage before giving up on its route.
/// Soldiers are obstacles to each other, so a route planned around the walls
/// can still be blocked by a comrade standing in it.
const NPC_SOLDIER_STUCK_FRAMES: usize = 45;

/// What a soldier is doing on this step.
#[derive(Clone, Copy, PartialEq, Eq)]
enum NpcAction {
    /// Player in sight and in range.
    Attack,
    /// Player in sight, closing the distance.
    Chase,
    /// Walking the current route.
    Follow,
    /// Standing still: pausing between legs, or lying low.
    Hold,
}

/// How many simulation steps a soldier may reuse its last line-of-sight
/// verdict for, chosen by its distance to the player: `(distance below,
/// steps between casts)`, first match wins.
///
/// The cast is the dominant per-step AI cost, and the further away the
/// player is the longer a stale verdict goes unnoticed -- a soldier twenty
/// tiles off that keeps walking for another fifth of a second after the
/// player breaks line of sight looks no different. Inside attack range the
/// cast still runs every step, where reaction time is actually visible.
const TARGET_CHECK_INTERVALS: [(Float, usize); 3] = [
    (NPC_SOLDIER_ATTACK_DISTANCE * 2.0, 1),
    (15.0, 4),
    (Float::INFINITY, 12),
];

#[derive(Default)]
pub struct NpcSystem {
    player_id: EntityID,
    maze_id: EntityID,
    // short term cache
    player_position: Vec2f,
    frames: usize,
    delta_time: f32,
}

impl NpcSystem {
    pub fn new() -> Self {
        Default::default()
    }

    fn update_storage_cache(&mut self, storage: &ComponentStorage) -> EngineResult<()> {
        refresh_cached_entity::<components::PlayerTag>(
            storage,
            &mut self.player_id,
            "[v2.npc] player",
        )?;
        refresh_cached_entity::<components::Maze>(storage, &mut self.maze_id, "[v2.npc] maze")
    }

    fn prefetch(&mut self, storage: &ComponentStorage) -> EngineResult<()> {
        self.player_position = storage
            .get::<components::Position>(self.player_id)
            .map(|x| x.0)
            .ok_or(EngineError::component_not_found("[v2.npc] player Position"))?;
        Ok(())
    }

    fn update_npc(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
    ) -> EngineResult<()> {
        // being hit, dying, or recovering from a hit overrides whatever the
        // soldier had in mind
        if let Some(state) =
            updated_state(self.frames, storage, entity_id, NPC_SOLDIER_DAMAGE_RECOVER)?
        {
            if matches!(state, ActorState::Dead(_)) {
                storage.set::<components::NpcTag>(entity_id, None)?;
                storage.set::<components::BoundingBox>(entity_id, None)?;
                storage.set::<components::Angle>(entity_id, None)?;
            } else if matches!(state, ActorState::Damaged(_)) {
                self.react_to_damage(storage, entity_id)?;
            }
            // Damage and death are applied here, not through apply_state:
            // state_if_damaged has already written the component, so asking
            // replace_actor_state to change it reports "no change" and the
            // death animation, the pain animation and both sounds are all
            // skipped. That is what leaves a killed soldier standing.
            storage.set(entity_id, Some(state))?;
            self.update_npc_view(storage, entity_id, &state)?;
            self.update_npc_sound(storage, entity_id, &state)?;
            _ = update_weapon_state(self.frames, storage, entity_id);
            return Ok(());
        }
        // still flinching from the last hit, or already dead
        if matches!(
            get_actor_state(storage, entity_id),
            ActorState::Damaged(_) | ActorState::Dead(_)
        ) {
            return Ok(());
        }
        _ = update_weapon_state(self.frames, storage, entity_id);

        let action = self.decide(storage, entity_id)?;
        let state = self.perform(storage, entity_id, action)?;
        self.apply_state(storage, entity_id, state)?;
        Ok(())
    }

    /// Records a change of state and brings the sprite and sound with it.
    ///
    /// Only for states derived from what the soldier chose to do, where
    /// nothing should happen if the state is the one it already had --
    /// otherwise the walk animation restarts on every step.
    fn apply_state(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
        state: ActorState,
    ) -> EngineResult<()> {
        let Some(new_state) = replace_actor_state(state, storage, entity_id)? else {
            return Ok(());
        };
        self.update_npc_view(storage, entity_id, &new_state)?;
        self.update_npc_sound(storage, entity_id, &new_state)
    }

    /// Chooses what the soldier does this step.
    ///
    /// The ladder is: an override already under way (stepping aside after a
    /// hit, or lying low wounded), then the player if visible, then whatever
    /// the soldier was walking towards.
    fn decide(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
    ) -> EngineResult<NpcAction> {
        if let Some(action) = self.continue_override(storage, entity_id) {
            return Ok(action);
        }
        if self.player_in_sight(storage, entity_id)? {
            let Some(position) = storage.get::<components::Position>(entity_id).map(|x| x.0) else {
                return Ok(NpcAction::Hold);
            };
            {
                let Some(mut plan) = storage.get_mut::<components::NpcPlan>(entity_id) else {
                    return Ok(NpcAction::Hold);
                };
                // remember where they were, and drop wherever we were headed
                plan.last_seen = Some(self.player_position);
                plan.route.clear();
                plan.intent = NpcIntent::Investigate;
                plan.hold_until = 0;
            }
            let distance = (self.player_position - position).length();
            return Ok(if distance < NPC_SOLDIER_ATTACK_DISTANCE {
                NpcAction::Attack
            } else {
                NpcAction::Chase
            });
        }
        // out of sight: chase down the memory, or drift
        self.replan_if_idle(storage, entity_id)?;
        Ok(self.route_action(storage, entity_id))
    }

    /// Keeps a reposition or a hide running to completion, so a soldier that
    /// has broken off does not turn and fight again on the next step.
    fn continue_override(
        &self,
        storage: &ComponentStorage,
        entity_id: EntityID,
    ) -> Option<NpcAction> {
        let plan = storage.get::<components::NpcPlan>(entity_id)?;
        if !matches!(plan.intent, NpcIntent::Reposition | NpcIntent::Hide) {
            return None;
        }
        if !plan.route.is_empty() {
            return Some(NpcAction::Follow);
        }
        if self.frames < plan.hold_until {
            return Some(NpcAction::Hold);
        }
        None
    }

    /// Runs the throttled line-of-sight check and keeps its verdict.
    fn player_in_sight(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
    ) -> EngineResult<bool> {
        let Some(npc_position) = storage.get::<components::Position>(entity_id).map(|x| x.0) else {
            return Ok(false);
        };
        let vector = self.player_position - npc_position;
        let distance = vector.length();
        let standing_verdict = storage
            .get::<components::NpcPlan>(entity_id)
            .map(|plan| plan.player_visible)
            .unwrap_or_default();
        if !self.is_target_check_due(entity_id, distance) {
            return Ok(standing_verdict);
        }
        let angle = vector.y.atan2(vector.x);
        let target_id =
            ray_cast_from_entity(entity_id, storage, self.maze_id, npc_position, angle)?;
        let visible = target_id == Some(self.player_id);
        if let Some(mut plan) = storage.get_mut::<components::NpcPlan>(entity_id) {
            plan.player_visible = visible;
        }
        if visible {
            // turn towards the player only once the ray confirms they can
            // actually be seen: turning first tracks them through walls
            storage.set(entity_id, Some(components::Angle(angle)))?;
        }
        Ok(visible)
    }

    /// Whether this soldier should re-cast toward the player on this step.
    ///
    /// The entity index offsets the phase so that soldiers sharing an
    /// interval spread their casts across it rather than all firing on the
    /// same step and spiking one frame.
    fn is_target_check_due(&self, entity_id: EntityID, distance: Float) -> bool {
        let interval = target_check_interval(distance);
        interval <= 1 || (self.frames + entity_id.index()).is_multiple_of(interval)
    }

    /// Picks somewhere to go when the soldier has run out of route.
    fn replan_if_idle(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
    ) -> EngineResult<()> {
        let (intent, idle, last_seen) = {
            let Some(plan) = storage.get::<components::NpcPlan>(entity_id) else {
                return Ok(());
            };
            let idle = plan.route.is_empty() && self.frames >= plan.hold_until;
            (plan.intent, idle, plan.last_seen)
        };
        if !idle {
            return Ok(());
        }
        // the leg is done: stand for a moment before choosing the next one
        let pause = storage
            .get::<components::NpcPlan>(entity_id)
            .map(|plan| plan.pause_after_route)
            .unwrap_or_default();
        if pause > 0 {
            if let Some(mut plan) = storage.get_mut::<components::NpcPlan>(entity_id) {
                plan.hold_until = self.frames + pause;
                plan.pause_after_route = 0;
            }
            return Ok(());
        }
        // a soldier that has just lost sight goes to look where they were
        if intent == NpcIntent::Investigate {
            if let Some(spot) = last_seen {
                if self.route_to_point(storage, entity_id, spot)? {
                    return Ok(());
                }
            }
            // arrived, or nowhere to go: look around, then drift off
            self.set_wander_plan(storage, entity_id, NPC_SOLDIER_LOOK_AROUND_FRAMES)?;
            return Ok(());
        }
        self.set_wander_plan(storage, entity_id, 0)
    }

    /// Routes to a random reachable tile, then stands for a moment.
    fn set_wander_plan(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
        extra_hold: usize,
    ) -> EngineResult<()> {
        let position = storage
            .get::<components::Position>(entity_id)
            .map(|x| x.0)
            .unwrap_or_default();
        let route = self.plan_route(storage, entity_id, |flood, _, _| {
            let reached = flood.reached();
            if reached.len() < 2 {
                return None;
            }
            Some(reached[rand::rng().random_range(1..reached.len())])
        });
        let pause = rand::rng().random_range(NPC_SOLDIER_WANDER_PAUSE) + extra_hold;
        if let Some(mut plan) = storage.get_mut::<components::NpcPlan>(entity_id) {
            plan.intent = NpcIntent::Wander;
            plan.route = route.unwrap_or_default().into();
            plan.hold_until = 0;
            plan.pause_after_route = pause;
            plan.last_seen = None;
            plan.progress_frame = self.frames;
            plan.last_position = position;
        }
        Ok(())
    }

    /// Routes to the tile holding `point`. False if it cannot be reached.
    fn route_to_point(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
        point: Vec2f,
    ) -> EngineResult<bool> {
        let Some(target) = cell_at(point) else {
            return Ok(false);
        };
        let route = self.plan_route(storage, entity_id, |flood, _, _| {
            flood.reached().contains(&target).then_some(target)
        });
        let Some(route) = route.filter(|route| !route.is_empty()) else {
            return Ok(false);
        };
        if let Some(mut plan) = storage.get_mut::<components::NpcPlan>(entity_id) {
            plan.route = route.into();
            plan.hold_until = 0;
            plan.progress_frame = self.frames;
            plan.last_position = point;
        }
        Ok(true)
    }

    /// Floods out from the soldier and routes to whatever `pick` chooses.
    fn plan_route(
        &self,
        storage: &ComponentStorage,
        entity_id: EntityID,
        pick: impl Fn(&Flood, &components::Maze, Vec2f) -> Option<MatrixPosition>,
    ) -> Option<Vec<Vec2f>> {
        let position = storage
            .get::<components::Position>(entity_id)
            .map(|x| x.0)?;
        let origin = cell_at(position)?;
        let maze = storage.get::<components::Maze>(self.maze_id)?;
        let flood = Flood::new(&maze, origin, NPC_NAV_FLOOD_CELLS);
        let target = pick(&flood, &maze, position)?;
        flood.route_to(target)
    }

    /// Whether the soldier is walking a route or standing at the end of one.
    fn route_action(&self, storage: &ComponentStorage, entity_id: EntityID) -> NpcAction {
        let Some(plan) = storage.get::<components::NpcPlan>(entity_id) else {
            return NpcAction::Hold;
        };
        if plan.route.is_empty() {
            NpcAction::Hold
        } else {
            NpcAction::Follow
        }
    }

    /// Turns a decision into movement, shots, and the animation to show.
    fn perform(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
        action: NpcAction,
    ) -> EngineResult<ActorState> {
        match action {
            NpcAction::Attack => {
                self.shoot(storage, entity_id)?;
                Ok(ActorState::Attack(usize::MAX))
            }
            NpcAction::Chase => {
                let angle = storage
                    .get::<components::Angle>(entity_id)
                    .map(|x| x.0)
                    .unwrap_or_default();
                self.step_towards(storage, entity_id, angle, 1.0)?;
                Ok(ActorState::Walk(usize::MAX))
            }
            NpcAction::Follow => {
                self.step_along_route(storage, entity_id)?;
                Ok(ActorState::Walk(usize::MAX))
            }
            NpcAction::Hold => Ok(ActorState::Idle(usize::MAX)),
        }
    }

    fn shoot(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
    ) -> EngineResult<()> {
        if !can_shoot(storage, entity_id) {
            return Ok(());
        }
        let (Some(position), Some(angle)) = (
            storage.get::<components::Position>(entity_id).map(|x| x.0),
            storage.get::<components::Angle>(entity_id).map(|x| x.0),
        ) else {
            return Ok(());
        };
        let shot = components::Shot {
            position,
            angle,
            deadline: self.frames + NPC_SOLDIER_SHOT_DEADLINE,
        };
        storage.set(entity_id, Some(shot))?;
        storage.set(entity_id, Some(SoundFx::once(SOUND_NPC_ATTACK)))?;
        Ok(())
    }

    /// Walks one step towards the next waypoint, dropping it on arrival.
    fn step_along_route(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
    ) -> EngineResult<()> {
        let Some(position) = storage.get::<components::Position>(entity_id).map(|x| x.0) else {
            return Ok(());
        };
        let waypoint = {
            let Some(mut plan) = storage.get_mut::<components::NpcPlan>(entity_id) else {
                return Ok(());
            };
            // a route planned around the walls can still be blocked by
            // another soldier standing in it; give up rather than shove
            if (position - plan.last_position).length() >= NPC_MIN_PROGRESS {
                plan.progress_frame = self.frames;
            }
            plan.last_position = position;
            if self.frames.saturating_sub(plan.progress_frame) > NPC_SOLDIER_STUCK_FRAMES {
                plan.route.clear();
                plan.progress_frame = self.frames;
                if plan.intent == NpcIntent::Reposition {
                    plan.intent = NpcIntent::Wander;
                }
                return Ok(());
            }
            while plan
                .route
                .front()
                .is_some_and(|point| (*point - position).length() < NPC_WAYPOINT_TOLERANCE)
            {
                plan.route.pop_front();
            }
            plan.route.front().copied()
        };
        let intent = storage
            .get::<components::NpcPlan>(entity_id)
            .map(|plan| plan.intent)
            .unwrap_or_default();
        let Some(waypoint) = waypoint else {
            return Ok(());
        };
        let heading = waypoint - position;
        let angle = heading.y.atan2(heading.x);
        // A soldier sidestepping a shot keeps its eyes on the player: it
        // turns its back only when it has somewhere to be. Movement and
        // facing are separate, so a dodge reads as a dodge rather than as
        // a soldier fleeing at a run.
        let dodging = matches!(intent, NpcIntent::Reposition);
        let facing = if dodging {
            let towards_player = self.player_position - position;
            towards_player.y.atan2(towards_player.x)
        } else {
            angle
        };
        storage.set(entity_id, Some(components::Angle(facing)))?;
        let speed = if dodging {
            NPC_SOLDIER_DODGE_SPEED
        } else {
            1.0
        };
        self.step_towards(storage, entity_id, angle, speed)
    }

    fn step_towards(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
        angle: Float,
        speed: Float,
    ) -> EngineResult<()> {
        let Some(velocity) = storage.get::<components::Velocity>(entity_id).map(|x| x.0) else {
            return Ok(());
        };
        let distance = velocity * speed * self.delta_time;
        let movement = components::Movement {
            x: distance * angle.cos(),
            y: distance * angle.sin(),
            angle: 0.0,
        };
        storage.set(entity_id, Some(movement))?;
        Ok(())
    }

    /// A hit either sends a soldier for cover or makes it step aside.
    fn react_to_damage(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
    ) -> EngineResult<()> {
        let health = storage
            .get::<components::Health>(entity_id)
            .map(|x| x.0)
            .unwrap_or_default();
        let may_hide = storage
            .get::<components::NpcPlan>(entity_id)
            .map(|plan| self.frames >= plan.hide_ready_at)
            .unwrap_or(true);
        if health <= NPC_SOLDIER_CRITICAL_HEALTH && may_hide {
            return self.set_hide_plan(storage, entity_id);
        }
        let may_dodge = storage
            .get::<components::NpcPlan>(entity_id)
            .map(|plan| self.frames >= plan.dodge_ready_at)
            .unwrap_or(true);
        if may_dodge {
            self.set_reposition_plan(storage, entity_id)?;
        }
        Ok(())
    }

    /// Heads for the nearest tile out of the player's sight, and stays there
    /// long enough for the trip to have been worth making.
    fn set_hide_plan(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
    ) -> EngineResult<()> {
        let player_position = self.player_position;
        let route = self.plan_route(storage, entity_id, |flood, maze, _| {
            flood.nearest(|cell| !has_line_of_sight(maze, player_position, cell_center(cell)))
        });
        if let Some(mut plan) = storage.get_mut::<components::NpcPlan>(entity_id) {
            plan.intent = NpcIntent::Hide;
            plan.route = route.unwrap_or_default().into();
            plan.hold_until = self.frames + NPC_SOLDIER_HIDE_FRAMES;
            plan.hide_ready_at = self.frames + NPC_SOLDIER_HIDE_COOLDOWN;
            plan.pause_after_route = 0;
            plan.progress_frame = self.frames;
            plan.last_position = Vec2f::default();
        }
        Ok(())
    }

    /// Steps a few tiles off, so as not to stand where the last shot landed.
    fn set_reposition_plan(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
    ) -> EngineResult<()> {
        let route = self.plan_route(storage, entity_id, |flood, _, origin| {
            let candidates = flood
                .reached()
                .iter()
                .skip(1)
                .filter(|cell| {
                    NPC_SOLDIER_DODGE_RANGE.contains(&(cell_center(**cell) - origin).length())
                })
                .copied()
                .collect::<Vec<_>>();
            if candidates.is_empty() {
                return None;
            }
            Some(candidates[rand::rng().random_range(0..candidates.len())])
        });
        if let Some(mut plan) = storage.get_mut::<components::NpcPlan>(entity_id) {
            plan.intent = NpcIntent::Reposition;
            plan.dodge_ready_at = self.frames + NPC_SOLDIER_DODGE_COOLDOWN;
            plan.route = route.unwrap_or_default().into();
            plan.hold_until = 0;
            plan.pause_after_route = 0;
            plan.progress_frame = self.frames;
            plan.last_position = Vec2f::default();
        }
        Ok(())
    }

    fn update_npc_view(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
        state: &components::ActorState,
    ) -> EngineResult<()> {
        let animation = match state {
            ActorState::Undefined => None,
            ActorState::Idle(_) => Some(Sprite::with_animation(
                NPC_SOLDIER_IDLE,
                self.frames,
                usize::MAX,
            )),
            ActorState::Dead(_) => Some(Sprite::with_animation(NPC_SOLDIER_DEATH, self.frames, 1)),
            ActorState::Walk(_) => Some(Sprite::with_animation(
                NPC_SOLDIER_WALK,
                self.frames,
                usize::MAX,
            )),
            ActorState::Attack(_) => Some(Sprite::with_animation(
                NPC_SOLDIER_ATTACK,
                self.frames,
                usize::MAX,
            )),
            ActorState::Damaged(_) => Some(Sprite::with_animation(
                NPC_SOLDIER_DAMAGE,
                self.frames,
                usize::MAX,
            )),
        };
        storage.set(entity_id, animation)?;
        Ok(())
    }

    fn update_npc_sound(
        &mut self,
        storage: &mut engine::ComponentStorage,
        entity_id: EntityID,
        state: &components::ActorState,
    ) -> EngineResult<()> {
        let sound_fx = match state {
            ActorState::Dead(_) => Some(SoundFx::once(SOUND_NPC_DEATH)),
            ActorState::Damaged(_) => Some(SoundFx::once(SOUND_NPC_PAIN)),
            _ => None,
        };
        storage.set(entity_id, sound_fx)?;
        Ok(())
    }
}

/// Steps between line-of-sight casts for a soldier at `distance`.
fn target_check_interval(distance: Float) -> usize {
    TARGET_CHECK_INTERVALS
        .iter()
        .find(|(below, _)| distance < *below)
        .map(|(_, interval)| *interval)
        .unwrap_or(1)
}

impl GameSystem for NpcSystem {
    fn setup(
        &mut self,
        storage: &mut engine::ComponentStorage,
        _asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<()> {
        self.update_storage_cache(storage)?;
        info!("setup ok");
        Ok(())
    }

    fn update(
        &mut self,
        frames: usize,
        delta_time: engine::Float,
        storage: &mut engine::ComponentStorage,
        _asset_manager: &engine::AssetManager,
    ) -> engine::EngineResult<engine::systems::GameSystemCommand> {
        self.update_storage_cache(storage)?;
        self.prefetch(storage)?;
        self.frames = frames;
        self.delta_time = delta_time;

        let query = Query::new().with_component::<components::NpcTag>();
        let entities = storage.fetch_entities(&query);
        let mut alive_npc = false;
        for entity_id in entities {
            self.update_npc(storage, entity_id)?;
            alive_npc |= !is_actor_dead(storage, entity_id);
        }

        let command = if alive_npc {
            engine::systems::GameSystemCommand::Nothing
        } else {
            let mut params = SceneParameters::default();
            params.insert(SCENE_PARAM_WIN.to_string(), "".to_string());
            engine::systems::GameSystemCommand::SwitchScene {
                id: SCENE_MAIN_MENU,
                params,
            }
        };
        Ok(command)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn inside_attack_range_the_cast_runs_every_step() {
        assert_eq!(target_check_interval(0.0), 1);
        assert_eq!(target_check_interval(NPC_SOLDIER_ATTACK_DISTANCE), 1);
        assert_eq!(target_check_interval(9.9), 1);
    }

    #[test]
    fn the_further_away_the_rarer_the_cast() {
        assert_eq!(target_check_interval(10.0), 4);
        assert_eq!(target_check_interval(14.9), 4);
        assert_eq!(target_check_interval(15.0), 12);
        assert_eq!(target_check_interval(1000.0), 12);
    }

    #[test]
    fn no_interval_is_zero() {
        // a zero interval would make every step look "not due", freezing the
        // soldier on whatever verdict it happened to hold
        assert!(TARGET_CHECK_INTERVALS.iter().all(|(_, steps)| *steps > 0));
    }

    #[test]
    fn soldiers_sharing_an_interval_spread_their_casts_across_it() {
        let mut storage = ComponentStorage::new();
        let ids = (0..4).map(|_| storage.add_entity()).collect::<Vec<_>>();
        let distance = 12.0;
        let interval = target_check_interval(distance);
        assert_eq!(interval, 4);

        let mut system = NpcSystem::new();
        // each soldier is due exactly once per interval ...
        for id in &ids {
            let mut due = 0;
            for step in 0..interval {
                system.frames = step;
                if system.is_target_check_due(*id, distance) {
                    due += 1;
                }
            }
            assert_eq!(due, 1, "entity {} is due {} times", id.index(), due);
        }
        // ... and no single step carries more than its share
        for step in 0..interval {
            system.frames = step;
            let due = ids
                .iter()
                .filter(|id| system.is_target_check_due(**id, distance))
                .count();
            assert_eq!(due, 1, "step {step} carries {due} casts");
        }
    }
}

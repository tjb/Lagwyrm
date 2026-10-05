//! The authoritative world. MODULE 1, GRADED: `World::step` and `World::checksum`.
//!
//! The rules are deliberately tiny: square entities slide around a bounded
//! square arena and block each other. Small enough to finish, but the blocking
//! rule makes the result depend on the order entities move in, which is what
//! makes determinism interesting.

use std::collections::BTreeMap;

use crate::command::Command;
use crate::fixed::Fx;
use crate::ids::{EntityId, PlayerId, Tick};

/// Fixed parameters of a world. Part of every replay recording.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldConfig {
    /// The arena is `[0, size]` on both axes, inclusive.
    pub size: Fx,
    /// Distance moved per tick along each axis an entity is pushing on.
    pub speed: Fx,
    /// Each entity is a square of side `2 * half_extent` centred on its position.
    pub half_extent: Fx,
}

impl Default for WorldConfig {
    fn default() -> WorldConfig {
        WorldConfig {
            size: Fx::from_int(256),
            speed: Fx::from_ratio(1, 4),
            half_extent: Fx::from_ratio(1, 2),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entity {
    pub id: EntityId,
    pub owner: PlayerId,
    pub x: Fx,
    pub y: Fx,
}

#[derive(Clone, Debug)]
pub struct World {
    config: WorldConfig,
    tick: Tick,
    next_entity: u32,
    entities: BTreeMap<EntityId, Entity>,
    by_owner: BTreeMap<PlayerId, EntityId>,
}

impl World {
    pub fn new(config: WorldConfig) -> World {
        World {
            config,
            tick: Tick::ZERO,
            next_entity: 0,
            entities: BTreeMap::new(),
            by_owner: BTreeMap::new(),
        }
    }

    pub fn config(&self) -> &WorldConfig {
        &self.config
    }

    /// The next tick [`World::step`] will simulate. Starts at 0.
    pub fn tick(&self) -> Tick {
        self.tick
    }

    /// Spawns the entity `player` controls at `(x, y)`, clamped into the arena.
    /// Ids are handed out sequentially from 0. Spawning ignores blocking.
    ///
    /// Panics if `player` already has an entity.
    pub fn spawn_at(&mut self, player: PlayerId, x: Fx, y: Fx) -> EntityId {
        assert!(
            !self.by_owner.contains_key(&player),
            "{player} already has an entity"
        );
        let id = EntityId(self.next_entity);
        self.next_entity += 1;
        let (lo, hi) = (Fx::ZERO, self.config.size);
        self.entities.insert(
            id,
            Entity {
                id,
                owner: player,
                x: x.clamp(lo, hi),
                y: y.clamp(lo, hi),
            },
        );
        self.by_owner.insert(player, id);
        id
    }

    pub fn despawn(&mut self, player: PlayerId) -> Option<Entity> {
        let id = self.by_owner.remove(&player)?;
        self.entities.remove(&id)
    }

    pub fn entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }

    pub fn entity_of(&self, player: PlayerId) -> Option<&Entity> {
        self.by_owner
            .get(&player)
            .and_then(|id| self.entities.get(id))
    }

    /// All entities in ascending id order.
    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// True iff the two squares centred on `a` and `b` overlap. Touching edges
    /// do not count: `|dx| < 2h && |dy| < 2h`.
    pub fn overlaps(&self, a: (Fx, Fx), b: (Fx, Fx)) -> bool {
        let reach = self.config.half_extent * 2;
        (a.0 - b.0).abs() < reach && (a.1 - b.1).abs() < reach
    }

    /// Simulates tick `self.tick()` and then advances it by one.
    ///
    /// `inputs` holds at most one command per player, in any order. Players
    /// with no entity are ignored; entities whose owner has no command stand
    /// still.
    ///
    /// The rules, in this order:
    /// 1. Entities move one at a time in ascending `EntityId` order.
    /// 2. An entity's destination is its position plus `speed * move_x` and
    ///    `speed * move_y`, each axis clamped into `[0, size]`.
    /// 3. If the destination overlaps any *other* entity's position as it is
    ///    right now (so including moves already made earlier this tick), the
    ///    entity does not move at all this tick.
    ///
    /// The result must not depend on the order of `inputs`.
    #[allow(unused_variables)] // remove once implemented
    pub fn step(&mut self, inputs: &[(PlayerId, Command)]) {
        todo!("module 1: one deterministic tick")
    }

    /// A 64-bit fingerprint of the full authoritative state, including the tick.
    ///
    /// Two worlds must have equal checksums iff their state is equal (modulo
    /// hash collisions), on any machine, in any build, on any Rust version. It
    /// must notice when two entities swap positions.
    pub fn checksum(&self) -> u64 {
        todo!("module 1: deterministic state hash")
    }
}

//! Module 1 tests for `World::step` and `World::checksum`. These fail until
//! you replace the `todo!()`s in crates/lagwyrm-sim/src/world.rs.

use lagwyrm_sim::{Command, Fx, PlayerId, SplitMix64, Tick, World, WorldConfig};

/// Integer-friendly rules: speed 1, unit half-extent, so two entities overlap
/// iff they are less than 2 apart on both axes.
fn config() -> WorldConfig {
    WorldConfig {
        size: Fx::from_int(100),
        speed: Fx::from_int(1),
        half_extent: Fx::from_int(1),
    }
}

fn at(world: &World, p: PlayerId) -> (i32, i32) {
    let e = world.entity_of(p).unwrap();
    (e.x.floor_int(), e.y.floor_int())
}

fn int(n: i32) -> Fx {
    Fx::from_int(n)
}

const A: PlayerId = PlayerId(10);
const B: PlayerId = PlayerId(20);
const C: PlayerId = PlayerId(30);

#[test]
fn step_advances_the_tick() {
    let mut w = World::new(config());
    w.step(&[]);
    w.step(&[]);
    assert_eq!(w.tick(), Tick(2));
}

#[test]
fn entities_move_by_speed_per_axis() {
    let mut w = World::new(config());
    w.spawn_at(A, int(10), int(10));
    w.step(&[(A, Command::new(1, 0))]);
    assert_eq!(at(&w, A), (11, 10));
    w.step(&[(A, Command::new(-1, 1))]);
    assert_eq!(at(&w, A), (10, 11));
    // Diagonals are not normalised in module 1: both axes move a full step.
    w.step(&[(A, Command::new(1, 1))]);
    assert_eq!(at(&w, A), (11, 12));
}

#[test]
fn no_command_means_no_movement() {
    let mut w = World::new(config());
    w.spawn_at(A, int(10), int(10));
    w.spawn_at(B, int(50), int(50));
    w.step(&[(B, Command::new(1, 0))]);
    assert_eq!(at(&w, A), (10, 10));
    assert_eq!(at(&w, B), (51, 50));
}

#[test]
fn commands_for_players_without_entities_are_ignored() {
    let mut w = World::new(config());
    w.spawn_at(A, int(10), int(10));
    w.step(&[(C, Command::new(1, 1)), (A, Command::new(0, 1))]);
    assert_eq!(at(&w, A), (10, 11));
}

#[test]
fn movement_is_clamped_to_the_arena() {
    let mut w = World::new(config());
    w.spawn_at(A, int(0), int(100));
    w.step(&[(A, Command::new(-1, 1))]);
    assert_eq!(at(&w, A), (0, 100));
    // Clamping happens per axis, so sliding along a wall still works.
    w.step(&[(A, Command::new(1, 1))]);
    assert_eq!(at(&w, A), (1, 100));
}

#[test]
fn moving_into_another_entity_is_blocked() {
    let mut w = World::new(config());
    w.spawn_at(A, int(10), int(10));
    w.spawn_at(B, int(12), int(10));
    w.step(&[(A, Command::new(1, 0))]);
    assert_eq!(
        at(&w, A),
        (10, 10),
        "11 is within 2 of 12, so A must not move"
    );
}

#[test]
fn touching_is_not_overlapping() {
    let mut w = World::new(config());
    w.spawn_at(A, int(10), int(10));
    w.spawn_at(B, int(13), int(10));
    w.step(&[(A, Command::new(1, 0))]);
    assert_eq!(at(&w, A), (11, 10), "exactly 2 apart is allowed");
}

#[test]
fn lower_entity_id_moves_first() {
    // A (entity 0) at 10, B (entity 1) at 13, moving toward each other.
    // A moves first to 11 (2 from B: fine). Then B wants 12, which is 1 from
    // A's *new* position, so B is blocked.
    let mut w = World::new(config());
    w.spawn_at(A, int(10), int(10));
    w.spawn_at(B, int(13), int(10));
    w.step(&[(B, Command::new(-1, 0)), (A, Command::new(1, 0))]);
    assert_eq!(at(&w, A), (11, 10));
    assert_eq!(at(&w, B), (13, 10));
}

#[test]
fn entity_order_is_by_id_not_by_player() {
    // Same geometry, but now B spawns first and so has the lower entity id.
    let mut w = World::new(config());
    w.spawn_at(B, int(13), int(10));
    w.spawn_at(A, int(10), int(10));
    w.step(&[(A, Command::new(1, 0)), (B, Command::new(-1, 0))]);
    assert_eq!(at(&w, B), (12, 10));
    assert_eq!(at(&w, A), (10, 10));
}

#[test]
fn walking_out_of_an_overlap_is_allowed() {
    // The check is on the destination, not on "currently overlapping": an
    // entity that spawned overlapping another can still walk clear of it.
    let mut w = World::new(config());
    w.spawn_at(A, int(10), int(10));
    w.spawn_at(B, int(11), int(10));
    w.step(&[(A, Command::new(-1, 0))]);
    assert_eq!(at(&w, A), (9, 10), "9 is exactly 2 from 11, so A gets out");
}

#[test]
fn moving_deeper_into_an_overlap_is_blocked() {
    let mut w = World::new(config());
    w.spawn_at(A, int(10), int(10));
    w.spawn_at(B, int(11), int(10));
    w.step(&[(A, Command::new(0, 1))]);
    assert_eq!(at(&w, A), (10, 10), "(10, 11) still overlaps B");
}

fn crowd(seed: u64) -> (World, Vec<Vec<(PlayerId, Command)>>) {
    let mut rng = SplitMix64::new(seed);
    let mut w = World::new(WorldConfig {
        size: int(20),
        ..config()
    });
    for p in 0..40 {
        let x = rng.below(21) as i32;
        let y = rng.below(21) as i32;
        w.spawn_at(PlayerId(p), int(x), int(y));
    }
    let script = (0..200)
        .map(|_| {
            (0..40)
                .map(|p| {
                    let dx = rng.below(3) as i8 - 1;
                    let dy = rng.below(3) as i8 - 1;
                    (PlayerId(p), Command::new(dx, dy))
                })
                .collect()
        })
        .collect();
    (w, script)
}

#[test]
fn input_order_does_not_matter() {
    let (mut forward, script) = crowd(7);
    let mut backward = forward.clone();
    let mut shuffled = forward.clone();
    let mut rng = SplitMix64::new(99);
    for inputs in &script {
        forward.step(inputs);
        let mut rev = inputs.clone();
        rev.reverse();
        backward.step(&rev);
        let mut shuf = inputs.clone();
        for i in (1..shuf.len()).rev() {
            shuf.swap(i, rng.below(i as u64 + 1) as usize);
        }
        shuffled.step(&shuf);
    }
    let f: Vec<_> = forward.entities().copied().collect();
    assert_eq!(f, backward.entities().copied().collect::<Vec<_>>());
    assert_eq!(f, shuffled.entities().copied().collect::<Vec<_>>());
}

#[test]
fn crowded_world_never_gains_overlaps() {
    // Entities may spawn overlapping, but movement must never create a new
    // overlapping pair.
    let (mut w, script) = crowd(3);
    let pairs = |w: &World| {
        let es: Vec<_> = w.entities().copied().collect();
        let mut set = std::collections::BTreeSet::new();
        for (i, a) in es.iter().enumerate() {
            for b in &es[i + 1..] {
                if w.overlaps((a.x, a.y), (b.x, b.y)) {
                    set.insert((a.id, b.id));
                }
            }
        }
        set
    };
    let mut allowed = pairs(&w);
    for inputs in &script {
        w.step(inputs);
        let now = pairs(&w);
        assert!(now.is_subset(&allowed), "a move created a new overlap");
        allowed = now;
    }
}

#[test]
fn checksum_is_equal_for_equal_worlds() {
    let (mut a, script) = crowd(11);
    let mut b = a.clone();
    assert_eq!(a.checksum(), b.checksum());
    for inputs in &script {
        a.step(inputs);
        b.step(inputs);
        assert_eq!(a.checksum(), b.checksum());
    }
}

#[test]
fn checksum_covers_the_tick() {
    let mut w = World::new(config());
    w.spawn_at(A, int(10), int(10));
    let before = w.checksum();
    w.step(&[]);
    assert_ne!(before, w.checksum(), "same entities, different tick");
}

#[test]
fn checksum_sees_sub_unit_movement() {
    let mut a = World::new(config());
    let mut b = World::new(config());
    a.spawn_at(A, int(10), int(10));
    b.spawn_at(A, int(10) + Fx::from_raw(1), int(10));
    assert_ne!(a.checksum(), b.checksum());
}

#[test]
fn checksum_notices_swapped_positions() {
    let mut a = World::new(config());
    a.spawn_at(A, int(10), int(20));
    a.spawn_at(B, int(30), int(40));
    let mut b = World::new(config());
    b.spawn_at(A, int(30), int(40));
    b.spawn_at(B, int(10), int(20));
    assert_ne!(a.checksum(), b.checksum());
}

#[test]
fn checksum_notices_swapped_axes() {
    let mut a = World::new(config());
    a.spawn_at(A, int(10), int(20));
    let mut b = World::new(config());
    b.spawn_at(A, int(20), int(10));
    assert_ne!(a.checksum(), b.checksum());
}

#[test]
fn checksum_notices_ownership() {
    let mut a = World::new(config());
    a.spawn_at(A, int(10), int(20));
    let mut b = World::new(config());
    b.spawn_at(B, int(10), int(20));
    assert_ne!(a.checksum(), b.checksum());
}

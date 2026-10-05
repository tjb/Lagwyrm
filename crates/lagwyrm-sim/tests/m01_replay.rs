//! Module 1 tests for deterministic replay. The replay tool is scaffolding,
//! but it only works once `World::step` and `World::checksum` do.

use lagwyrm_sim::{Command, Divergence, Fx, PlayerId, Recording, SplitMix64, Tick, WorldConfig};

fn record(seed: u64, players: u32, ticks: u64) -> Recording {
    let mut rng = SplitMix64::new(seed);
    let mut rec = Recording::new(WorldConfig::default());
    for p in 0..players {
        let x = Fx::from_int(rng.below(64) as i32);
        let y = Fx::from_int(rng.below(64) as i32);
        rec.record_spawn(PlayerId(p), x, y);
    }
    let mut world = rec.initial_world();
    for t in 0..ticks {
        let inputs: Vec<_> = (0..players)
            .map(|p| {
                let dx = rng.below(3) as i8 - 1;
                let dy = rng.below(3) as i8 - 1;
                (PlayerId(p), Command::new(dx, dy))
            })
            .collect();
        world.step(&inputs);
        rec.record_frame(Tick(t), &inputs, world.checksum());
    }
    rec
}

#[test]
fn a_recording_replays_cleanly() {
    let rec = record(5, 32, 500);
    let world = rec.verify().expect("replay should match its own recording");
    assert_eq!(world.tick(), Tick(500));
}

#[test]
fn a_recording_survives_the_text_format() {
    let rec = record(6, 16, 200);
    let back = Recording::from_text(&rec.to_text()).unwrap();
    assert_eq!(
        back.verify().unwrap().checksum(),
        rec.verify().unwrap().checksum()
    );
}

#[test]
fn tampering_is_detected_at_the_right_tick() {
    let mut rec = record(8, 16, 200);
    // Flip one player's command on tick 120. Replay must diverge there,
    // not before.
    let frame = &mut rec.frames[120];
    let cmd = &mut frame.inputs[3].1;
    *cmd = Command::new(-cmd.move_x, if cmd.move_y == 0 { 1 } else { -cmd.move_y });
    match rec.verify() {
        Err(Divergence::Checksum { tick, .. }) => assert_eq!(tick, Tick(120)),
        other => panic!("expected a checksum divergence at t120, got {other:?}"),
    }
}

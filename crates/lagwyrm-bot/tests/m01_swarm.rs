//! Module 1 end-to-end tests: bots, server, timestep, input queue, world and
//! replay together. These pass once all of module 1's `todo!()`s are done.

use lagwyrm_bot::{Swarm, SwarmConfig};

fn small(seed: u64) -> SwarmConfig {
    SwarmConfig {
        bots: 32,
        ticks: 300,
        seed,
        ..SwarmConfig::default()
    }
}

#[test]
fn swarm_runs_the_requested_ticks() {
    let (report, server) = Swarm::new(small(1)).run();
    assert_eq!(report.ticks, 300);
    assert_eq!(server.tick_stats().count(), 300);
    assert!(report.inputs.accepted > 0);
}

#[test]
fn same_seed_same_world() {
    let (a, _) = Swarm::new(small(42)).run();
    let (b, _) = Swarm::new(small(42)).run();
    assert_eq!(a.final_checksum, b.final_checksum);
}

#[test]
fn different_seed_different_world() {
    let (a, _) = Swarm::new(small(1)).run();
    let (b, _) = Swarm::new(small(2)).run();
    assert_ne!(a.final_checksum, b.final_checksum);
}

#[test]
fn server_recording_replays_to_the_same_checksum() {
    let (report, server) = Swarm::new(small(9)).run();
    let rec = server.recording().unwrap();
    let world = rec.verify().expect("server recording must replay cleanly");
    assert_eq!(world.checksum(), report.final_checksum);
}

#[test]
fn late_and_lost_inputs_are_counted_not_fatal() {
    let config = SwarmConfig {
        late_per_mille: 200,
        loss_per_mille: 200,
        ..small(3)
    };
    let (report, _) = Swarm::new(config).run();
    assert!(report.inputs.late > 0, "expected some late inputs");
    assert!(
        report.inputs.repeated > 0,
        "lost inputs should cause repeats"
    );
}

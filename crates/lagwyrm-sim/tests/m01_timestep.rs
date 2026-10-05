//! Module 1 tests for `FixedTimestep`. These fail until you replace the
//! `todo!()`s in crates/lagwyrm-sim/src/timestep.rs.

use std::time::Duration;

use lagwyrm_sim::{FixedTimestep, Steps};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[test]
fn exact_tick_runs_once() {
    let mut ts = FixedTimestep::new(50, 5); // dt = 20ms
    assert_eq!(
        ts.advance(ms(20)),
        Steps {
            ticks: 1,
            dropped: Duration::ZERO
        }
    );
    assert_eq!(ts.accumulated(), Duration::ZERO);
}

#[test]
fn zero_elapsed_runs_nothing() {
    let mut ts = FixedTimestep::new(50, 5);
    assert_eq!(ts.advance(Duration::ZERO).ticks, 0);
    assert_eq!(ts.alpha(), 0.0);
}

#[test]
fn partial_frames_accumulate() {
    let mut ts = FixedTimestep::new(50, 5);
    assert_eq!(ts.advance(ms(10)).ticks, 0);
    assert_eq!(ts.alpha(), 0.5);
    assert_eq!(ts.advance(ms(10)).ticks, 1);
    assert_eq!(ts.alpha(), 0.0);
}

#[test]
fn long_frame_runs_several_ticks_and_keeps_remainder() {
    let mut ts = FixedTimestep::new(50, 5);
    let steps = ts.advance(ms(65));
    assert_eq!(
        steps,
        Steps {
            ticks: 3,
            dropped: Duration::ZERO
        }
    );
    assert_eq!(ts.accumulated(), ms(5));
    assert_eq!(ts.alpha(), 0.25);
}

#[test]
fn spiral_of_death_is_capped_and_reported() {
    let mut ts = FixedTimestep::new(50, 5);
    // 1010ms is 50 whole ticks plus 10ms. Run 5, drop 45 ticks' worth, keep 10ms.
    let steps = ts.advance(ms(1010));
    assert_eq!(steps.ticks, 5);
    assert_eq!(steps.dropped, ms(900));
    assert_eq!(ts.accumulated(), ms(10));
    // The next normal frame is back to normal.
    assert_eq!(
        ts.advance(ms(10)),
        Steps {
            ticks: 1,
            dropped: Duration::ZERO
        }
    );
}

#[test]
fn sixty_hz_does_not_drift() {
    let mut ts = FixedTimestep::new(60, 5);
    assert_eq!(ts.dt(), Duration::from_nanos(16_666_666));
    let mut ticks = 0u64;
    for _ in 0..10_000 {
        ticks += ts.advance(ms(1)).ticks as u64;
    }
    // 10s at 60Hz with a 16_666_666ns tick is exactly 600 ticks plus 400ns.
    assert_eq!(ticks, 600);
    assert_eq!(ts.accumulated(), Duration::from_nanos(400));
}

#[test]
fn time_is_conserved_under_uneven_frames() {
    let mut ts = FixedTimestep::new(30, 3);
    let frames = [1, 7, 33, 0, 250, 16, 17, 99, 1000, 2, 45, 34];
    let mut fed = Duration::ZERO;
    let mut ticks = 0u32;
    let mut dropped = Duration::ZERO;
    for f in frames {
        let s = ts.advance(ms(f));
        assert!(s.ticks <= 3, "ran {} ticks, max is 3", s.ticks);
        assert!(ts.accumulated() < ts.dt());
        fed += ms(f);
        ticks += s.ticks;
        dropped += s.dropped;
    }
    assert_eq!(ts.dt() * ticks + dropped + ts.accumulated(), fed);
}

#[test]
fn alpha_stays_in_unit_interval() {
    let mut ts = FixedTimestep::new(60, 5);
    for f in [3, 5, 8, 13, 21, 34, 55] {
        ts.advance(ms(f));
        let a = ts.alpha();
        assert!((0.0..1.0).contains(&a), "alpha {a} out of range");
    }
}

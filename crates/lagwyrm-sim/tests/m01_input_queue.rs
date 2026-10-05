//! Module 1 tests for `InputQueue`. These fail until you replace the
//! `todo!()`s in crates/lagwyrm-sim/src/input_queue.rs.

use lagwyrm_sim::{Command, Input, InputQueue, PlayerId, Rejected, Tick};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const P3: PlayerId = PlayerId(3);
const RIGHT: Command = Command {
    move_x: 1,
    move_y: 0,
};
const UP: Command = Command {
    move_x: 0,
    move_y: 1,
};

fn input(player: PlayerId, tick: u64, command: Command) -> Input {
    Input {
        player,
        tick: Tick(tick),
        command,
    }
}

#[test]
fn input_is_applied_on_its_tick_only() {
    let mut q = InputQueue::new(8);
    q.register(P1);
    q.push(input(P1, 2, RIGHT)).unwrap();
    assert_eq!(q.take(Tick(0)), vec![(P1, Command::IDLE)]);
    assert_eq!(q.take(Tick(1)), vec![(P1, Command::IDLE)]);
    assert_eq!(q.take(Tick(2)), vec![(P1, RIGHT)]);
    assert_eq!(q.next_tick(), Tick(3));
}

#[test]
fn missing_input_repeats_last_command() {
    let mut q = InputQueue::new(8);
    q.register(P1);
    q.push(input(P1, 0, RIGHT)).unwrap();
    assert_eq!(q.take(Tick(0)), vec![(P1, RIGHT)]);
    assert_eq!(q.take(Tick(1)), vec![(P1, RIGHT)]);
    assert_eq!(q.take(Tick(2)), vec![(P1, RIGHT)]);
    assert_eq!(q.stats().repeated, 2);
}

#[test]
fn late_input_is_rejected() {
    let mut q = InputQueue::new(8);
    q.register(P1);
    for t in 0..5 {
        q.take(Tick(t));
    }
    assert_eq!(
        q.push(input(P1, 4, RIGHT)),
        Err(Rejected::Late {
            tick: Tick(4),
            next_tick: Tick(5)
        })
    );
    assert_eq!(q.push(input(P1, 5, RIGHT)), Ok(()));
    assert_eq!(q.stats().late, 1);
    assert_eq!(q.stats().accepted, 1);
}

#[test]
fn window_bounds_how_far_ahead_inputs_may_be() {
    let mut q = InputQueue::new(8);
    q.register(P1);
    assert_eq!(q.push(input(P1, 8, RIGHT)), Ok(()));
    assert_eq!(
        q.push(input(P1, 9, RIGHT)),
        Err(Rejected::TooFarAhead {
            tick: Tick(9),
            next_tick: Tick(0)
        })
    );
    assert_eq!(q.stats().too_far_ahead, 1);
}

#[test]
fn first_input_for_a_tick_wins() {
    let mut q = InputQueue::new(8);
    q.register(P1);
    q.push(input(P1, 0, RIGHT)).unwrap();
    assert_eq!(
        q.push(input(P1, 0, UP)),
        Err(Rejected::Duplicate { tick: Tick(0) })
    );
    assert_eq!(q.take(Tick(0)), vec![(P1, RIGHT)]);
    assert_eq!(q.stats().duplicate, 1);
}

#[test]
fn unknown_player_is_rejected() {
    let mut q = InputQueue::new(8);
    assert_eq!(q.push(input(P1, 0, RIGHT)), Err(Rejected::UnknownPlayer));
    assert_eq!(q.stats().unknown_player, 1);
}

#[test]
fn rejection_order_follows_the_enum() {
    // Unknown beats everything, and Late is checked before Duplicate.
    let mut q = InputQueue::new(8);
    assert_eq!(q.push(input(P1, 99, RIGHT)), Err(Rejected::UnknownPlayer));
    q.register(P1);
    q.push(input(P1, 0, RIGHT)).unwrap();
    q.take(Tick(0));
    assert!(matches!(
        q.push(input(P1, 0, UP)),
        Err(Rejected::Late { .. })
    ));
}

#[test]
fn take_is_sorted_by_player_regardless_of_registration_order() {
    let mut q = InputQueue::new(8);
    q.register(P3);
    q.register(P1);
    q.register(P2);
    q.push(input(P2, 0, UP)).unwrap();
    q.push(input(P3, 0, RIGHT)).unwrap();
    assert_eq!(
        q.take(Tick(0)),
        vec![(P1, Command::IDLE), (P2, UP), (P3, RIGHT)]
    );
}

#[test]
fn players_are_independent() {
    let mut q = InputQueue::new(8);
    q.register(P1);
    q.register(P2);
    q.push(input(P1, 0, RIGHT)).unwrap();
    q.push(input(P2, 1, UP)).unwrap();
    assert_eq!(q.take(Tick(0)), vec![(P1, RIGHT), (P2, Command::IDLE)]);
    assert_eq!(q.take(Tick(1)), vec![(P1, RIGHT), (P2, UP)]);
}

#[test]
fn skipping_ticks_discards_stale_inputs() {
    let mut q = InputQueue::new(8);
    q.register(P1);
    q.push(input(P1, 1, UP)).unwrap();
    q.push(input(P1, 3, RIGHT)).unwrap();
    q.push(input(P1, 5, UP)).unwrap();
    assert_eq!(q.pending_len(), 3);
    assert_eq!(q.take(Tick(3)), vec![(P1, RIGHT)]);
    assert_eq!(q.pending_len(), 1, "tick 1's input should be gone");
    assert_eq!(q.next_tick(), Tick(4));
}

#[test]
#[should_panic]
fn going_back_in_time_is_a_caller_bug() {
    let mut q = InputQueue::new(8);
    q.register(P1);
    q.take(Tick(3));
    q.take(Tick(2));
}

#[test]
fn window_slides_with_next_tick() {
    let mut q = InputQueue::new(4);
    q.register(P1);
    assert!(q.push(input(P1, 6, RIGHT)).is_err());
    q.take(Tick(0));
    q.take(Tick(1));
    assert_eq!(q.push(input(P1, 6, RIGHT)), Ok(()));
}

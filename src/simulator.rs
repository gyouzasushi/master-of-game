use crate::game::{Game, Outcome, Player};
use crate::observer::Observer;
use crate::strategy::Strategy;
use rand::RngCore;

pub fn run<G: Game>(
    p1: &mut dyn Strategy<G>,
    p2: &mut dyn Strategy<G>,
    observers: &mut [&mut dyn Observer<G>],
    rng: &mut dyn RngCore,
) -> Outcome {
    let mut state = G::initial_state();
    for o in observers.iter_mut() {
        o.on_start(&state);
    }
    let outcome = loop {
        if let Some(o) = G::outcome(&state) {
            break o;
        }
        let legal = G::legal_actions(&state);
        let action = match G::current_player(&state) {
            Player::P1 => p1.choose(&state, &legal, rng),
            Player::P2 => p2.choose(&state, &legal, rng),
        };
        let next = G::apply(&state, &action, rng);
        for o in observers.iter_mut() {
            o.on_action(&state, &action, &next);
        }
        state = next;
    };
    for o in observers.iter_mut() {
        o.on_end(&state, &outcome);
    }
    outcome
}

use crate::game::{Game, Outcome, Player};
use rand::RngCore;

#[derive(Debug, Clone)]
pub struct NimState {
    pub heaps: Vec<u32>,
    pub next: Player,
}

#[derive(Debug, Clone, Copy)]
pub struct NimAction {
    pub heap: usize,
    pub count: u32,
}

pub struct Nim;

impl Game for Nim {
    type State = NimState;
    type Action = NimAction;

    fn initial_state() -> Self::State {
        NimState { heaps: vec![3, 4, 5], next: Player::P1 }
    }

    fn current_player(state: &Self::State) -> Player {
        state.next
    }

    fn legal_actions(state: &Self::State) -> Vec<Self::Action> {
        let mut acts = Vec::new();
        for (i, &h) in state.heaps.iter().enumerate() {
            for c in 1..=h {
                acts.push(NimAction { heap: i, count: c });
            }
        }
        acts
    }

    fn apply(state: &Self::State, action: &Self::Action, _rng: &mut dyn RngCore) -> Self::State {
        let mut next = state.clone();
        next.heaps[action.heap] -= action.count;
        next.next = state.next.other();
        next
    }

    fn outcome(state: &Self::State) -> Option<Outcome> {
        if state.heaps.iter().all(|&h| h == 0) {
            Some(Outcome::Win(state.next.other()))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observer::Observer;
    use crate::simulator::run;
    use crate::strategy::RandomStrategy;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn legal_action_count_at_start() {
        let s = Nim::initial_state();
        assert_eq!(Nim::legal_actions(&s).len(), 3 + 4 + 5);
    }

    #[test]
    fn apply_decrements_and_switches_player() {
        let s = Nim::initial_state();
        let mut rng = StdRng::seed_from_u64(0);
        let next = Nim::apply(&s, &NimAction { heap: 1, count: 2 }, &mut rng);
        assert_eq!(next.heaps, vec![3, 2, 5]);
        assert_eq!(next.next, Player::P2);
    }

    #[test]
    fn outcome_when_all_empty() {
        let s = NimState { heaps: vec![0, 0, 0], next: Player::P2 };
        assert_eq!(Nim::outcome(&s), Some(Outcome::Win(Player::P1)));
    }

    #[test]
    fn random_vs_random_terminates() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut p1 = RandomStrategy;
        let mut p2 = RandomStrategy;
        let mut observers: Vec<&mut dyn Observer<Nim>> = vec![];
        let outcome = run::<Nim>(&mut p1, &mut p2, &mut observers, &mut rng);
        assert!(matches!(outcome, Outcome::Win(_)));
    }
}

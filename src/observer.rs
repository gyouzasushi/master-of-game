use crate::game::{Game, Outcome};

pub trait Observer<G: Game> {
    fn on_start(&mut self, _state: &G::State) {}
    fn on_action(&mut self, _before: &G::State, _action: &G::Action, _after: &G::State) {}
    fn on_end(&mut self, _state: &G::State, _outcome: &Outcome) {}
}

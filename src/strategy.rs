use crate::game::Game;
use rand::seq::SliceRandom;
use rand::RngCore;
use std::marker::PhantomData;

pub trait Strategy<G: Game> {
    fn choose(&mut self, state: &G::State, legal: &[G::Action], rng: &mut dyn RngCore) -> G::Action;
}

pub struct RandomStrategy;

impl<G: Game> Strategy<G> for RandomStrategy {
    fn choose(&mut self, _state: &G::State, legal: &[G::Action], rng: &mut dyn RngCore) -> G::Action {
        legal.choose(rng).expect("no legal actions").clone()
    }
}

pub trait Prompt<G: Game> {
    fn select(&mut self, state: &G::State, legal: &[G::Action]) -> usize;
}

pub struct ManualStrategy<G: Game, P: Prompt<G>> {
    pub prompt: P,
    _g: PhantomData<G>,
}

impl<G: Game, P: Prompt<G>> ManualStrategy<G, P> {
    pub fn new(prompt: P) -> Self {
        Self { prompt, _g: PhantomData }
    }
}

impl<G: Game, P: Prompt<G>> Strategy<G> for ManualStrategy<G, P> {
    fn choose(&mut self, state: &G::State, legal: &[G::Action], _rng: &mut dyn RngCore) -> G::Action {
        let idx = self.prompt.select(state, legal);
        legal[idx].clone()
    }
}

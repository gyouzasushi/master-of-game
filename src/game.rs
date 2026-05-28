use rand::RngCore;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    P1,
    P2,
}

impl Player {
    pub fn other(self) -> Self {
        match self {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Win(Player),
    Draw,
}

pub trait Game {
    type State: Clone;
    type Action: Clone;

    fn initial_state() -> Self::State;
    fn current_player(state: &Self::State) -> Player;
    fn legal_actions(state: &Self::State) -> Vec<Self::Action>;
    fn apply(state: &Self::State, action: &Self::Action, rng: &mut dyn RngCore) -> Self::State;
    fn outcome(state: &Self::State) -> Option<Outcome>;
}

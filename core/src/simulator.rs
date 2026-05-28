//! 対局ループ本体。

use crate::game::{Game, Outcome, Player};
use crate::observer::Observer;
use crate::strategy::Strategy;
use rand::RngCore;

/// 1ゲームを最後まで進めて結果を返す。
///
/// `p1`, `p2` はそれぞれ [`Player::P1`], [`Player::P2`] 側の戦略。`observers` には任意個の
/// 観測者を登録でき、全員に対して `on_start` → `on_action` ×手数 → `on_end` の順で
/// コールバックが呼ばれる。
///
/// 各手番の処理は次の流れ:
///
/// 1. [`Game::outcome`] で終局判定。終局なら全 observer の `on_end` を呼んで返る。
/// 2. [`Game::legal_actions`] で合法手を列挙し、[`Game::current_player`] が示す側の `Strategy`
///    に `choose` させる。
/// 3. [`Game::apply`] で次状態に遷移し、全 observer の `on_action` を呼ぶ。
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

//! 状態から行動を返す戦略の trait と、汎用実装。

use crate::game::Game;
use rand::seq::SliceRandom;
use rand::RngCore;
use std::marker::PhantomData;

/// 状態と合法手を見て、行動を1つ選ぶ。
///
/// AI、ランダム、人間入力（[`ManualStrategy`]）など、どんな選び方も同じ trait の背後に置ける。
/// [`crate::simulator::run`] には `&mut dyn Strategy<G>` として渡される。
pub trait Strategy<G: Game> {
    /// `legal` の中から1つ選んで返す。返り値は必ず `legal` に含まれていなければならない。
    ///
    /// 確率的な戦略のため `rng` を受け取るが、決定的な戦略では無視してよい。
    fn choose(&mut self, state: &G::State, legal: &[G::Action], rng: &mut dyn RngCore) -> G::Action;
}

/// 合法手から一様ランダムに1つ選ぶ戦略。
///
/// 動作確認用、もしくは弱いベースライン対戦相手として使う。
pub struct RandomStrategy;

impl<G: Game> Strategy<G> for RandomStrategy {
    fn choose(&mut self, _state: &G::State, legal: &[G::Action], rng: &mut dyn RngCore) -> G::Action {
        legal.choose(rng).expect("no legal actions").clone()
    }
}

/// 人間に手を選ばせるための入出力インターフェース。
///
/// 表示と入力の手段（標準入出力、TUI、GUI など）は実装側が決める。[`ManualStrategy`] と
/// 組み合わせて使うことで、[`Strategy`] として振る舞わせられる。
pub trait Prompt<G: Game> {
    /// 合法手のうち、ユーザーが選んだものの添字を返す。
    ///
    /// 返り値は `0..legal.len()` の範囲に収める。範囲外の値を返したときは
    /// [`ManualStrategy::choose`] がパニックする。
    fn select(&mut self, state: &G::State, legal: &[G::Action]) -> usize;
}

/// [`Prompt`] を介して人間に手を選ばせる戦略。
pub struct ManualStrategy<G: Game, P: Prompt<G>> {
    /// 入出力インターフェース。
    pub prompt: P,
    _g: PhantomData<G>,
}

impl<G: Game, P: Prompt<G>> ManualStrategy<G, P> {
    /// `prompt` を使う戦略を生成する。
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

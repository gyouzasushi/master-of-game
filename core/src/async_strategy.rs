//! 非同期版の戦略 trait と、sync 戦略からの変換ラッパ。
//!
//! [`Strategy`] が「即座に決定する」抽象なのに対し、[`AsyncStrategy`] は「いつか決定する」抽象。
//! 人間のクリック入力のようにイベント到来を待つ戦略を、AI 戦略と同じ形で扱うためにある。
//!
//! sync な [`Strategy`] は [`AsAsync`] でラップすれば [`AsyncStrategy`] として使える。
//! 多くの場合 [`into_async`] ヘルパで `Box<dyn AsyncStrategy<G>>` に直接変換できる。
//!
//! ## なぜ blanket impl にしないか
//!
//! `impl<S: Strategy<G>> AsyncStrategy<G> for S` を書くと、ダウンストリームが `Strategy` を
//! 後から実装する可能性に対する coherence エラーになる。ラッパ経由なら衝突しない。

use crate::game::Game;
use crate::strategy::Strategy;
use async_trait::async_trait;
use rand::RngCore;

/// 非同期に行動を返す戦略。
///
/// `Strategy::choose` が同期で返るのに対し、`AsyncStrategy::choose` は `Future` を返すため、
/// クリックや network I/O の完了などを待ってから返す戦略を表現できる。
#[async_trait(?Send)]
pub trait AsyncStrategy<G: Game> {
    async fn choose(
        &mut self,
        state: &G::State,
        legal: &[G::Action],
        rng: &mut dyn RngCore,
    ) -> G::Action;
}

/// sync な [`Strategy`] を [`AsyncStrategy`] として扱うためのラッパ。
pub struct AsAsync<S>(pub S);

#[async_trait(?Send)]
impl<G, S> AsyncStrategy<G> for AsAsync<S>
where
    G: Game,
    S: Strategy<G>,
{
    async fn choose(
        &mut self,
        state: &G::State,
        legal: &[G::Action],
        rng: &mut dyn RngCore,
    ) -> G::Action {
        Strategy::choose(&mut self.0, state, legal, rng)
    }
}

/// sync 戦略を `Box<dyn AsyncStrategy<G>>` に変換する便利関数。
pub fn into_async<G, S>(s: S) -> Box<dyn AsyncStrategy<G>>
where
    G: Game + 'static,
    S: Strategy<G> + 'static,
{
    Box::new(AsAsync(s))
}

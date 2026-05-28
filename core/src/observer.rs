//! 対局進行を観測するためのフック。

use crate::game::{Game, Outcome};

/// シミュレータが主要な節目で呼び出すコールバック群。
///
/// 描画、棋譜の記録、統計の収集など、状態を変えずに進行を見守る処理を実装する。
/// 全メソッドに空のデフォルト実装があるので、必要なものだけ上書きすればよい。
///
/// [`crate::simulator::run`] には `&mut [&mut dyn Observer<G>]` として渡され、登録された全員が
/// 順に呼ばれる。
pub trait Observer<G: Game> {
    /// 初期状態が決まった直後に1回呼ばれる。
    fn on_start(&mut self, _state: &G::State) {}

    /// 1手指された直後に呼ばれる。`before` は遷移前、`after` は遷移後の状態。
    fn on_action(&mut self, _before: &G::State, _action: &G::Action, _after: &G::State) {}

    /// 終局直後に1回呼ばれる。
    fn on_end(&mut self, _state: &G::State, _outcome: &Outcome) {}
}

//! ルールを表現するための基本型と trait。

use rand::RngCore;

/// 手番を表す。2人ゲーム固定。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    /// 先手。
    P1,
    /// 後手。
    P2,
}

impl Player {
    /// 相手側の手番。
    pub fn other(self) -> Self {
        match self {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        }
    }
}

/// ゲームの結果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// 勝者あり。
    Win(Player),
    /// 引き分け。
    Draw,
}

/// ゲームのルール一式を定義する trait。
///
/// 状態の型 [`State`] と行動の型 [`Action`] を関連型として与え、合法手の列挙、状態遷移、
/// 終局判定を関連関数として実装する。これだけ揃えば [`crate::simulator::run`] で対局を回せる。
///
/// 手番情報は状態側に持たせる（[`current_player`] で取り出す）。連続手番や任意の手番遷移を
/// ルール側で自由に表現できる。
///
/// [`apply`] には `&mut dyn RngCore` を渡せるので、確率的な遷移を含むゲーム（例: バックギャモン）
/// にも対応する。決定的なゲームでは無視してよい。
///
/// # 実装例
///
/// 簡単なサンプル実装は [`crate::games::nim`] を参照。
///
/// [`State`]: Game::State
/// [`Action`]: Game::Action
/// [`current_player`]: Game::current_player
/// [`apply`]: Game::apply
pub trait Game {
    /// ゲーム状態の型。
    type State: Clone;

    /// 行動の型。
    type Action: Clone;

    /// 初期状態を返す。
    fn initial_state() -> Self::State;

    /// `state` における手番。
    fn current_player(state: &Self::State) -> Player;

    /// `state` における合法手のリスト。終局状態では空でもよい。
    fn legal_actions(state: &Self::State) -> Vec<Self::Action>;

    /// `state` に `action` を適用した次状態。
    ///
    /// 渡される `action` は [`legal_actions`] が返したものに限る前提。範囲外の行動を渡したときの
    /// 挙動は実装側に委ねる（パニックでも未定義状態を返すのでも構わない）。
    ///
    /// [`legal_actions`]: Game::legal_actions
    fn apply(state: &Self::State, action: &Self::Action, rng: &mut dyn RngCore) -> Self::State;

    /// 終局していれば結果を、続いていれば `None` を返す。
    fn outcome(state: &Self::State) -> Option<Outcome>;
}

//! 2人対戦ゲームの汎用シミュレータ。
//!
//! ルールは [`Game`]、戦略は [`Strategy`]、進行の観測（描画やログ）は [`Observer`] でそれぞれ
//! 独立に定義し、[`simulator::run`] に渡して1ゲームを駆動する。
//!
//! # 構成
//!
//! - [`game`] — [`Game`] trait、[`Player`]、[`Outcome`]
//! - [`strategy`] — [`Strategy`] trait と汎用実装（[`RandomStrategy`]、[`ManualStrategy`]）
//! - [`observer`] — [`Observer`] trait
//! - [`simulator`] — 対局ループ [`simulator::run`]
//! - [`games`] — サンプルゲーム（[`games::nim`]）
//!
//! # 例
//!
//! Nim をランダム同士で1局回す:
//!
//! ```
//! use master_of_game_core::games::nim::Nim;
//! use master_of_game_core::observer::Observer;
//! use master_of_game_core::simulator::run;
//! use master_of_game_core::strategy::RandomStrategy;
//! use rand::SeedableRng;
//!
//! let mut rng = rand::rngs::StdRng::seed_from_u64(0);
//! let mut p1 = RandomStrategy;
//! let mut p2 = RandomStrategy;
//! let mut observers: Vec<&mut dyn Observer<Nim>> = vec![];
//! let outcome = run::<Nim>(&mut p1, &mut p2, &mut observers, &mut rng);
//! ```
//!
//! [`Game`]: game::Game
//! [`Player`]: game::Player
//! [`Outcome`]: game::Outcome
//! [`Strategy`]: strategy::Strategy
//! [`RandomStrategy`]: strategy::RandomStrategy
//! [`ManualStrategy`]: strategy::ManualStrategy
//! [`Observer`]: observer::Observer

pub mod game;
pub mod games;
pub mod observer;
pub mod simulator;
pub mod strategy;

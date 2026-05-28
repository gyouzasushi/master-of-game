//! Leptos ベースの GUI runner。
//!
//! [`GuiView`] を実装すれば、[`run_in_browser`] でブラウザで動く対戦 GUI になる。
//! 戦略は [`AsyncStrategy`] として扱われ、AI も人間入力 ([`ManualGui`]) も同じ抽象の下で
//! 統一的に扱える。

use async_trait::async_trait;
use futures_channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures_util::StreamExt;
use leptos::prelude::*;
use master_of_game_core::async_strategy::AsyncStrategy;
use master_of_game_core::game::{Game, Outcome, Player};
use rand::rngs::SmallRng;
use rand::RngCore;
use rand::SeedableRng;
use std::cell::RefCell;
use std::rc::Rc;

/// 状態と合法手から GUI を描画する trait。
pub trait GuiView<G: Game> {
    fn view(
        state: G::State,
        legal: Vec<G::Action>,
        on_action: Callback<G::Action>,
    ) -> impl IntoView;
}

/// 画面 → 戦略 のアクション入力を仲介する mpsc チャンネル。
///
/// view 側からは [`send`](Input::send) で送り、[`ManualGui`] が内部の receiver を await して
/// 受け取る。両者 Human の場合は同じ `Input` から2つの `ManualGui` を作り、receiver を Rc で
/// 共有する (ターン制なので同時に await されない)。
///
/// `waiting()` で「いま人間入力待ちか?」を reactive に観測でき、view 側がボタン表示と
/// 「考慮中」を切替えるのに使える。
pub struct Input<A: Send + 'static> {
    tx: UnboundedSender<A>,
    rx: Rc<RefCell<UnboundedReceiver<A>>>,
    waiting: RwSignal<bool, LocalStorage>,
}

impl<A: Send + 'static> Input<A> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded();
        Self {
            tx,
            rx: Rc::new(RefCell::new(rx)),
            waiting: RwSignal::new_local(false),
        }
    }

    /// view 側からアクションを送信する。
    pub fn send(&self, action: A) {
        let _ = self.tx.unbounded_send(action);
    }

    /// 「いま入力待機中?」signal。
    pub fn waiting(&self) -> RwSignal<bool, LocalStorage> {
        self.waiting
    }
}

impl<A: Send + 'static> Default for Input<A> {
    fn default() -> Self {
        Self::new()
    }
}

/// 人間入力 (画面クリック等) を行動として返す戦略。
///
/// 内部で `Input` の receiver を `next().await` する。
pub struct ManualGui<G: Game>
where
    G::Action: Send + 'static,
{
    rx: Rc<RefCell<UnboundedReceiver<G::Action>>>,
    waiting: RwSignal<bool, LocalStorage>,
}

impl<G: Game> ManualGui<G>
where
    G::Action: Send + 'static,
{
    pub fn new(input: &Input<G::Action>) -> Self {
        Self {
            rx: input.rx.clone(),
            waiting: input.waiting,
        }
    }
}

#[async_trait(?Send)]
impl<G: Game> AsyncStrategy<G> for ManualGui<G>
where
    G::Action: Send + 'static,
{
    async fn choose(
        &mut self,
        _state: &G::State,
        _legal: &[G::Action],
        _rng: &mut dyn RngCore,
    ) -> G::Action {
        self.waiting.set(true);
        let action = self
            .rx
            .borrow_mut()
            .next()
            .await
            .expect("input channel closed");
        self.waiting.set(false);
        action
    }
}

/// 1ゲームをブラウザ上で動かす。
///
/// `p1`, `p2` はそれぞれの戦略 (Human / AI を問わず [`AsyncStrategy`])。`input` は画面からの
/// アクションを受ける channel で、`ManualGui` には事前にこの `input` から構築させておく。
pub fn run_in_browser<G, V>(
    _view: V,
    p1: Box<dyn AsyncStrategy<G>>,
    p2: Box<dyn AsyncStrategy<G>>,
    input: Input<G::Action>,
) where
    G: Game + 'static,
    G::State: Clone + 'static,
    G::Action: Clone + Send + 'static,
    V: GuiView<G> + 'static,
{
    console_error_panic_hook::set_once();

    let state = RwSignal::new_local(G::initial_state());
    let rng: Rc<RefCell<SmallRng>> = Rc::new(RefCell::new(SmallRng::from_entropy()));

    let tx = input.tx.clone();
    let waiting = input.waiting;

    let on_action: Callback<G::Action> = Callback::new(move |action: G::Action| {
        let _ = tx.unbounded_send(action);
    });

    let rng_for_loop = rng.clone();
    wasm_bindgen_futures::spawn_local(async move {
        let mut p1 = p1;
        let mut p2 = p2;
        loop {
            let s = state.get_untracked();
            if G::outcome(&s).is_some() {
                break;
            }
            let legal = G::legal_actions(&s);
            let player = G::current_player(&s);
            let action = {
                let mut rng_ref = rng_for_loop.borrow_mut();
                let rng_dyn: &mut dyn RngCore = &mut *rng_ref;
                match player {
                    Player::P1 => p1.choose(&s, &legal, rng_dyn).await,
                    Player::P2 => p2.choose(&s, &legal, rng_dyn).await,
                }
            };
            let next = {
                let mut rng_ref = rng_for_loop.borrow_mut();
                G::apply(&s, &action, &mut *rng_ref)
            };
            state.set(next);
        }
    });

    leptos::mount::mount_to_body(move || {
        view! {
            <div>
                {move || {
                    let s = state.get();
                    if let Some(outcome) = G::outcome(&s) {
                        return view! {
                            <div class="game-over">
                                <h2>{format_outcome(&outcome)}</h2>
                            </div>
                        }.into_any();
                    }
                    if !waiting.get() {
                        return view! { <div>"考慮中..."</div> }.into_any();
                    }
                    let legal = G::legal_actions(&s);
                    V::view(s, legal, on_action).into_any()
                }}
            </div>
        }
    });
}

fn format_outcome(o: &Outcome) -> String {
    match o {
        Outcome::Win(Player::P1) => "P1 の勝ち".to_string(),
        Outcome::Win(Player::P2) => "P2 の勝ち".to_string(),
        Outcome::Draw => "引き分け".to_string(),
    }
}

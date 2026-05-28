//! Leptos ベースの GUI runner。
//!
//! [`GuiView`] を実装すれば、[`run_in_browser`] でブラウザで動く対戦 GUI になる。
//! 戦略は [`AsyncStrategy`] として扱われ、AI も人間入力 ([`ManualGui`]) も同じ抽象の下で
//! 統一的に扱える。

use async_trait::async_trait;
use futures_channel::oneshot;
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

/// 画面からのアクション入力を仲介する単発 channel。
///
/// `submit(action)` を view の click ハンドラから呼び、`next().await` を [`ManualGui::choose`]
/// から呼ぶ。両者 Human の場合は同じ `InputChannel` を複製して両方の `ManualGui` に渡す
/// (ターン制なので同時には awaiting されない)。
///
/// 内部状態は Leptos の `StoredValue<_, LocalStorage>` に格納するため、Send + Sync を満たし
/// `Callback` から扱える。
pub struct InputChannel<A: 'static> {
    pending: StoredValue<Option<oneshot::Sender<A>>, LocalStorage>,
    waiting: RwSignal<bool, LocalStorage>,
}

impl<A: 'static> Copy for InputChannel<A> {}
impl<A: 'static> Clone for InputChannel<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: 'static> Default for InputChannel<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: 'static> InputChannel<A> {
    pub fn new() -> Self {
        Self {
            pending: StoredValue::new_local(None),
            waiting: RwSignal::new_local(false),
        }
    }

    /// view 側から手の入力を投入する。
    pub fn submit(&self, action: A) {
        self.pending.update_value(|p| {
            if let Some(tx) = p.take() {
                let _ = tx.send(action);
            }
        });
    }

    /// 次の入力が来るまで待つ。
    pub async fn next(&self) -> A {
        let (tx, rx) = oneshot::channel();
        self.pending.update_value(|p| *p = Some(tx));
        self.waiting.set(true);
        let result = rx.await.expect("input channel closed");
        self.waiting.set(false);
        result
    }

    /// 「いま入力を待っているか?」を reactive に観測するための signal。
    pub fn waiting(&self) -> RwSignal<bool, LocalStorage> {
        self.waiting
    }
}

/// 人間入力 (画面クリック等) を行動として返す戦略。
///
/// `choose` は内部で `InputChannel::next` を await するため、`AsyncStrategy` としてのみ実装される。
pub struct ManualGui<G: Game>
where
    G::Action: 'static,
{
    input: InputChannel<G::Action>,
}

impl<G: Game> ManualGui<G>
where
    G::Action: 'static,
{
    pub fn new(input: InputChannel<G::Action>) -> Self {
        Self { input }
    }
}

#[async_trait(?Send)]
impl<G: Game> AsyncStrategy<G> for ManualGui<G>
where
    G::Action: 'static,
{
    async fn choose(
        &mut self,
        _state: &G::State,
        _legal: &[G::Action],
        _rng: &mut dyn RngCore,
    ) -> G::Action {
        self.input.next().await
    }
}

/// 1ゲームをブラウザ上で動かす。
///
/// `p1`, `p2` はそれぞれの戦略 (Human/AI を問わず `AsyncStrategy`)。`input` は画面からの
/// アクション入力を受ける channel で、`ManualGui` には事前にこの channel を持たせておく。
pub fn run_in_browser<G, V>(
    _view: V,
    p1: Box<dyn AsyncStrategy<G>>,
    p2: Box<dyn AsyncStrategy<G>>,
    input: InputChannel<G::Action>,
) where
    G: Game + 'static,
    G::State: Clone + 'static,
    G::Action: Clone + 'static,
    V: GuiView<G> + 'static,
{
    console_error_panic_hook::set_once();

    let state = RwSignal::new_local(G::initial_state());
    let rng: Rc<RefCell<SmallRng>> = Rc::new(RefCell::new(SmallRng::from_entropy()));

    let on_action: Callback<G::Action> = Callback::new(move |action: G::Action| {
        input.submit(action);
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

    let waiting = input.waiting();
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

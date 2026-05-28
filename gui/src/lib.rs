//! Leptos ベースの GUI runner。
//!
//! [`GuiView`] を実装すれば、[`run_in_browser`] でそのまま遊べる GUI として動く。

use leptos::prelude::*;
use master_of_game_core::game::{Game, Outcome, Player};
use master_of_game_core::strategy::Strategy;
use rand::rngs::SmallRng;
use rand::SeedableRng;

/// 状態と合法手から GUI を描画するための trait。
///
/// `view` は状態を見て DOM を構築し、ユーザがアクションを選んだら `on_action` を呼ぶ。
pub trait GuiView<G: Game> {
    fn view(
        state: G::State,
        legal: Vec<G::Action>,
        on_action: Callback<G::Action>,
    ) -> impl IntoView;
}

/// P1=人間 / P2=AI で1ゲームをブラウザ上で動かす。
///
/// 状態を reactive な signal で保持し、ユーザ操作 → `Game::apply` → AI 自動応手 → 再描画
/// のループを Leptos のリアクティビティに乗せて回す。
pub fn run_in_browser<G, V, AI>(_view: V, ai: AI)
where
    G: Game + 'static,
    G::State: Clone + 'static,
    G::Action: Clone + 'static,
    V: GuiView<G> + 'static,
    AI: Strategy<G> + 'static,
{
    console_error_panic_hook::set_once();

    let state = RwSignal::new_local(G::initial_state());
    let engine = StoredValue::new_local((ai, SmallRng::from_entropy()));

    let on_action: Callback<G::Action> = Callback::new(move |action: G::Action| {
        state.update(|s| {
            engine.update_value(|(_, r)| {
                *s = G::apply(s, &action, r);
            });
        });
        // AI 手番が続く間、自動で進める
        loop {
            let s = state.get_untracked();
            if G::outcome(&s).is_some() {
                break;
            }
            if G::current_player(&s) == Player::P1 {
                break;
            }
            let legal = G::legal_actions(&s);
            let mut next = s.clone();
            engine.update_value(|(a, r)| {
                let chosen = a.choose(&s, &legal, r);
                next = G::apply(&s, &chosen, r);
            });
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
                    if G::current_player(&s) != Player::P1 {
                        return view! { <div>"AI 思考中..."</div> }.into_any();
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
        Outcome::Win(Player::P1) => "あなたの勝ち!".to_string(),
        Outcome::Win(Player::P2) => "AI の勝ち".to_string(),
        Outcome::Draw => "引き分け".to_string(),
    }
}

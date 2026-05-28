//! Leptos ベースの GUI runner。
//!
//! [`GuiView`] を実装すれば、[`run_in_browser`] でそのまま遊べる GUI として動く。
//! 各プレイヤーは [`PlayerKind`] で「人間入力」か「AI 戦略」かを選べる。

use leptos::prelude::*;
use master_of_game_core::game::{Game, Outcome, Player};
use master_of_game_core::strategy::Strategy;
use rand::rngs::SmallRng;
use rand::SeedableRng;

/// 状態と合法手から GUI を描画する trait。
///
/// `view` は状態を見て DOM を構築し、ユーザがアクションを選んだら `on_action` を呼ぶ。
pub trait GuiView<G: Game> {
    fn view(
        state: G::State,
        legal: Vec<G::Action>,
        on_action: Callback<G::Action>,
    ) -> impl IntoView;
}

/// プレイヤーの種別。
pub enum PlayerKind<G: Game> {
    /// 人間入力。GUI 側のクリック等で手を決める。
    Human,
    /// AI 戦略。自動で手を決める。
    Ai(Box<dyn Strategy<G>>),
}

/// 1ゲームをブラウザ上で動かす。
///
/// 状態を reactive な signal で保持し、Human 手番では view を表示してクリックを待つ、
/// AI 手番では自動で `Strategy::choose` → `Game::apply` を回す。両方 Human なら毎手 view を、
/// 両方 AI なら全自動で終局まで進む。
pub fn run_in_browser<G, V>(
    _view: V,
    p1_kind: PlayerKind<G>,
    p2_kind: PlayerKind<G>,
)
where
    G: Game + 'static,
    G::State: Clone + 'static,
    G::Action: Clone + 'static,
    V: GuiView<G> + 'static,
{
    console_error_panic_hook::set_once();

    let p1_human = matches!(p1_kind, PlayerKind::Human);
    let p2_human = matches!(p2_kind, PlayerKind::Human);
    let p1_ai = if let PlayerKind::Ai(a) = p1_kind { Some(a) } else { None };
    let p2_ai = if let PlayerKind::Ai(a) = p2_kind { Some(a) } else { None };

    let state = RwSignal::new_local(G::initial_state());
    let engine = StoredValue::new_local((p1_ai, p2_ai, SmallRng::from_entropy()));

    let is_human = move |p: Player| match p {
        Player::P1 => p1_human,
        Player::P2 => p2_human,
    };

    let auto_play = move || loop {
        let s = state.get_untracked();
        if G::outcome(&s).is_some() {
            break;
        }
        let player = G::current_player(&s);
        if is_human(player) {
            break;
        }
        let legal = G::legal_actions(&s);
        let mut next: Option<G::State> = None;
        engine.update_value(|(p1_opt, p2_opt, rng)| {
            let strat = match player {
                Player::P1 => p1_opt.as_mut(),
                Player::P2 => p2_opt.as_mut(),
            };
            if let Some(s_box) = strat {
                let chosen = s_box.choose(&s, &legal, rng);
                next = Some(G::apply(&s, &chosen, rng));
            }
        });
        match next {
            Some(n) => state.set(n),
            None => break,
        }
    };

    auto_play();

    let on_action: Callback<G::Action> = Callback::new(move |action: G::Action| {
        state.update(|s| {
            engine.update_value(|(_, _, r)| {
                *s = G::apply(s, &action, r);
            });
        });
        auto_play();
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
                    let player = G::current_player(&s);
                    if !is_human(player) {
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
        Outcome::Win(Player::P1) => "P1 の勝ち".to_string(),
        Outcome::Win(Player::P2) => "P2 の勝ち".to_string(),
        Outcome::Draw => "引き分け".to_string(),
    }
}

//! Nim の Leptos GUI バイナリ。
//!
//! `trunk serve` で起動するとブラウザで対戦できる。
//! 対戦の組合せは `main` で `PlayerKind` を切り替えて指定する。

use leptos::prelude::*;
use master_of_game_core::games::nim::{Nim, NimAction, NimState};
use master_of_game_core::strategy::RandomStrategy;
use master_of_game_gui::{run_in_browser, GuiView, PlayerKind};

struct NimView;

impl GuiView<Nim> for NimView {
    fn view(
        state: NimState,
        legal: Vec<NimAction>,
        on_action: Callback<NimAction>,
    ) -> impl IntoView {
        let heaps: Vec<(usize, u32)> = state.heaps.iter().copied().enumerate().collect();
        let turn = format!("{:?} の手番", state.next);
        view! {
            <div class="nim">
                <h2>{turn}</h2>
                <div class="board">
                    {heaps.into_iter().map(|(i, n)| view! {
                        <div class="heap">
                            <span class="label">{format!("heap {}: ", i)}</span>
                            <span class="stones">{"*".repeat(n as usize)}</span>
                            <span class="count">{format!(" ({})", n)}</span>
                        </div>
                    }).collect_view()}
                </div>
                <div class="actions">
                    {legal.into_iter().map(|a| view! {
                        <button on:click=move |_| on_action.run(a)>
                            {format!("heap {} から {} 個", a.heap, a.count)}
                        </button>
                    }).collect_view()}
                </div>
            </div>
        }
    }
}

fn main() {
    // 組合せはここで決める。任意に書き換え可:
    //   PlayerKind::Human                                 — 入力
    //   PlayerKind::Ai(Box::new(RandomStrategy))          — ランダムAI
    run_in_browser::<Nim, _>(
        NimView,
        PlayerKind::Human,
        PlayerKind::Ai(Box::new(RandomStrategy)),
    );
}

//! Nim の Leptos GUI バイナリ。
//!
//! `trunk serve` で起動するとブラウザで対戦できる。

use leptos::prelude::*;
use master_of_game_core::games::nim::{Nim, NimAction, NimState};
use master_of_game_core::strategy::RandomStrategy;
use master_of_game_gui::{run_in_browser, GuiView};

struct NimView;

impl GuiView<Nim> for NimView {
    fn view(
        state: NimState,
        legal: Vec<NimAction>,
        on_action: Callback<NimAction>,
    ) -> impl IntoView {
        let heaps: Vec<(usize, u32)> = state.heaps.iter().copied().enumerate().collect();
        let actions: Vec<NimAction> = legal;
        view! {
            <div class="nim">
                <h2>"あなたの手番"</h2>
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
                    {actions.into_iter().map(|a| view! {
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
    run_in_browser::<Nim, _, _>(NimView, RandomStrategy);
}

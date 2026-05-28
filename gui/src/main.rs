//! Nim の Leptos GUI バイナリ。
//!
//! `trunk serve` で起動するとブラウザで対戦できる。
//! main で各プレイヤーに `ManualGui` (入力) か AI 戦略を割り当てる。

use leptos::prelude::*;
use master_of_game_core::async_strategy::{into_async, AsyncStrategy};
use master_of_game_core::games::nim::{Nim, NimAction, NimState};
use master_of_game_core::strategy::RandomStrategy;
use master_of_game_gui::{run_in_browser, GuiView, Input, ManualGui};

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
    // 画面 → 戦略 の入力チャンネル。
    // Human プレイヤーには &input から ManualGui を作って渡す。
    let input: Input<NimAction> = Input::new();

    // P1=人間, P2=ランダムAI
    let p1: Box<dyn AsyncStrategy<Nim>> = Box::new(ManualGui::new(&input));
    let p2: Box<dyn AsyncStrategy<Nim>> = into_async(RandomStrategy);

    run_in_browser::<Nim, _>(NimView, p1, p2, input);
}

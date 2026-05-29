//! ratatui で動く TUI runner。
//!
//! ゲームごとに [`TuiView`] を実装すると、`simulator::run` の Strategy / Observer に
//! [`ManualTui`] と [`TuiObserver`] として組み込める。端末ライフサイクルは [`TuiApp`] が抱える。

use crossterm::event::{self, Event, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use master_of_game_core::game::{Game, Outcome};
use master_of_game_core::observer::Observer;
use master_of_game_core::simulator;
use master_of_game_core::strategy::Strategy;
use rand::RngCore;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::Frame;
use ratatui::Terminal;
use std::cell::RefCell;
use std::io;
use std::marker::PhantomData;
use std::rc::Rc;

/// 盤面の描画とキー入力の解釈。
///
/// `render` は状態変化時に呼ばれる。`handle_key` はキー押下時に呼ばれ、アクションに
/// 翻訳できれば返す。
pub trait TuiView<G: Game> {
    /// 状態を描画する。
    fn render(state: &G::State, legal: &[G::Action], frame: &mut Frame, area: Rect);

    /// キーをアクションに翻訳する。該当なしは `None`。
    fn handle_key(
        state: &G::State,
        legal: &[G::Action],
        key: KeyEvent,
    ) -> Option<G::Action>;
}

type SharedTerminal = Rc<RefCell<Terminal<CrosstermBackend<io::Stdout>>>>;

/// 端末のライフサイクルを RAII で管理する。
///
/// 構築時に raw mode と alternate screen に入り、`Drop` で抜ける。panic でも unwind 中に
/// `Drop` が走るので、端末が raw mode のまま残らない。
pub struct TuiApp<G: Game, V: TuiView<G>> {
    terminal: SharedTerminal,
    _marker: PhantomData<(fn() -> G, fn() -> V)>,
}

impl<G: Game, V: TuiView<G>> TuiApp<G, V> {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            terminal: Rc::new(RefCell::new(terminal)),
            _marker: PhantomData,
        })
    }

    /// 1ゲームを最後まで進めて結果を返す。
    ///
    /// 内部で [`TuiObserver`] を立てて [`simulator::run`] に渡す。
    pub fn run(
        &mut self,
        p1: &mut dyn Strategy<G>,
        p2: &mut dyn Strategy<G>,
    ) -> Outcome {
        let mut observer = TuiObserver::<G, V>::new(self.terminal.clone());
        let mut observers: Vec<&mut dyn Observer<G>> = vec![&mut observer];
        let mut rng = rand::thread_rng();
        simulator::run::<G>(p1, p2, &mut observers, &mut rng)
    }
}

impl<G: Game, V: TuiView<G>> Drop for TuiApp<G, V> {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

/// 状態変化のたびに [`TuiView::render`] を呼ぶ Observer。
pub struct TuiObserver<G: Game, V: TuiView<G>> {
    terminal: SharedTerminal,
    _marker: PhantomData<(fn() -> G, fn() -> V)>,
}

impl<G: Game, V: TuiView<G>> TuiObserver<G, V> {
    pub fn new(terminal: SharedTerminal) -> Self {
        Self {
            terminal,
            _marker: PhantomData,
        }
    }

    fn draw(&mut self, state: &G::State) {
        let legal = G::legal_actions(state);
        let _ = self.terminal.borrow_mut().draw(|frame| {
            let area = frame.area();
            V::render(state, &legal, frame, area);
        });
    }
}

impl<G: Game, V: TuiView<G>> Observer<G> for TuiObserver<G, V> {
    fn on_start(&mut self, state: &G::State) {
        self.draw(state);
    }

    fn on_action(&mut self, _before: &G::State, _action: &G::Action, after: &G::State) {
        self.draw(after);
    }

    fn on_end(&mut self, state: &G::State, _outcome: &Outcome) {
        self.draw(state);
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

/// キー入力を blocking で待ち、[`TuiView::handle_key`] でアクションに翻訳する Strategy。
///
/// `crossterm::event::read` で待機する。
pub struct ManualTui<G: Game, V: TuiView<G>> {
    _marker: PhantomData<(fn() -> G, fn() -> V)>,
}

impl<G: Game, V: TuiView<G>> ManualTui<G, V> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<G: Game, V: TuiView<G>> Default for ManualTui<G, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<G: Game, V: TuiView<G>> Strategy<G> for ManualTui<G, V> {
    fn choose(
        &mut self,
        state: &G::State,
        legal: &[G::Action],
        _rng: &mut dyn RngCore,
    ) -> G::Action {
        loop {
            if let Ok(Event::Key(key)) = event::read() {
                if let Some(action) = V::handle_key(state, legal, key) {
                    return action;
                }
            }
        }
    }
}

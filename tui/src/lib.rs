//! ratatui で動く TUI runner。
//!
//! ゲームごとに [`TuiView`] を実装すると、`simulator::run` の Strategy / Observer に
//! [`ManualTui`] と [`TuiObserver`] として組み込める。`TuiView` 自身が UI 状態
//! (カーソル位置・入力モード等) を持てる。

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
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
/// 実装側にカーソル位置などの UI 状態を持てる。`render` は状態変化時に、
/// `handle_key` はキー押下時に呼ばれる。
pub trait TuiView<G: Game> {
    /// 状態を描画する。
    fn render(&self, state: &G::State, legal: &[G::Action], frame: &mut Frame, area: Rect);

    /// キーをアクションに翻訳する。確定しないなら `None`。
    fn handle_key(
        &mut self,
        state: &G::State,
        legal: &[G::Action],
        key: KeyEvent,
    ) -> Option<G::Action>;
}

type SharedTerminal = Rc<RefCell<Terminal<CrosstermBackend<io::Stdout>>>>;

/// 端末のライフサイクルを RAII で管理する。
///
/// 構築時に raw mode + alternate screen に入り、`Drop` で抜ける。
pub struct TuiApp<G: Game, V: TuiView<G>> {
    view: Rc<RefCell<V>>,
    terminal: SharedTerminal,
    _marker: PhantomData<fn() -> G>,
}

impl<G: Game, V: TuiView<G>> TuiApp<G, V> {
    pub fn new(view: V) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            view: Rc::new(RefCell::new(view)),
            terminal: Rc::new(RefCell::new(terminal)),
            _marker: PhantomData,
        })
    }

    /// `view` を共有する人間入力 Strategy を生成する。
    pub fn manual(&self) -> ManualTui<G, V> {
        ManualTui {
            view: self.view.clone(),
            terminal: self.terminal.clone(),
            _marker: PhantomData,
        }
    }

    /// 1ゲームを最後まで進めて結果を返す。
    pub fn run(
        &mut self,
        p1: &mut dyn Strategy<G>,
        p2: &mut dyn Strategy<G>,
    ) -> Outcome {
        let mut observer = TuiObserver {
            view: self.view.clone(),
            terminal: self.terminal.clone(),
            _marker: PhantomData,
        };
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

fn draw<G, V>(view: &Rc<RefCell<V>>, terminal: &SharedTerminal, state: &G::State)
where
    G: Game,
    V: TuiView<G>,
{
    let legal = G::legal_actions(state);
    let view = view.borrow();
    let _ = terminal.borrow_mut().draw(|frame| {
        let area = frame.area();
        view.render(state, &legal, frame, area);
    });
}

/// 状態変化のたびに [`TuiView::render`] を呼ぶ Observer。
pub struct TuiObserver<G: Game, V: TuiView<G>> {
    view: Rc<RefCell<V>>,
    terminal: SharedTerminal,
    _marker: PhantomData<fn() -> G>,
}

impl<G: Game, V: TuiView<G>> Observer<G> for TuiObserver<G, V> {
    fn on_start(&mut self, state: &G::State) {
        draw::<G, V>(&self.view, &self.terminal, state);
    }
    fn on_action(&mut self, _before: &G::State, _action: &G::Action, after: &G::State) {
        draw::<G, V>(&self.view, &self.terminal, after);
    }
    fn on_end(&mut self, state: &G::State, _outcome: &Outcome) {
        draw::<G, V>(&self.view, &self.terminal, state);
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

/// キー入力ごとに再描画し、[`TuiView::handle_key`] がアクションを返すまで待つ Strategy。
pub struct ManualTui<G: Game, V: TuiView<G>> {
    view: Rc<RefCell<V>>,
    terminal: SharedTerminal,
    _marker: PhantomData<fn() -> G>,
}

impl<G: Game, V: TuiView<G>> Strategy<G> for ManualTui<G, V> {
    fn choose(
        &mut self,
        state: &G::State,
        legal: &[G::Action],
        _rng: &mut dyn RngCore,
    ) -> G::Action {
        loop {
            draw::<G, V>(&self.view, &self.terminal, state);
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                let mut view = self.view.borrow_mut();
                if let Some(action) = view.handle_key(state, legal, key) {
                    return action;
                }
            }
        }
    }
}

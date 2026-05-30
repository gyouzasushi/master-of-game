//! Nim を TUI で遊ぶバイナリ。
//!
//! 「山を選ぶ → 取る個数を選ぶ」の2段階。矢印キーで操作、Enter で次へ、Esc で戻る。
//! 終局後は任意キーで終了。

use crossterm::event::{KeyCode, KeyEvent};
use master_of_game_core::game::{Game, Outcome, Player};
use master_of_game_core::games::nim::{Nim, NimAction, NimState};
use master_of_game_core::strategy::RandomStrategy;
use master_of_game_tui::{TuiApp, TuiView};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use std::io;

enum Mode {
    ChoosingHeap,
    ChoosingCount { heap: usize, count: u32 },
}

struct NimTuiView {
    heap_cursor: usize,
    mode: Mode,
}

impl NimTuiView {
    fn new() -> Self {
        Self {
            heap_cursor: 0,
            mode: Mode::ChoosingHeap,
        }
    }

    /// `heap_cursor` を、空でない山にスナップして返す。
    fn valid_heap(&self, state: &NimState) -> usize {
        let heaps = &state.heaps;
        if self.heap_cursor < heaps.len() && heaps[self.heap_cursor] > 0 {
            self.heap_cursor
        } else {
            heaps.iter().position(|&n| n > 0).unwrap_or(0)
        }
    }
}

fn outcome_label(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Win(Player::P1) => "P1 (あなた) の勝ち!".to_string(),
        Outcome::Win(Player::P2) => "P2 (ランダムAI) の勝ち".to_string(),
        Outcome::Draw => "引き分け".to_string(),
    }
}

impl TuiView<Nim> for NimTuiView {
    fn render(&self, state: &NimState, _legal: &[NimAction], frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(4),
            ])
            .split(area);

        let over = Nim::outcome(state);

        // ヘッダ
        let header_text = match &over {
            Some(o) => format!("ゲーム終了 — {}", outcome_label(o)),
            None => format!("Nim — {:?} の手番", state.next),
        };
        let header = Paragraph::new(header_text)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, chunks[0]);

        // 盤面
        let cur_heap = match &self.mode {
            Mode::ChoosingHeap => self.valid_heap(state),
            Mode::ChoosingCount { heap, .. } => *heap,
        };
        let preview_count = if over.is_some() {
            None
        } else {
            match &self.mode {
                Mode::ChoosingCount { count, .. } => Some(*count as usize),
                _ => None,
            }
        };

        let lines: Vec<Line> = state
            .heaps
            .iter()
            .enumerate()
            .map(|(i, &n)| {
                let highlight = over.is_none() && i == cur_heap;
                let marker = if highlight { "▶ " } else { "  " };
                let mut spans = vec![
                    Span::styled(
                        marker,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!("heap {}: ", i)),
                ];
                if let (Some(c), true) = (preview_count, highlight) {
                    let take = c.min(n as usize);
                    let remain = (n as usize).saturating_sub(take);
                    spans.push(Span::styled(
                        "●".repeat(take),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::styled(
                        "●".repeat(remain),
                        Style::default().fg(Color::DarkGray),
                    ));
                } else {
                    spans.push(Span::styled(
                        "●".repeat(n as usize),
                        Style::default().fg(Color::Red),
                    ));
                }
                spans.push(Span::raw(format!(" ({})", n)));
                Line::from(spans)
            })
            .collect();
        let board = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Board"));
        frame.render_widget(board, chunks[1]);

        // 入力ガイド
        let status = if over.is_some() {
            Paragraph::new("任意キーで終了")
        } else {
            match &self.mode {
                Mode::ChoosingHeap => Paragraph::new(vec![Line::from(
                    "↑↓ で山を選択 / Enter で個数選択へ",
                )]),
                Mode::ChoosingCount { heap, count } => Paragraph::new(vec![
                    Line::from(format!("heap {} から {} 個取る", heap, count)),
                    Line::from("↑↓ で個数 / Enter で確定 / Esc で戻る"),
                ]),
            }
        };
        frame.render_widget(
            status.block(Block::default().borders(Borders::ALL)),
            chunks[2],
        );
    }

    fn handle_key(
        &mut self,
        state: &NimState,
        _legal: &[NimAction],
        key: KeyEvent,
    ) -> Option<NimAction> {
        let nonempty: Vec<usize> = state
            .heaps
            .iter()
            .enumerate()
            .filter_map(|(i, &n)| if n > 0 { Some(i) } else { None })
            .collect();
        if nonempty.is_empty() {
            return None;
        }

        match &self.mode {
            Mode::ChoosingHeap => {
                let cur = self.valid_heap(state);
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.heap_cursor = nonempty
                            .iter()
                            .rev()
                            .find(|&&i| i < cur)
                            .copied()
                            .unwrap_or(cur);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.heap_cursor = nonempty
                            .iter()
                            .find(|&&i| i > cur)
                            .copied()
                            .unwrap_or(cur);
                    }
                    KeyCode::Enter => {
                        self.mode = Mode::ChoosingCount {
                            heap: cur,
                            count: 1,
                        };
                    }
                    _ => {}
                }
                None
            }
            Mode::ChoosingCount { heap, count } => {
                let h = *heap;
                let c = *count;
                let max = state.heaps[h];
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.mode = Mode::ChoosingCount {
                            heap: h,
                            count: (c + 1).min(max),
                        };
                        None
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.mode = Mode::ChoosingCount {
                            heap: h,
                            count: if c > 1 { c - 1 } else { 1 },
                        };
                        None
                    }
                    KeyCode::Enter => {
                        self.mode = Mode::ChoosingHeap;
                        Some(NimAction { heap: h, count: c })
                    }
                    KeyCode::Esc | KeyCode::Backspace => {
                        self.mode = Mode::ChoosingHeap;
                        None
                    }
                    _ => None,
                }
            }
        }
    }
}

fn main() -> io::Result<()> {
    let mut app = TuiApp::<Nim, NimTuiView>::new(NimTuiView::new())?;
    let mut p1 = app.manual();
    let mut p2 = RandomStrategy;
    app.run(&mut p1, &mut p2);
    Ok(())
}

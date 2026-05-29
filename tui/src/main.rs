//! Nim を TUI で対戦するバイナリ。
//!
//! 起動すると alternate screen に切り替わり、ratatui で盤面を描画する。
//! 数字キー (1-9) で合法手を選択。

use crossterm::event::{KeyCode, KeyEvent};
use master_of_game_core::games::nim::{Nim, NimAction, NimState};
use master_of_game_core::strategy::RandomStrategy;
use master_of_game_tui::{ManualTui, TuiApp, TuiView};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use std::io;

struct NimTuiView;

impl TuiView<Nim> for NimTuiView {
    fn render(state: &NimState, legal: &[NimAction], frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(legal.len().min(9) as u16 + 2),
            ])
            .split(area);

        let header = Paragraph::new(format!("Nim — {:?} の手番", state.next))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, chunks[0]);

        let lines: Vec<Line> = state
            .heaps
            .iter()
            .enumerate()
            .map(|(i, &n)| {
                Line::from(vec![
                    Span::raw(format!("heap {}: ", i)),
                    Span::styled("●".repeat(n as usize), Style::default().fg(Color::Red)),
                    Span::raw(format!(" ({})", n)),
                ])
            })
            .collect();
        let board = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Board"));
        frame.render_widget(board, chunks[1]);

        let action_lines: Vec<Line> = legal
            .iter()
            .enumerate()
            .take(9)
            .map(|(i, a)| Line::from(format!("[{}] heap {} から {} 個", i + 1, a.heap, a.count)))
            .collect();
        let actions = Paragraph::new(action_lines)
            .block(Block::default().borders(Borders::ALL).title("Actions (1-9)"));
        frame.render_widget(actions, chunks[2]);
    }

    fn handle_key(_state: &NimState, legal: &[NimAction], key: KeyEvent) -> Option<NimAction> {
        if let KeyCode::Char(c) = key.code {
            if let Some(d) = c.to_digit(10) {
                let i = d as usize;
                if (1..=legal.len()).contains(&i) {
                    return Some(legal[i - 1]);
                }
            }
        }
        None
    }
}

fn main() -> io::Result<()> {
    let outcome = {
        let mut app = TuiApp::<Nim, NimTuiView>::new()?;
        let mut p1 = ManualTui::<Nim, NimTuiView>::new();
        let mut p2 = RandomStrategy;
        app.run(&mut p1, &mut p2)
    };
    println!("Result: {:?}", outcome);
    Ok(())
}

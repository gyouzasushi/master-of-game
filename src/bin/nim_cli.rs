use master_of_game::game::Outcome;
use master_of_game::games::nim::{Nim, NimAction, NimState};
use master_of_game::observer::Observer;
use master_of_game::simulator::run;
use master_of_game::strategy::{ManualStrategy, Prompt, RandomStrategy};
use std::io::{self, BufRead, Write};

struct CuiObserver;

impl Observer<Nim> for CuiObserver {
    fn on_start(&mut self, s: &NimState) {
        println!("=== Nim 開始 ===");
        print_heaps(&s.heaps);
    }

    fn on_action(&mut self, before: &NimState, a: &NimAction, after: &NimState) {
        println!("{:?} は heap {} から {} 個取った", before.next, a.heap, a.count);
        print_heaps(&after.heaps);
    }

    fn on_end(&mut self, _s: &NimState, o: &Outcome) {
        match o {
            Outcome::Win(p) => println!("=== 終了: {:?} の勝ち ===", p),
            Outcome::Draw => println!("=== 終了: 引き分け ==="),
        }
    }
}

fn print_heaps(heaps: &[u32]) {
    for (i, &h) in heaps.iter().enumerate() {
        println!("  heap {}: {} ({})", i, "*".repeat(h as usize), h);
    }
}

struct StdinPrompt;

impl Prompt<Nim> for StdinPrompt {
    fn select(&mut self, _state: &NimState, legal: &[NimAction]) -> usize {
        println!("--- あなたの手番 ---");
        for (i, a) in legal.iter().enumerate() {
            println!("  [{}] heap {} から {} 個", i, a.heap, a.count);
        }
        let stdin = io::stdin();
        let mut stdin = stdin.lock();
        loop {
            print!("> ");
            io::stdout().flush().ok();
            let mut line = String::new();
            if stdin.read_line(&mut line).unwrap() == 0 {
                std::process::exit(0);
            }
            if let Ok(i) = line.trim().parse::<usize>() {
                if i < legal.len() {
                    return i;
                }
            }
            println!("無効な入力です。0..={} で指定してください", legal.len() - 1);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("manual");

    let mut obs = CuiObserver;
    let mut observers: Vec<&mut dyn Observer<Nim>> = vec![&mut obs];
    let mut rng = rand::thread_rng();

    match mode {
        "random" => {
            println!("(P1=ランダム / P2=ランダム)");
            let mut p1 = RandomStrategy;
            let mut p2 = RandomStrategy;
            run::<Nim>(&mut p1, &mut p2, &mut observers, &mut rng);
        }
        _ => {
            println!("(P1=あなた / P2=ランダム)");
            let mut p1: ManualStrategy<Nim, StdinPrompt> = ManualStrategy::new(StdinPrompt);
            let mut p2 = RandomStrategy;
            run::<Nim>(&mut p1, &mut p2, &mut observers, &mut rng);
        }
    }
}

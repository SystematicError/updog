mod display;
mod evaluate;
mod ordering;
mod pv;
mod search;
mod uci;

use crate::display::display_board;
use crate::search::Searcher;
use crate::uci::Uci;
use cozy_chess::util::display_uci_move;
use std::io::{BufRead, stdin};
use std::process::exit;

fn main() {
    let mut chess960 = false;
    let mut searcher = Searcher::new();

    for line in stdin().lock().lines() {
        if let Some(command) = Uci::parse(&line.expect("Should be able to read line"), chess960) {
            match command {
                Uci::Uci => {
                    println!("id name Updog");
                    println!("id author SystematicError");
                    println!("option name UCI_Chess960 type check default false");
                    println!("option name Threads type spin default 1 min 1 max 1");
                    println!("option name Hash type spin default 1 min 1 max 1");
                    println!("uciok");
                }

                Uci::IsReady => println!("readyok"),

                Uci::NewGame => todo!(),

                Uci::SetOption(name, value) => {
                    // TODO: Use try blocks instead of IIFE
                    (|| {
                        match name.as_str() {
                            "UCI_Chess960" => chess960 = value?.parse().ok()?,
                            "Threads" => todo!(),
                            "Hash" => todo!(),

                            _ => {}
                        }

                        Some(())
                    })();
                }

                Uci::Position(board, moves) => searcher.set_position(board, moves),

                Uci::Go(options) => searcher.best_move(options, |best| {
                    let mv = match best {
                        Some((board, mv)) => &display_uci_move(board, mv).to_string(),
                        None => "(none)",
                    };

                    println!("bestmove {mv}");
                }),

                Uci::Stop => todo!(),

                Uci::Quit => exit(0),

                Uci::D => display_board(searcher.board()),

                Uci::Bench(_depth) => todo!(),
            }
        }
    }
}

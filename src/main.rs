mod bench;
mod display;
mod engine;
mod evaluate;
mod ordering;
mod pv;
mod search;
mod time;
mod uci;

use crate::bench::bench;
use crate::display::display_board;
use crate::engine::Engine;
use crate::uci::Uci;
use cozy_chess::util::display_uci_move;
use std::io::{BufRead, stdin};
use std::process::exit;

fn main() {
    let mut chess960 = false;
    let mut engine = Engine::new();

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

                // TODO: Implement ucinewgame command
                Uci::NewGame => {}

                Uci::SetOption(name, value) => {
                    // TODO: Use try blocks instead of IIFE
                    (|| {
                        match name.as_str() {
                            "UCI_Chess960" => chess960 = value?.parse().ok()?,

                            "Threads" => {
                                // TODO: Implement threads option
                            }

                            "Hash" => {
                                // TODO: Implement hash option
                            }

                            _ => {}
                        }

                        Some(())
                    })();
                }

                Uci::Position(board, moves) => engine.set_position(board, moves),

                Uci::Go(time_options, search_options) => {
                    engine.best_move(time_options, search_options, |result| {
                        let mv = if let Some(mv) = result.best_move {
                            &display_uci_move(&result.board, mv).to_string()
                        } else {
                            "(none)"
                        };

                        println!("bestmove {mv}");
                    })
                }

                Uci::Stop => engine.stop(),

                Uci::Quit => exit(0),

                Uci::D => display_board(engine.board()),

                Uci::Bench => {
                    let (nodes, elapsed) = bench();
                    let nps = nodes as f64 / elapsed.as_secs_f64();

                    println!("{nodes} in {elapsed:#?} ({nps:.0} nps)");
                }
            }
        }
    }
}

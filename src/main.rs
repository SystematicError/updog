mod bench;
mod cli;
mod display;
mod engine;
mod evaluate;
mod ordering;
mod pv;
mod search;
mod time;
mod transposition;
mod uci;

use crate::bench::bench;
use crate::cli::{Cli, Command};
use crate::display::display_board;
use crate::engine::Engine;
use crate::evaluate::EvaluationUtils;
use crate::uci::Uci;
use clap::Parser;
use cozy_chess::util::display_uci_move;
use std::io::{BufRead, stdin};
use std::process::exit;

pub fn bench_and_display() {
    let (nodes, elapsed) = bench();
    let nps = nodes as f64 / elapsed.as_secs_f64();

    println!("{nodes} nodes in {elapsed:#?} ({nps:.0} nps)");
}

const MB: usize = 1024 * 1024;
const HASH_DEFAULT: usize = 16;
const HASH_MIN: usize = 1;
const HASH_MAX: usize = 64 * 1024;

fn uci_loop() {
    let mut chess960 = false;
    let mut engine =
        Engine::with_table_size(HASH_DEFAULT * MB).expect("Default hash size should be sufficient");

    for line in stdin().lock().lines() {
        if let Some(command) = Uci::parse(&line.expect("Should be able to read line"), chess960) {
            match command {
                Uci::Uci => {
                    println!("id name Updog");
                    println!("id author SystematicError");
                    println!(
                        "option name Hash type spin default {HASH_DEFAULT} min {HASH_MIN} max {HASH_MAX}"
                    );
                    println!("option name Clear Hash type button");
                    println!("option name Threads type spin default 1 min 1 max 1");
                    println!("option name UCI_Chess960 type check default false");
                    println!("uciok");
                }

                Uci::IsReady => println!("readyok"),

                Uci::NewGame => engine.new_game(),

                Uci::SetOption(name, None) => {
                    if name == "Clear Hash" {
                        engine.clear_table()
                    }
                }

                Uci::SetOption(name, Some(value)) => {
                    // TODO: Use try blocks instead of IIFE
                    (|| {
                        match name.as_str() {
                            "UCI_Chess960" => chess960 = value.parse().ok()?,

                            "Threads" => {
                                // TODO: Implement threads option
                            }

                            "Hash" => {
                                let size = value.parse().ok()?;

                                if !(HASH_MIN..=HASH_MAX).contains(&size) {
                                    return None;
                                }

                                engine.resize_table(size * MB).ok()?;
                            }

                            _ => {}
                        }

                        Some(())
                    })();
                }

                Uci::Position(board, moves) => engine.set_position(board, moves),

                Uci::Go(time_options, search_options) => {
                    engine.best_move(
                        time_options,
                        search_options,
                        |result| {
                            println!(
                                "info depth {} score {} nodes {} hashfull {} pv {}",
                                result.depth,
                                result.score.display(),
                                result.info.nodes,
                                result.hashfull,
                                result.pv_line.display(result.board)
                            );
                        },
                        |result| {
                            let mv = if let Some(mv) = result.best_move {
                                &display_uci_move(result.board, mv).to_string()
                            } else {
                                "(none)"
                            };

                            println!("bestmove {mv}");
                        },
                    );
                }

                Uci::Stop => engine.stop(),

                Uci::Quit => exit(0),

                Uci::D => display_board(engine.board()),

                Uci::Bench => bench_and_display(),
            }
        }
    }
}

fn main() {
    match Cli::parse().command {
        Some(Command::Bench { depth: _, short: _ }) => bench_and_display(),
        None => uci_loop(),
    }
}

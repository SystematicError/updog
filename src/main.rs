mod uci;

use crate::uci::Uci;
use std::io::{BufRead, stdin};

fn main() {
    let chess960 = false;

    for line in stdin().lock().lines() {
        if let Some(command) = Uci::parse(&line.unwrap(), chess960) {}
    }
}

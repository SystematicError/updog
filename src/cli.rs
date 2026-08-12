use crate::bench::DEPTH;
use crate::search::Ply;
use clap::Parser;

#[derive(Parser)]
#[clap(about, version)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Parser)]
pub enum Command {
    /// Calculate the node count fingerprint for the search algorithm
    Bench {
        /// Depth to search each position to
        #[arg(default_value_t = DEPTH)]
        depth: Ply,

        /// Print only the node count
        #[arg(short, long)]
        short: bool,
    },
}

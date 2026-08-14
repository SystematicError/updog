use crate::bench::DEPTH_DEFAULT;
use crate::search::Ply;
use clap::Parser;

// TODO: Add about

#[derive(Parser)]
#[clap(version)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Parser)]
pub enum Command {
    /// Calculate the node count fingerprint for the search algorithm
    Bench {
        /// Depth to search each position to
        #[arg(default_value_t = DEPTH_DEFAULT)]
        depth: Ply,

        /// Print only the node count
        #[arg(short, long)]
        short: bool,
    },
}

use crate::bench::DEPTH_DEFAULT;
use crate::search::Ply;
use cozy_chess::util::parse_uci_move;
use cozy_chess::{Board, Move};
use std::fmt;
use std::str::SplitWhitespace;
use std::time::Duration;

#[allow(clippy::enum_variant_names)]
pub enum Uci {
    Uci,
    IsReady,
    NewGame,
    SetOption(String, Option<String>),
    Position(Board, Vec<Move>),
    Go(TimeOptions, SearchOptions),
    Stop,
    Quit,

    // Non standard commands
    D,
    Bench(Ply),
}

pub enum TimeOptions {
    Clock {
        wtime: Duration,
        btime: Duration,
        winc: Duration,
        binc: Duration,
    },
    MoveTime(Duration),
    Infinite,
}

pub struct SearchOptions {
    pub depth: Option<Ply>,
    pub nodes: Option<usize>,
}

pub enum UciParseError {
    ExpectedToken,
    UnexpectedToken,
    InvalidFen,
    IllegalMove,
    InvalidDuration,
    InvalidDepth,
    InvalidNodes,
}

impl fmt::Display for UciParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "UCI parse error: {}",
            match self {
                Self::ExpectedToken => "Didn't receive an expected token",
                Self::UnexpectedToken => "Received an unexpected token",
                Self::InvalidFen => "Invalid FEN position",
                Self::IllegalMove => "Illegal move",
                Self::InvalidDuration => "Invalid value for a time duration",
                Self::InvalidDepth => "Invalid value for depth",
                Self::InvalidNodes => "Invalid value for nodes",
            }
        )?;

        Ok(())
    }
}

fn parse_duration(tokens: &mut SplitWhitespace<'_>) -> Result<Duration, UciParseError> {
    Ok(Duration::from_millis(
        tokens
            .next()
            .ok_or(UciParseError::ExpectedToken)?
            .parse()
            .map_err(|_| UciParseError::InvalidDuration)?,
    ))
}

impl Uci {
    pub fn parse(input: &str, chess960: bool) -> Result<Option<Self>, UciParseError> {
        let mut tokens = input.split_whitespace();

        // Return `None` if the input is empty or just whitespace
        let command = match tokens.next() {
            Some(command) => command,
            None => return Ok(None),
        };

        let parsed = match command {
            "uci" => Self::Uci,
            "isready" => Self::IsReady,
            "ucinewgame" => Self::NewGame,

            "setoption" => {
                if tokens.next().ok_or(UciParseError::ExpectedToken)? != "name" {
                    return Err(UciParseError::UnexpectedToken);
                }

                let name: Vec<_> = tokens.by_ref().take_while(|&t| t != "value").collect();

                if name.is_empty() {
                    return Err(UciParseError::ExpectedToken);
                }

                let name = name.join(" ");

                let value: Vec<_> = tokens.by_ref().collect();

                let value = if value.is_empty() {
                    None
                } else {
                    Some(value.join(" "))
                };

                Self::SetOption(name, value)
            }

            "position" => {
                let board = match tokens.next().ok_or(UciParseError::ExpectedToken)? {
                    "startpos" => Board::default(),

                    "fen" => {
                        let fen: Vec<_> = tokens.by_ref().take(6).collect();

                        if fen.len() != 6 {
                            return Err(UciParseError::InvalidFen);
                        }

                        Board::from_fen(&fen.join(" "), chess960)
                            .map_err(|_| UciParseError::InvalidFen)?
                    }

                    _ => return Err(UciParseError::UnexpectedToken),
                };

                if let Some(token) = tokens.next()
                    && token != "moves"
                {
                    return Err(UciParseError::UnexpectedToken);
                }

                let mut moves = Vec::new();
                let mut new_board = board.clone();

                for mv in tokens.by_ref() {
                    let mv = parse_uci_move(&new_board, mv)
                        .map_err(|_| UciParseError::UnexpectedToken)?;

                    new_board
                        .try_play(mv)
                        .map_err(|_| UciParseError::IllegalMove)?;

                    moves.push(mv);
                }

                Self::Position(board, moves)
            }

            "go" => {
                let mut wtime = Duration::ZERO;
                let mut btime = Duration::ZERO;
                let mut winc = Duration::ZERO;
                let mut binc = Duration::ZERO;

                let mut movetime = Duration::ZERO;
                let mut infinite = false;

                let mut search_options = SearchOptions {
                    depth: Some(Ply::MAX),
                    nodes: None,
                };

                while let Some(token) = tokens.next() {
                    match token {
                        "wtime" => wtime = parse_duration(&mut tokens)?,
                        "btime" => btime = parse_duration(&mut tokens)?,
                        "winc" => winc = parse_duration(&mut tokens)?,
                        "binc" => binc = parse_duration(&mut tokens)?,

                        "movetime" => movetime = parse_duration(tokens.by_ref())?,
                        "infinite" => infinite = true,

                        "depth" => {
                            search_options.depth = Some(
                                tokens
                                    .next()
                                    .ok_or(UciParseError::ExpectedToken)?
                                    .parse()
                                    .map_err(|_| UciParseError::InvalidDepth)?,
                            );
                        }

                        "nodes" => {
                            search_options.nodes = Some(
                                tokens
                                    .next()
                                    .ok_or(UciParseError::ExpectedToken)?
                                    .parse()
                                    .map_err(|_| UciParseError::InvalidNodes)?,
                            )
                        }

                        _ => return Err(UciParseError::UnexpectedToken),
                    }
                }

                // Get rid of default depth limit for infinite searches
                if infinite {
                    search_options.depth = None;
                }

                let time_options = if infinite {
                    TimeOptions::Infinite
                } else if movetime != Duration::ZERO {
                    TimeOptions::MoveTime(movetime)
                } else if wtime != Duration::ZERO || btime != Duration::ZERO {
                    TimeOptions::Clock {
                        wtime,
                        btime,
                        winc,
                        binc,
                    }
                } else {
                    TimeOptions::Infinite
                };

                Self::Go(time_options, search_options)
            }

            "stop" => Self::Stop,
            "quit" => Self::Quit,

            "d" => Self::D,
            "bench" => {
                let depth = match tokens.next() {
                    Some(depth) => depth.parse().map_err(|_| UciParseError::InvalidDepth)?,
                    None => DEPTH_DEFAULT,
                };

                Self::Bench(depth)
            }

            _ => return Err(UciParseError::UnexpectedToken),
        };

        // Ensure all tokens have been consumed
        if tokens.next().is_some() {
            return Err(UciParseError::UnexpectedToken);
        }

        Ok(Some(parsed))
    }
}

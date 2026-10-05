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
    UnexpectedToken(String),
    InvalidFen(String),
    IllegalMove(Move),
    InvalidDuration(String),
    InvalidDepth(String),
    InvalidNodes(String),
}

impl fmt::Display for UciParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "UCI parse error: {}",
            match self {
                Self::ExpectedToken => String::from("Didn't receive an expected token"),
                Self::UnexpectedToken(token) => format!("Received an unexpected token: {token}"),
                Self::InvalidFen(fen) => format!("Invalid FEN position: {fen}"),
                Self::IllegalMove(mv) => format!("Illegal move: {mv}"),
                Self::InvalidDuration(token) =>
                    format!("Invalid value for a time duration: {token}"),
                Self::InvalidDepth(token) => format!("Invalid value for depth: {token}"),
                Self::InvalidNodes(token) => format!("Invalid value for nodes: {token}"),
            }
        )?;

        Ok(())
    }
}

fn parse_duration(tokens: &mut SplitWhitespace<'_>) -> Result<Duration, UciParseError> {
    let token = tokens.next().ok_or(UciParseError::ExpectedToken)?;

    Ok(Duration::from_millis(token.parse().map_err(|_| {
        UciParseError::InvalidDuration(token.to_owned())
    })?))
}

impl Uci {
    pub fn parse(input: &str, chess960: bool) -> Result<Option<Self>, UciParseError> {
        let mut tokens = input.split_whitespace();

        // Return `None` if the input is empty or just whitespace
        let command = match tokens.next() {
            Some(command) => command,
            None => return Ok(None),
        };

        let parsed =
            match command {
                "uci" => Self::Uci,
                "isready" => Self::IsReady,
                "ucinewgame" => Self::NewGame,

                "setoption" => {
                    if let token = tokens.next().ok_or(UciParseError::ExpectedToken)?
                        && token != "name"
                    {
                        return Err(UciParseError::UnexpectedToken(token.to_owned()));
                    }

                    let name: Vec<_> = tokens
                        .by_ref()
                        .take_while(|&token| token != "value")
                        .collect();

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
                                return Err(UciParseError::ExpectedToken);
                            }

                            let fen = fen.join(" ");

                            Board::from_fen(&fen, chess960)
                                .map_err(|_| UciParseError::InvalidFen(fen))?
                        }

                        token => return Err(UciParseError::UnexpectedToken(token.to_owned())),
                    };

                    if let Some(token) = tokens.next()
                        && token != "moves"
                    {
                        return Err(UciParseError::UnexpectedToken(token.to_owned()));
                    }

                    let mut moves = Vec::new();
                    let mut new_board = board.clone();

                    for mv in tokens.by_ref() {
                        let mv = parse_uci_move(&new_board, mv)
                            .map_err(|_| UciParseError::UnexpectedToken(mv.to_owned()))?;

                        new_board
                            .try_play(mv)
                            .map_err(|_| UciParseError::IllegalMove(mv))?;

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
                                let token = tokens.next().ok_or(UciParseError::ExpectedToken)?;

                                search_options.depth =
                                    Some(token.parse().map_err(|_| {
                                        UciParseError::InvalidDepth(token.to_owned())
                                    })?);
                            }

                            "nodes" => {
                                let token = tokens.next().ok_or(UciParseError::ExpectedToken)?;

                                search_options.nodes =
                                    Some(token.parse().map_err(|_| {
                                        UciParseError::InvalidNodes(token.to_owned())
                                    })?)
                            }

                            token => return Err(UciParseError::UnexpectedToken(token.to_owned())),
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
                        Some(depth) => depth
                            .parse()
                            .map_err(|_| UciParseError::InvalidDepth(depth.to_owned()))?,
                        None => DEPTH_DEFAULT,
                    };

                    Self::Bench(depth)
                }

                token => return Err(UciParseError::UnexpectedToken(token.to_owned())),
            };

        // Ensure all tokens have been consumed
        if let Some(token) = tokens.next() {
            return Err(UciParseError::UnexpectedToken(token.to_owned()));
        }

        Ok(Some(parsed))
    }
}

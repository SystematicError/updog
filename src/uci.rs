use crate::position::Position;
use cozy_chess::Board;
use cozy_chess::util::parse_uci_move;

pub enum Uci {
    Uci,
    IsReady,
    SetOption(String, Option<String>),
    Position(Position),
    Go,
    Quit,
}

impl Uci {
    pub fn parse(input: &str) -> Option<Self> {
        let mut tokens = input.split_whitespace();

        let command = tokens.next()?;

        let parsed = match command {
            "uci" => Uci::Uci,
            "isready" => Uci::IsReady,

            "setoption" => {
                if tokens.next()? != "name" {
                    return None;
                };

                let name: Vec<_> = tokens.by_ref().take_while(|&t| t != "value").collect();

                if name.is_empty() {
                    return None;
                }

                let name = name.join(" ");

                let value: Vec<_> = tokens.by_ref().collect();

                let value = if value.is_empty() {
                    None
                } else {
                    Some(value.join(" "))
                };

                Uci::SetOption(name, value)
            }

            "position" => {
                let mut position = Position::new(match tokens.next()? {
                    "startpos" => Board::default(),

                    "fen" => {
                        let fen: Vec<_> = tokens.by_ref().take(6).collect();

                        if fen.len() != 6 {
                            return None;
                        }

                        Board::from_fen(&fen.join(" "), false).ok()?
                    }

                    _ => return None,
                });

                if let Some(token) = tokens.next()
                    && token != "moves"
                {
                    return None;
                }

                // Apply all moves, or until a malformed or illegal move is encountered
                tokens
                    .by_ref()
                    .map_while(|mv| {
                        let mv = parse_uci_move(position.board(), mv).ok()?;
                        position.try_play(mv).ok()?;
                        Some(())
                    })
                    .count();

                Uci::Position(position)
            }

            "go" => {
                // TODO: Parse options passed to go
                let _ = tokens.by_ref().collect::<String>();

                Uci::Go
            }

            "quit" => Uci::Quit,

            _ => return None,
        };

        if tokens.next().is_some() {
            return None;
        }

        Some(parsed)
    }
}

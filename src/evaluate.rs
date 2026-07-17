use cozy_chess::Board;
use std::fmt;

pub type Evaluation = i32;

pub trait EvaluationUtils {
    const INFINITY: Self;

    fn display(self) -> impl fmt::Display;
}

impl EvaluationUtils for Evaluation {
    const INFINITY: Self = Self::MAX;

    fn display(self) -> impl fmt::Display {
        EvaluationDisplay(self)
    }
}

struct EvaluationDisplay(Evaluation);

impl fmt::Display for EvaluationDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "cp {}", self.0)?;
        Ok(())
    }
}

// TODO: Implement evaluation
pub fn evaluate(_board: &Board) -> Evaluation {
    0
}

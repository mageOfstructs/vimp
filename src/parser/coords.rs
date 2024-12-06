use leptos::logging;

use super::get_cursor_pos;
use super::push_num;
use std::fmt::{Debug, Display, Formatter};

mod rel_coords;
use rel_coords::FinishedRelCoord;
pub use rel_coords::{RelCoord, RelCoordPair};

#[derive(Debug, Clone)]
pub enum Coords {
    AbsCoord(u32, u32),
    RelCoord(FinishedRelCoord),
}

impl From<CoordFSM> for Coords {
    fn from(value: CoordFSM) -> Self {
        match value {
            CoordFSM::Abs(abs) => abs.get_coords(),
            CoordFSM::Rel(rc) => Coords::from(rc),
        }
    }
}

impl Coords {
    pub fn from_cursor() -> Self {
        let (x, y) = get_cursor_pos();
        Self::AbsCoord(x, y)
    }
    pub fn resolve(&self) -> (u32, u32) {
        match self {
            Coords::AbsCoord(x, y) => (*x, *y),
            Coords::RelCoord(fcp) => fcp.resolve_fcp(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AbsCoord {
    EnteringFirstNum(u32),
    EnteringSecondNum(u32, u32),
}

impl AbsCoord {
    fn get_coords(&self) -> Coords {
        match self {
            Self::EnteringFirstNum(num) => Coords::AbsCoord(*num, 0),
            Self::EnteringSecondNum(num, num2) => Coords::AbsCoord(*num, *num2),
        }
    }
}

impl Display for AbsCoord {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "a{}",
            match self {
                Self::EnteringFirstNum(num) => num.to_string(),
                Self::EnteringSecondNum(num, num2) => {
                    let mut ret = num.to_string();
                    ret.push(';');
                    ret.push_str(&num2.to_string());
                    ret
                }
            }
        )
    }
}

impl AbsCoord {
    fn advance(self, next_char: char) -> Result<Coords, Self> {
        match self {
            Self::EnteringFirstNum(num) => match next_char {
                '0'..='9' => Err(Self::EnteringFirstNum(push_num(num, next_char))),
                ';' => Err(Self::EnteringSecondNum(num, 0)),
                _ => {
                    logging::error!("Not part of AbsCoord Syntax (first num): {next_char}");
                    Err(self)
                }
            },
            Self::EnteringSecondNum(num1, num) => match next_char {
                '0'..='9' => Err(Self::EnteringSecondNum(num1, push_num(num, next_char))),
                ';' => Ok(Coords::AbsCoord(num1, num)),
                _ => {
                    logging::error!("Not part of AbsCoord Syntax (second num): {next_char}");
                    Err(self)
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum CoordFSM {
    Abs(AbsCoord),
    Rel(RelCoord),
}

impl Display for CoordFSM {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Self::Abs(c) => c.to_string(),
                Self::Rel(c) => c.to_string(),
            }
        )
    }
}

impl CoordFSM {
    pub fn advance(self, next_char: char) -> Result<Coords, Self> {
        match self {
            Self::Abs(absc) => match absc.advance(next_char) {
                Ok(coords) => Ok(coords),
                Err(next_state) => Err(Self::Abs(next_state)),
            },
            Self::Rel(relc) => match relc.advance(next_char) {
                Ok(coords) => Ok(Coords::RelCoord(coords)),
                Err(next_state) => Err(Self::Rel(next_state)),
            },
        }
    }
}

impl From<Coords> for CoordFSM {
    fn from(value: Coords) -> Self {
        match value {
            Coords::AbsCoord(x, y) => Self::Abs(AbsCoord::EnteringSecondNum(x, y)),
            Coords::RelCoord(frc) => {
                let (x, y) = frc.resolve_fcp();
                Self::Abs(AbsCoord::EnteringSecondNum(x, y))
            }
        }
    }
}

impl From<RelCoord> for Coords {
    fn from(value: RelCoord) -> Self {
        let ret = match value {
            RelCoord::FirstNumAndDirection(rcp) => FinishedRelCoord::OneCoord(rcp),
            RelCoord::EnteringFirstNum(_) => {
                FinishedRelCoord::OneCoord(RelCoordPair(0, Direction::Up))
            }
            RelCoord::EnteringSecondNum(rcp, _) => FinishedRelCoord::OneCoord(rcp),
            RelCoord::BothNums(rcp1, rcp2) => FinishedRelCoord::TwoCoords(rcp1, rcp2),
            RelCoord::EnteringDistance(dir, dist) => {
                logging::log!("{dir:?}: {dist}");
                let (x, y) = get_cursor_pos();
                return Coords::AbsCoord(
                    (x as i32 + dir.horiz.resolve(dist)) as u32,
                    (y as i32 + dir.vert.resolve(dist)) as u32, // this is horrible
                );
            }
            RelCoord::Direction(dir) => {
                let (x, y) = get_cursor_pos();
                return Coords::AbsCoord(
                    (x as i32 + dir.horiz.resolve(5)) as u32,
                    (y as i32 + dir.vert.resolve(5)) as u32, // this is horrible
                );
            }
        };

        Coords::RelCoord(ret)
    }
}

#[derive(Debug, Clone)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

const LEFT: char = 'h';
const DOWN: char = 'j';
const UP: char = 'k';
const RIGHT: char = 'l';

impl From<char> for Direction {
    fn from(value: char) -> Self {
        match value {
            LEFT => Direction::Left,
            DOWN => Direction::Down,
            UP => Direction::Up,
            RIGHT => Direction::Right,
            _ => {
                logging::error!("Not a Direction '{}'!", value);
                panic!("Not a Direction '{}'!", value)
            }
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Self::Left => LEFT,
                Self::Right => RIGHT,
                Self::Down => DOWN,
                Self::Up => UP,
            }
        )
    }
}

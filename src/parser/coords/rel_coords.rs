use super::{DOWN, LEFT, RIGHT, UP};
use crate::parser::{get_cursor_pos, push_num, short_distance, AutoHide, Direction, FastDirection};
use leptos::logging;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone)]
pub enum FinishedRelCoord {
    OneCoord(RelCoordPair),
    TwoCoords(RelCoordPair, RelCoordPair),
}

impl FinishedRelCoord {
    pub fn resolve_with_offset(&self, off: (u32, u32)) -> (u32, u32) {
        let (x, y) = off;
        match self {
            Self::OneCoord(rcp) => rcp.get_coords(x, y),
            Self::TwoCoords(rcp, rcp2) => {
                let (x, y) = rcp.get_coords(x, y);
                rcp2.get_coords(x, y)
            }
        }
    }

    /// needs CursorSetter to be in context
    pub fn resolve_fcp(&self) -> (u32, u32) {
        self.resolve_with_offset(get_cursor_pos())
    }
}

impl RelCoord {
    pub fn advance(self, next_char: char) -> Result<FinishedRelCoord, Self> {
        match self {
            Self::EnteringFirstNum(num) => match next_char {
                '0'..='9' => Err(Self::EnteringFirstNum(push_num(num, next_char))),
                LEFT | DOWN | UP | RIGHT => Err(Self::FirstNumAndDirection(RelCoordPair(
                    num,
                    next_char.into(),
                ))),
                _ if short_distance(next_char).is_ok() => Err(Self::EnteringDistance(
                    FastDirection::try_from('a').unwrap(),
                    short_distance(next_char).unwrap(),
                )),
                _ => {
                    logging::error!("Not part of RelCoord Syntax (first num): {next_char}");
                    Err(self)
                }
            },
            Self::FirstNumAndDirection(ref rcp) => match next_char {
                '\n' => Ok(FinishedRelCoord::OneCoord(rcp.clone())),
                ';' => Err(Self::EnteringSecondNum(rcp.clone(), 0)),
                '0'..='9' => Err(Self::EnteringSecondNum(
                    rcp.clone(),
                    next_char.to_digit(10).unwrap(),
                )),
                _ => {
                    logging::error!("Not part of RelCoord Syntax (second num): {next_char}");
                    Err(self)
                }
            },
            Self::EnteringSecondNum(ref rcp, num) => match next_char {
                '0'..='9' => Err(Self::EnteringSecondNum(
                    rcp.clone(),
                    push_num(num, next_char),
                )),
                LEFT | DOWN | UP | RIGHT => Ok(FinishedRelCoord::TwoCoords(
                    rcp.clone(),
                    RelCoordPair(num, next_char.into()),
                )),
                _ => {
                    logging::error!(
                        "Not part of RelCoord Syntax (entering second num): {next_char}"
                    );
                    Err(self)
                }
            },
            Self::BothNums(ref rcp1, ref rcp2) => match next_char {
                '\n' | ';' => Ok(FinishedRelCoord::TwoCoords(rcp1.clone(), rcp2.clone())),
                _ => {
                    logging::error!("Not part of RelCoord Syntax (both nums): {next_char}");
                    Err(self)
                }
            },
            Self::Direction(ref dir) => match short_distance(next_char) {
                Ok(dist) => Err(Self::EnteringDistance(dir.clone(), dist)),
                Err(_) => {
                    logging::error!("Not part of short distance syntax: {next_char}");
                    Err(self)
                }
            },
            Self::EnteringDistance(ref dir, cur_dist) => match short_distance(next_char) {
                Ok(dist) => Err(Self::EnteringDistance(dir.clone(), cur_dist + dist)),
                Err(_) => {
                    logging::error!("Not part of short distance syntax: {next_char}");
                    Err(self)
                }
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct RelCoordPair(pub u32, pub Direction);

impl RelCoordPair {
    pub fn get_coords(&self, x: u32, y: u32) -> (u32, u32) {
        // FIXME: this panics if one tries to move a form even partially out-of-bounds, the
        // solution to this should be to truncate the form to fit, however this requires knowing
        // both points, which cannot be known by this + this is a widely used API, so changing it
        // will be painful
        match self.1 {
            Direction::Up => (x, y.checked_sub(self.0).unwrap_or(0)), // hotfix to prevent panics
            Direction::Down => (x, y + self.0),
            Direction::Left => (x.checked_sub(self.0).unwrap_or(0), y), // hotfix to prevent panics
            Direction::Right => (x + self.0, y),
        }
    }
}

impl Display for RelCoordPair {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}{}", self.0, self.1)
    }
}

#[derive(Debug, Clone)]
pub enum RelCoord {
    EnteringFirstNum(u32),
    FirstNumAndDirection(RelCoordPair),
    EnteringSecondNum(RelCoordPair, u32),
    BothNums(RelCoordPair, RelCoordPair),
    // second route
    Direction(FastDirection),
    EnteringDistance(FastDirection, u32),
}

impl Display for RelCoord {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Self::EnteringFirstNum(num) => num.to_string_autohide(),
                Self::FirstNumAndDirection(rcp) => rcp.to_string(),
                Self::EnteringSecondNum(rcp, num) => {
                    let mut ret = rcp.to_string();
                    ret.push(';');
                    ret.push_str(&num.to_string_autohide());
                    ret
                }
                Self::BothNums(rcp, rcp2) => {
                    let mut ret = rcp.to_string();
                    ret.push(';');
                    ret.push_str(&rcp2.to_string());
                    ret
                }
                Self::Direction(dir) =>
                    <FastDirection as Into<char>>::into(dir.clone()).to_string(),
                Self::EnteringDistance(dir, dist) => {
                    format!(
                        "{}{}",
                        <FastDirection as Into<char>>::into(dir.clone()),
                        dist
                    )
                }
            }
        )
    }
}

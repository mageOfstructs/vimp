use std::fmt::{Debug, Display, Formatter};

use leptos::logging;

use crate::components::get_cursor_pos;

#[derive(Debug, Clone)]
pub enum CommandType {
    Move,
    Line,
    Rectangle,
    Text,
    Circle(u32),
}

impl Display for CommandType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                CommandType::Move => "",
                CommandType::Line => "l",
                CommandType::Rectangle => "r",
                CommandType::Text => "t",
                CommandType::Circle(_) => "c",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct CommandFSM {
    coords: Option<Result<Coords, CoordFSM>>,
    ctype: CommandType,
    color: Option<String>,
}

// impl Display for CommandFSM {
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
//         write!(
//             f,
//             "{}{}",
//             self.ctype,
//             match self.coords {
//                 None => "".to_string(),
//                 Some(ref coords) => coords.to_string(),
//             }
//         )
//     }
// }

// impl IntoView for CommandFSM {
//     fn into_view(self) -> leptos::View {
//         #[allow(unused_braces)]
//         view! {
//             {self.to_string()}
//         }
//         .into_view()
//     }
// }

#[derive(Debug, Clone)]
pub struct Command {
    coords: Coords,
    ctype: CommandType,
    color: Option<String>,
}

impl From<CommandFSM> for Command {
    fn from(value: CommandFSM) -> Self {
        let coords: Coords = match value.coords {
            None => Coords::from_cursor(),
            Some(Ok(coords)) => coords,
            Some(Err(fsm)) => Coords::from(fsm),
        };
        Self {
            coords,
            ctype: value.ctype,
            color: value.color,
        }
    }
}

impl Command {
    pub fn ctype(&self) -> CommandType {
        self.ctype.clone()
    }
    pub fn coords(&self) -> Coords {
        self.coords.clone()
    }
    pub fn color(&self) -> Option<String> {
        self.color.clone()
    }
}

pub enum FSMResult {
    OkCommand(Command),
    OkFSM(CommandFSM),
    Err(char),
}

impl CommandFSM {
    pub fn from(str: String) -> FSMResult {
        if str.is_empty() {
            return FSMResult::Err('\0');
        }
        let mut it = str.chars();
        let mut ret = match Self::new(it.next().unwrap()) {
            Ok(fsm) => fsm,
            Err(c) => return FSMResult::Err(c),
        };
        for char in it {
            match ret.advance(char) {
                Ok(com) => return FSMResult::OkCommand(com),
                Err(new_state) => ret = new_state,
            }
        }
        FSMResult::OkFSM(ret)
    }

    pub fn new(next_char: char) -> Result<Self, char> {
        let mut coords = None;
        let ctype = match next_char {
            'l' => CommandType::Line,
            'r' => CommandType::Rectangle,
            't' => CommandType::Text,
            'c' => CommandType::Circle(0),
            'a' => {
                coords = Some(Err(CoordFSM::Abs(AbsCoord::EnteringFirstNum(0))));
                CommandType::Move
            }
            '0'..='9' => {
                coords = Some(Err(CoordFSM::Rel(RelCoord::EnteringFirstNum(
                    next_char.to_digit(10).unwrap(),
                ))));
                CommandType::Move
            }
            _ => {
                logging::error!("Not valid command begin: {next_char}");
                return Err(next_char);
            }
        };
        Ok(Self {
            coords,
            ctype,
            color: None,
        })
    }

    pub fn advance(mut self, next_char: char) -> Result<Command, Self> {
        if next_char == '\n' {
            return Ok(Command::from(self));
        }
        if next_char == '@' {
            logging::log!("Reading into color buffer from now on");
            return Err(Self {
                color: Some(String::with_capacity(5)),
                ..self
            });
        }
        if let Some(ref mut str) = self.color
            && let Some(Ok(_)) = self.coords
        {
            logging::log!("Got part of color: {next_char}");
            return match next_char {
                '\n' | ';' => Ok(Command::from(self)),
                _ => {
                    str.push(next_char);
                    Err(Self {
                        color: Some(str.to_string()),
                        ..self
                    })
                }
            };
        } else {
            match self.coords {
                None => match next_char {
                    '0'..='9' => Err(Self {
                        coords: Some(Err(CoordFSM::Rel(RelCoord::EnteringFirstNum(
                            next_char.to_digit(10).unwrap(),
                        )))),
                        ..self
                    }),
                    'a' => Err(Self {
                        coords: Some(Err(CoordFSM::Abs(AbsCoord::EnteringFirstNum(0)))),
                        ..self
                    }),
                    _ if FastDirection::try_from(next_char).is_ok() => Err(Self {
                        coords: Some(Err(CoordFSM::Rel(RelCoord::Direction(
                            FastDirection::try_from(next_char).unwrap(),
                        )))),
                        ..self
                    }),
                    _ => {
                        logging::error!("Not valid coord begin: {next_char}");
                        Err(self)
                    }
                },
                Some(ref fsm) => match fsm {
                    Ok(coords) => match self.ctype {
                        CommandType::Circle(num) => match next_char {
                            '0'..='9' => Err(Self {
                                ctype: CommandType::Circle(push_num(num, next_char)),
                                ..self
                            }),
                            ';' => Ok(Command {
                                coords: coords.clone(),
                                ctype: self.ctype,
                                color: None,
                            }),
                            c => {
                                logging::error!("Not part of Circle Radius Syntax: {c}");
                                Err(self)
                            }
                        },
                        _ => unreachable!(),
                    },
                    Err(fsm) => match fsm.clone().advance(next_char) {
                        Ok(coords) => match self.ctype {
                            _ => Err(Self {
                                coords: Some(Ok(coords)),
                                ..self
                            }),
                        },
                        Err(next_state) => Err(Self {
                            coords: Some(Err(next_state)),
                            ..self
                        }),
                    },
                },
            }
        }
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
enum RelCoord {
    EnteringFirstNum(u32),
    FirstNumAndDirection(RelCoordPair),
    EnteringSecondNum(RelCoordPair, u32),
    BothNums(RelCoordPair, RelCoordPair),
    // second route
    Direction(FastDirection),
    EnteringDistance(FastDirection, u32),
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

#[derive(Clone, Debug)]
enum FastDirectionType {
    Pos,
    Neg,
    None,
}

impl FastDirectionType {
    fn resolve(&self, dist: u32) -> i32 {
        match self {
            Self::Pos => dist as i32,
            Self::Neg => -(dist as i32),
            Self::None => 0,
        }
    }
}
#[derive(Clone, Debug)]
struct FastDirection {
    horiz: FastDirectionType, // Pos = Left, Neg = Right
    vert: FastDirectionType,  // Pos = Down, Neg = Up
}
impl TryFrom<char> for FastDirection {
    type Error = String;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        let mut horiz = FastDirectionType::None;
        let mut vert = FastDirectionType::None;
        match value {
            'q' => {
                horiz = FastDirectionType::Neg;
                vert = FastDirectionType::Neg;
            }
            'w' => vert = FastDirectionType::Neg,
            'e' => {
                horiz = FastDirectionType::Pos;
                vert = FastDirectionType::Neg;
            }
            'd' => horiz = FastDirectionType::Pos,
            'c' => {
                horiz = FastDirectionType::Pos;
                vert = FastDirectionType::Pos;
            }
            'x' => vert = FastDirectionType::Pos,
            'y' => {
                vert = FastDirectionType::Pos;
                horiz = FastDirectionType::Neg;
            }
            'a' => horiz = FastDirectionType::Neg,
            other => return Err(format!("Failed converting to FastDirection: {other}")),
        };
        Ok(Self { horiz, vert })
    }
}

impl Into<char> for FastDirection {
    fn into(self) -> char {
        match self.horiz {
            FastDirectionType::Pos => match self.vert {
                FastDirectionType::Pos => 'c',
                FastDirectionType::Neg => 'e',
                FastDirectionType::None => 'd',
            },
            FastDirectionType::Neg => match self.vert {
                FastDirectionType::Pos => 'y',
                FastDirectionType::Neg => 'q',
                FastDirectionType::None => 'a',
            },
            FastDirectionType::None => match self.vert {
                FastDirectionType::Pos => 'x',
                FastDirectionType::Neg => 'w',
                FastDirectionType::None => panic!("FastDirection goes nowhere!"),
            },
        }
    }
}

trait AutoHide {
    fn to_string_autohide(&self) -> String;
}
impl AutoHide for u32 {
    fn to_string_autohide(&self) -> String {
        if *self == 0 {
            return "".to_string();
        }
        self.to_string()
    }
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

fn push_num(num: u32, digit: char) -> u32 {
    num * 10 + digit.to_digit(10).unwrap()
}

impl RelCoord {
    fn advance(self, next_char: char) -> Result<FinishedRelCoord, Self> {
        match self {
            Self::EnteringFirstNum(num) => match next_char {
                '0'..='9' => Err(Self::EnteringFirstNum(push_num(num, next_char))),
                LEFT | DOWN | UP | RIGHT => Err(Self::FirstNumAndDirection(RelCoordPair(
                    num,
                    next_char.into(),
                ))),
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

const SHORT_5: char = 'q';
const SHORT_15: char = 'e';
const SHORT_25: char = 'r';
const SHORT_50: char = 't';
const SHORT_75: char = 'z';

fn short_distance(value: char) -> Result<u32, ()> {
    Ok(match value {
        SHORT_5 => 5,
        SHORT_15 => 15,
        SHORT_25 => 25,
        SHORT_50 => 50,
        SHORT_75 => 75,
        _ => return Err(()),
    })
}

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
enum AbsCoord {
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
enum CoordFSM {
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
    fn advance(self, next_char: char) -> Result<Coords, Self> {
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

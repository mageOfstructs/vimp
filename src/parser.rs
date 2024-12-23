use std::fmt::{Debug, Display, Formatter};

use leptos::logging;

use crate::{
    components::get_cursor_pos,
    graphics::{Circle, Form, Line, Rect, Text},
};

pub mod coords;
use coords::{AbsCoord, CoordFSM, RelCoord};
pub use coords::{Coords, Direction, RelCoordPair};

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
pub struct CreateComFSM {
    coords: Option<Result<Coords, CoordFSM>>,
    ctype: CommandType,
    color: Option<String>,
    mods: Modifiers,
}

#[derive(Debug, Clone)]
pub struct Command {
    start_coords: Option<Coords>,
    coords: Coords,
    ctype: CommandType,
    color: Option<String>,
    mods: Modifiers,
}

impl TryInto<Form> for CreateComFSM {
    type Error = CommandType;
    fn try_into(self) -> Result<Form, Self::Error> {
        let com = Command::from(self);
        let res = match com.ctype {
            CommandType::Line => Form::Line(Line::try_from(com).unwrap()),
            CommandType::Rectangle => Form::Rect(Rect::try_from(com).unwrap()),
            CommandType::Text => Form::Text(Text::try_from(com).unwrap()),
            CommandType::Circle(_) => Form::Circle(Circle::try_from(com).unwrap()),
            other => return Err(other),
        };
        Ok(res)
    }
}

impl From<CreateComFSM> for Command {
    fn from(value: CreateComFSM) -> Self {
        let mut value = value;
        let coords: Coords = match value.coords {
            None => Coords::from_cursor(),
            Some(Ok(coords)) => coords,
            Some(Err(fsm)) => {
                if let CommandType::Circle(rad) = value.ctype
                    && rad == 0
                    && let CoordFSM::Rel(RelCoord::EnteringFirstNum(real_rad)) = fsm
                {
                    value.ctype = CommandType::Circle(real_rad);
                }
                Coords::from(fsm)
            }
        };
        Self {
            start_coords: None,
            coords,
            ctype: value.ctype,
            color: value.color,
            mods: value.mods,
        }
    }
}

impl Command {
    pub fn new(
        ctype: CommandType,
        start_coords: Option<Coords>,
        coords: Coords,
        color: Option<String>,
        mods: Modifiers,
    ) -> Self {
        Self {
            start_coords,
            coords,
            ctype,
            color,
            mods,
        }
    }
    pub fn ctype(&self) -> CommandType {
        self.ctype.clone()
    }
    pub fn coords(&self) -> Coords {
        self.coords.clone()
    }
    pub fn color(&self) -> Option<String> {
        self.color.clone()
    }
    pub fn mods(&self) -> &Modifiers {
        &self.mods
    }
    pub fn start_coords(&self) -> (u32, u32) {
        match &self.start_coords {
            None => get_cursor_pos(),
            Some(c) => c.resolve(),
        }
    }
}

pub enum FSMResult {
    OkCommand(Command),
    OkFSM(CreateComFSM),
    Err(char),
}

pub enum ModifierType {
    MoveCursor,
    Collide,
    CursorIsMiddle,
}

#[derive(Clone, Debug)]
pub struct Modifiers(u8);

impl TryFrom<char> for ModifierType {
    type Error = char;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            'm' => ModifierType::CursorIsMiddle,
            'c' => ModifierType::Collide,
            'o' => ModifierType::MoveCursor,
            _ => {
                return Err(value);
            }
        })
    }
}

impl Modifiers {
    pub fn new() -> Self {
        Modifiers(0)
    }
    pub fn new_with_state(move_cursor: bool, collided: bool, cis_middle: bool) -> Self {
        let state = move_cursor as u8 | (collided as u8) << 1 | (cis_middle as u8) << 2;
        Modifiers(state)
    }
    fn set_internal(&mut self, i: u8, val: bool) {
        if val {
            self.0 |= 1 << i;
        } else {
            self.0 &= 255 ^ (1 << i);
        }
    }

    fn get_internal(&self, i: u8) -> bool {
        (self.0 >> i) & 1 == 1
    }

    pub fn move_cursor(&self) -> bool {
        self.get_internal(0)
    }
    pub fn collide(&self) -> bool {
        self.get_internal(1)
    }
    pub fn cursor_is_middle(&self) -> bool {
        self.get_internal(2)
    }

    fn set(&mut self, mod_type: ModifierType) {
        self.set_internal(mod_type as u8, true);
    }

    pub fn get(&self, mod_type: ModifierType) -> bool {
        self.get_internal(mod_type as u8)
    }
}

impl CreateComFSM {
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
            mods: Modifiers::new(),
        })
    }

    fn parse_mods(&mut self, next_char: char) -> FSMResult {
        match next_char {
            'm' => {
                self.mods.set(ModifierType::CursorIsMiddle);
            }
            'c' => {
                self.mods.set(ModifierType::Collide);
            }
            'o' => {
                self.mods.set(ModifierType::MoveCursor);
            }
            _ => {
                return FSMResult::Err(next_char);
            }
        }
        FSMResult::OkFSM(Self { ..self.clone() })
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
        if let Some(ref mut str) = self.color {
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
            // FIXME: hotfix until I do it better
            if let Some(Err(CoordFSM::Rel(RelCoord::Direction(_)))) = self.coords {
                if let FSMResult::OkFSM(fsm) = self.parse_mods(next_char) {
                    return Err(fsm);
                }
            }
            if let Some(Err(CoordFSM::Abs(AbsCoord::EnteringFirstNum(_)))) = self.coords {
                logging::log!("Parsing mods, even though this feels sketchy");
                if let FSMResult::OkFSM(fsm) = self.parse_mods(next_char) {
                    logging::log!("{:?}", self.mods);
                    // return Err(fsm);
                }
            }

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
                                start_coords: None,
                                coords: coords.clone(),
                                ctype: self.ctype,
                                color: None,
                                mods: self.mods,
                            }),
                            c => {
                                logging::error!("Not part of Circle Radius Syntax: {c}");
                                Err(self)
                            }
                        },
                        _ => {
                            // FIXME: will never get executed
                            let mut mods = self.mods;

                            match next_char {
                                'm' => {
                                    mods.set(ModifierType::CursorIsMiddle);
                                }
                                'c' => {
                                    mods.set(ModifierType::Collide);
                                }
                                'o' => {
                                    mods.set(ModifierType::MoveCursor);
                                }
                                _ => {
                                    logging::error!("Invalid modifier key: {next_char}");
                                }
                            }
                            Err(Self { mods, ..self })
                        }
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

fn push_num(num: u32, digit: char) -> u32 {
    num * 10 + digit.to_digit(10).unwrap()
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

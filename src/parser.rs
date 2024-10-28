enum CommandType {
    Move,
    Line,
    Rectangle,
}

struct CommandFSM {
    coords: Option<CoordFSM>,
    ctype: CommandType,
}

struct Command {
    coords: Coords,
    ctype: CommandType,
}

impl CommandFSM {
    fn new(next_char: char) -> Self {
        let mut coords = None;
        let ctype = match next_char {
            'l' => CommandType::Line,
            'r' => CommandType::Rectangle,
            'a' => {
                coords = Some(CoordFSM::Abs(AbsCoord::EnteringFirstNum(0)));
                CommandType::Move
            }
            '0'..'9' => {
                coords = Some(CoordFSM::Rel(RelCoord::EnteringFirstNum(0)));
                CommandType::Move
            }
            _ => panic!("Not valid command begin: {next_char}"),
        };
        Self { coords, ctype }
    }

    fn advance(self, next_char: char) -> Result<Command, Self> {
        match self.coords {
            None => match next_char {
                '0'..'9' => Err(Self {
                    coords: Some(CoordFSM::Rel(RelCoord::EnteringFirstNum(0))),
                    ctype: self.ctype,
                }),
                'a' => Err(Self {
                    coords: Some(CoordFSM::Abs(AbsCoord::EnteringFirstNum(0))),
                    ctype: self.ctype,
                }),
                _ => panic!("Not valid coord begin: {next_char}"),
            },
            Some(fsm) => match fsm.advance(next_char) {
                Ok(coords) => Ok(Command {
                    coords,
                    ctype: self.ctype,
                }),
                Err(next_state) => Err(Self {
                    coords: Some(next_state),
                    ctype: self.ctype,
                }),
            },
        }
    }
}

enum Direction {
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
            _ => panic!("Not a Direction '{}'!", value),
        }
    }
}

struct RelCoordPair(u32, Direction);

enum RelCoord {
    EnteringFirstNum(u32),
    FirstNumAndDirection(RelCoordPair),
    EnteringSecondNum(RelCoordPair, u32),
    BothNums(RelCoordPair, RelCoordPair),
}

enum FinishedRelCoord {
    OneCoord(RelCoordPair),
    TwoCoords(RelCoordPair, RelCoordPair),
}

fn push_num(num: u32, digit: char) -> u32 {
    num * 10 + digit.to_digit(10).unwrap() as u32
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
                _ => panic!("Not part of RelCoord Syntax (first num): {next_char}"),
            },
            Self::FirstNumAndDirection(rcp) => {
                match next_char {
                    '\n' => {
                        // TODO: do some drawing
                        Ok(FinishedRelCoord::OneCoord(rcp))
                    }
                    ';' => Err(Self::EnteringSecondNum(rcp, 0)),
                    _ => panic!("Not part of RelCoord Syntax (second num): {next_char}"),
                }
            }
            Self::EnteringSecondNum(rcp, num) => match next_char {
                '0'..='9' => Err(Self::EnteringSecondNum(rcp, push_num(num, next_char))),
                LEFT | DOWN | UP | RIGHT => {
                    Err(Self::BothNums(rcp, RelCoordPair(num, next_char.into())))
                }
                _ => panic!("Not part of RelCoord Syntax (entering second num): {next_char}"),
            },
            Self::BothNums(rcp1, rcp2) => match next_char {
                '\n' | ';' => Ok(FinishedRelCoord::TwoCoords(rcp1, rcp2)),
                _ => panic!("Not part of RelCoord Syntax (both nums): {next_char}"),
            },
        }
    }
}

enum Coords {
    AbsCoord(u32, u32),
    RelCoord(FinishedRelCoord),
}

enum AbsCoord {
    EnteringFirstNum(u32),
    EnteringSecondNum(u32, u32),
}

impl AbsCoord {
    fn advance(self, next_char: char) -> Result<Coords, Self> {
        match self {
            Self::EnteringFirstNum(num) => match next_char {
                '0'..'9' => Err(Self::EnteringFirstNum(push_num(num, next_char))),
                ';' => Err(Self::EnteringSecondNum(num, 0)),
                _ => panic!("Not part of AbsCoord Syntax (first num): {next_char}"),
            },
            Self::EnteringSecondNum(num1, num) => match next_char {
                '0'..'9' => Err(Self::EnteringFirstNum(push_num(num, next_char))),
                ';' => Ok(Coords::AbsCoord(num1, num)),
                _ => panic!("Not part of AbsCoord Syntax (second num): {next_char}"),
            },
        }
    }
}

enum CoordFSM {
    Abs(AbsCoord),
    Rel(RelCoord),
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

use itertools::Itertools;
use std::slice::Iter;

pub fn start_day(day: &str) {
    println!("Advent of Code 2024 - Day {:0>2}", day);
}

// Additional common functions

pub type Position = (usize, usize);
pub type Rectangle = (Position, Position);

pub type AbsolutePosition = (isize, isize);
pub type AbsoluteRectangle = (crate::AbsolutePosition, crate::AbsolutePosition);

pub fn rectangle_includes(rectangle: &Rectangle, pos: Position) -> bool {
    pos.0 >= rectangle.0 .0
        && pos.1 >= rectangle.0 .1
        && pos.0 <= rectangle.1 .0
        && pos.1 <= rectangle.1 .1
}

pub fn arectangle_includes(rectangle: &AbsoluteRectangle, pos: AbsolutePosition) -> bool {
    pos.0 >= rectangle.0 .0
        && pos.1 >= rectangle.0 .1
        && pos.0 <= rectangle.1 .0
        && pos.1 <= rectangle.1 .1
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Direction {
    N,
    S,
    W,
    E,
    NE,
    NW,
    SE,
    SW,
}

impl Direction {
    pub fn iter() -> Iter<'static, Direction> {
        use Direction::*;
        static DIRECTIONS: [Direction; 8] = [N, S, W, E, NE, NW, SE, SW];
        DIRECTIONS.iter()
    }

    pub fn from_symbol(c: char) -> Option<Direction> {
        match c {
            '^' => Some(Direction::N),
            'v' => Some(Direction::S),
            '<' => Some(Direction::W),
            '>' => Some(Direction::E),
            _ => None,
        }
    }

    pub fn turn_right(self) -> Direction {
        use Direction::*;
        match self {
            N => E,
            E => S,
            S => W,
            W => N,

            NW => NE,
            NE => SE,
            SE => SW,
            SW => NW,
        }
    }

    pub fn turn_45_deg(self) -> Direction {
        use Direction::*;
        match self {
            N => NE,
            S => SW,
            W => NW,
            E => SE,
            NE => E,
            NW => N,
            SE => S,
            SW => W,
        }
    }

    pub fn turn_left(self) -> Direction {
        use Direction::*;
        match self {
            N => W,
            W => S,
            S => E,
            E => N,

            NW => SW,
            SW => SE,
            SE => NE,
            NE => NW,
        }
    }
}

pub fn leap(pos: Position, dir: Direction, dist: usize) -> Option<Position> {
    use Direction::*;
    match dir {
        N if (pos.0 >= dist) => Some((pos.0 - dist, pos.1)),
        S => Some((pos.0 + dist, pos.1)),
        W if (pos.1 >= dist) => Some((pos.0, pos.1 - dist)),
        E => Some((pos.0, pos.1 + dist)),
        NE if pos.0 >= dist => Some((pos.0 - dist, pos.1 + dist)),
        NW if pos.0 >= dist && pos.1 >= dist => Some((pos.0 - dist, pos.1 - dist)),
        SE => Some((pos.0 + dist, pos.1 + dist)),
        SW if pos.1 >= dist => Some((pos.0 + dist, pos.1 - dist)),
        _ => None,
    }
}

pub fn aleap(pos: AbsolutePosition, dir: Direction, dist: isize) -> AbsolutePosition {
    use Direction::*;
    match dir {
        N => (pos.0 - dist, pos.1),
        S => (pos.0 + dist, pos.1),
        W => (pos.0, pos.1 - dist),
        E => (pos.0, pos.1 + dist),
        NE => (pos.0 - dist, pos.1 + dist),
        NW => (pos.0 - dist, pos.1 - dist),
        SE => (pos.0 + dist, pos.1 + dist),
        SW => (pos.0 + dist, pos.1 - dist),
    }
}

pub fn leap_in_bounds(
    pos: Position,
    dir: Direction,
    dist: usize,
    area: &Rectangle,
) -> Option<Position> {
    leap(pos, dir, dist).filter(|np| rectangle_includes(area, *np))
}

pub fn aleap_in_bounds(
    pos: AbsolutePosition,
    dir: Direction,
    dist: isize,
    area: &AbsoluteRectangle,
) -> Option<AbsolutePosition> {
    let np = aleap(pos, dir, dist);
    if arectangle_includes(area, np) {
        Some(np)
    } else {
        None
    }
}

pub fn beam(pos: Position, dir: Direction, len: usize, border: Position) -> Option<Vec<Position>> {
    match dir {
        Direction::N => {
            if pos.0 >= len - 1 {
                Some((0..len).map(|d| (pos.0 - d, pos.1)).collect_vec())
            } else {
                None
            }
        }
        Direction::S => {
            if pos.0 + len - 1 <= border.0 {
                Some((0..len).map(|d| (pos.0 + d, pos.1)).collect_vec())
            } else {
                None
            }
        }
        Direction::W => {
            if pos.1 >= len - 1 {
                Some((0..len).map(|d| (pos.0, pos.1 - d)).collect_vec())
            } else {
                None
            }
        }
        Direction::E => {
            if pos.1 + len - 1 <= border.1 {
                Some((0..len).map(|d| (pos.0, pos.1 + d)).collect_vec())
            } else {
                None
            }
        }
        Direction::NW => {
            if pos.0 >= len - 1 && pos.1 >= len - 1 {
                Some((0..len).map(|d| (pos.0 - d, pos.1 - d)).collect_vec())
            } else {
                None
            }
        }
        Direction::NE => {
            if pos.0 >= len - 1 && pos.1 + len - 1 <= border.1 {
                Some((0..len).map(|d| (pos.0 - d, pos.1 + d)).collect_vec())
            } else {
                None
            }
        }
        Direction::SW => {
            if pos.0 + len - 1 <= border.0 && pos.1 >= len - 1 {
                Some((0..len).map(|d| (pos.0 + d, pos.1 - d)).collect_vec())
            } else {
                None
            }
        }
        Direction::SE => {
            if pos.0 + len - 1 <= border.0 && pos.1 + len - 1 <= border.1 {
                Some((0..len).map(|d| (pos.0 + d, pos.1 + d)).collect_vec())
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        start_day("00");
    }
}

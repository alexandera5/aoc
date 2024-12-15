use adv_code_2024::*;
use anyhow::*;
use code_timing_macros::time_snippet;
use const_format::concatcp;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::result::Result::Ok;
use itertools::Itertools;


const DAY: &str = "15";
const INPUT_FILE: &str = concatcp!("input/", DAY, ".txt");

const TEST: &str = r#"##########
#..O..O.O#
#......O.#
#.OO..O.O#
#..O@..O.#
#O#..O...#
#O..O..O.#
#.OO.O.OO#
#....O...#
##########

<vv>^<v^>v>^vv^v>v<>v^v<v<^vv<<<^><<><>>v<vvv<>^v^>^<<<><<v<<<v^vv^v>^
vvv<<^>^v^^><<>>><>^<<><^vv^^<>vvv<>><^^v>^>vv<>v<<<<v<^v>^<^^>>>^<v<v
><>vv>v^v^<>><>>>><^^>vv>v<^^^>>v^v^<^^>v^^>v^<^v>v<>>v^v^<v>v^^<^^vv<
<<v<^>>^^^^>>>v^<>vvv^><v<<<>^^^vv^<vvv>^>v<^^^^v<>^>vvvv><>>v^<<^^^^^
^><^><>>><>^^<<^^v>>><^<v>^<vv>>v>>>^v><>^v><<<<v>>v<v<v>vvv>^<><<>^><
^>><>^v<><^vvv<^^<><v<<<<<><^v<<<><<<^^<v<^^^><^>>^<v^><<<^>>^v<v^v<v^
>^>>^v>vv>^<<^v<>><<><<v<<v><>v<^vv<<<>^^v^>^^>>><<^v>>v^v><^^>>^<>vv^
<><^^>^^^<><vvvvv^v<v<<>^v<v>v<<^><<><<><<<^^<<<^<<>><<><^^^>^^<>^>v<>
^^>vv<^v^v<vv>^<><v<^v>^^^>>>^^vvv^>vvv<>>>^<^>>>>>^<<^v>^vvv<>^<><<v>
v^^>>><<^^<>>^v^<v^vv<>v^<<>^<^v^v><^<<<><<^<v><v<>vv>>v><v^<vv<>v^<<^"#;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Tile {
    Wall,
    Box,
    WBox1,
    WBox2,
    Robot,
    Space,
}

impl Tile {
    fn from_char(c: char) -> Result<Tile> {
        match c {
            '#' => Ok(Tile::Wall),
            'O' => Ok(Tile::Box),
            '@' => Ok(Tile::Robot),
            '[' => Ok(Tile::WBox1),
            ']' => Ok(Tile::WBox2),
            '.' => Ok(Tile::Space),
            _ => Err(anyhow!("invalid tile: {}", c)),
        }
    }
}

#[derive(Debug)]
struct TileMap {
    tiles: Vec<Vec<Tile>>,
    area: Rectangle,
}

impl TileMap {
    fn get_tile(&self, pos: Position) -> Option<Tile> {
        self.tiles.get(pos.0).and_then(|row| row.get(pos.1).copied())
    }

    fn find_first(&self, target_tile: Tile) -> Option<Position> {
        for (i, row) in self.tiles.iter().enumerate() {
            for (j, tile) in row.iter().copied().enumerate() {
                if tile == target_tile {
                    return Some((i, j));
                }
            }
        }
        None
    }

    fn find_all(&self, target_tile: Tile) -> Vec<Position> {
        let mut positions = Vec::new();
        for (i, row) in self.tiles.iter().enumerate() {
            for (j, tile) in row.iter().copied().enumerate() {
                if tile == target_tile {
                    positions.push((i, j));
                }
            }
        }
        positions
    }

    fn swap_tiles(&mut self, src: Position, dst: Position) {
        let src_tile = self.tiles[src.0][src.1];
        let dst_tile = self.tiles[dst.0][dst.1];
        self.tiles[src.0][src.1] = dst_tile;
        self.tiles[dst.0][dst.1] = src_tile;
    }

    fn widen(&self) -> Self {
        /*
        If the tile is #, the new map contains ## instead.
        If the tile is O, the new map contains [] instead.
        If the tile is ., the new map contains .. instead.
        If the tile is @, the new map contains @. instead.
        */

        let mut new_tiles = Vec::with_capacity(self.tiles.len() * 2);
        for row in self.tiles.iter() {
            let mut new_row = Vec::with_capacity(row.len() * 2);
            for tile in row.iter().copied() {
                match tile {
                    Tile::Box => {
                        new_row.push(Tile::WBox1);
                        new_row.push(Tile::WBox2);
                    }
                    Tile::Robot => {
                        new_row.push(Tile::Robot);
                        new_row.push(Tile::Space);
                    }
                    Tile::WBox1 | Tile::WBox2 => {
                        panic!("Trying to widen a wall tile");
                    }
                    other => {
                        new_row.push(other);
                        new_row.push(other);
                    },
                }
            }
            new_tiles.push(new_row);
        }

        Self {
            tiles: new_tiles,
            area: (self.area.0, (self.area.1.0 * 2, self.area.1.1 * 2)),
        }
    }
}

fn read_input<R: BufRead>(input: R) -> Result<(TileMap, Vec<Direction>)> {
    let mut max_i = 0;
    let mut max_j = 0;
    let mut directions = None;

    let tiles = input
        .lines()
        .map_while(Result::ok)
        .enumerate()
        .filter_map(|(i, line)| {
            if line.is_empty() {
                directions = Some(vec![]);
                None
            } else if let Some(directions) = directions.as_mut() {
                directions.extend(line.chars().filter_map(Direction::from_symbol));
                None
            } else {
                max_i = max_i.max(i);
                max_j = max_j.max(line.len() - 1);
                Some(line.chars().flat_map(Tile::from_char).collect_vec())
            }
        })
        .collect::<Vec<_>>();

    if let Some(directions) = directions {
        let map = TileMap {
            tiles,
            area: ((0, 0), (max_i, max_j)),
        };
        Ok((map, directions))
    } else {
        Err(anyhow!("no directions found"))
    }
}

fn apply_moves(mut map: TileMap, directions: &[Direction]) -> TileMap {
    let mut robot = map.find_first(Tile::Robot).expect("robot exists");

    let mut moved_tiles = vec![];
    for dir in directions.iter().copied() {
        // Leap in direction
        // Case 0: Space, stop and move all tiles to this pos. @.
        // Case 1: Wall, stop: @#
        // Case 2: Box, add to moved: @O

        let mut target_pos = None;
        moved_tiles.clear();
        moved_tiles.push(robot);

        let mut moved_pos = robot;
        loop {
            let next_pos = leap_in_bounds(moved_pos, dir, 1, &map.area);
            match next_pos.and_then(|p| map.get_tile(p)) {
                Some(Tile::Box) | Some(Tile::WBox1) | Some(Tile::WBox2) => {
                    moved_tiles.push(next_pos.unwrap());
                    moved_pos = next_pos.unwrap();
                },
                Some(Tile::Space) => {
                    target_pos = next_pos;
                    break;
                }
                _ => {
                    moved_tiles.clear();
                    break;
                }
            }
        }

        if let Some(mut target_pos) = target_pos {
            for i in (0..moved_tiles.len()).rev() {
                map.swap_tiles(moved_tiles[i], target_pos);
                if i == 0 {
                    robot = target_pos;
                } else {
                    target_pos = moved_tiles[i];
                }
            }
        }
    }

    map
}

fn part1<R: BufRead>(reader: R) -> Result<usize> {
    let (map, directions) = read_input(reader)?;
    let robot_pos = map.find_first(Tile::Robot).expect("robot found");
    println!("Map: {:?}, robot: {:?}", map.area, robot_pos);

    let map = apply_moves(map, &directions);
    let answer = map.find_all(Tile::Box).into_iter().map(|(x, y)| x * 100 + y).sum();
    Ok(answer)
}

fn part2<R: BufRead>(reader: R) -> Result<usize> {
    let (map, directions) = read_input(reader)?;
    let map = map.widen();
    let map = apply_moves(map, &directions);
    let answer = map.find_all(Tile::WBox1).into_iter().map(|(x, y)| x * 100 + y).sum();
    Ok(answer)
}

fn main() -> Result<()> {
    start_day(DAY);

    //region Part 1
    println!("=== Part 1 ===");
    assert_eq!(10092, part1(BufReader::new(TEST.as_bytes()))?);

    let input_file = BufReader::new(File::open(INPUT_FILE)?);
    let result = time_snippet!(part1(input_file)?);
    println!("Result = {}", result);
    //endregion

    // region Part 2
    println!("\n=== Part 2 ===");
    assert_eq!(10092, part2(BufReader::new(TEST.as_bytes()))?);

    let input_file = BufReader::new(File::open(INPUT_FILE)?);
    let result = time_snippet!(part2(input_file)?);
    println!("Result = {}", result);
    // endregion

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST: &str = r#"########
#..O.O.#
##@.O..#
#...O..#
#.#.O..#
#...O..#
#......#
########

<^^>>>vv<v>>v<<"#;

    #[test]
    fn test_part1() {
        let answer = part1(TEST.as_bytes()).unwrap();
        assert_eq!(2028, answer);
    }

    #[test]
    fn test_part2() {
        let answer = part2(TEST.as_bytes()).unwrap();
        assert_eq!(9021, answer);
    }
}
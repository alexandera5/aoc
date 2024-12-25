use adv_code_2024::*;
use anyhow::*;
use code_timing_macros::time_snippet;
use const_format::concatcp;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::result::Result::Ok;

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

    fn as_char(&self) -> char {
        match self {
            Tile::Wall => '#',
            Tile::Box => 'O',
            Tile::WBox1 => '[',
            Tile::WBox2 => ']',
            Tile::Robot => '@',
            Tile::Space => '.',
        }
    }
}

#[derive(Debug)]
struct TileMap {
    tiles: Vec<Vec<Tile>>,
    area: AbsoluteRectangle,
}

impl Display for TileMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.tiles.iter() {
            for tile in row.iter() {
                write!(f, "{}", tile.as_char())?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl TileMap {
    fn new(size: (isize, isize), tiles: &[(Tile, AbsolutePosition)]) -> Self {
        let area = ((0, 0), size);
        let mut map = Self {
            area,
            tiles: Vec::with_capacity(area.1 .0 as usize),
        };
        for i in 0..area.1 .0 {
            if i == 0 || i == area.1 .0 - 1 {
                map.tiles.push(vec![Tile::Wall; area.1 .1 as usize]);
            } else {
                let mut row = Vec::with_capacity(area.1 .1 as usize);
                for j in 0..area.1 .1 {
                    if j == 0 || j == area.1 .1 - 1 {
                        row.push(Tile::Wall);
                    } else {
                        row.push(Tile::Space);
                    }
                }
                map.tiles.push(row);
            }
        }
        for (tile, pos) in tiles {
            map.tiles[pos.0 as usize][pos.1 as usize] = *tile;
        }
        map
    }

    fn get_tile(&self, pos: AbsolutePosition) -> Option<Tile> {
        self.tiles
            .get(pos.0 as usize)
            .and_then(|row| row.get(pos.1 as usize).copied())
    }

    fn find_first(&self, target_tile: Tile) -> Option<AbsolutePosition> {
        for (i, row) in self.tiles.iter().enumerate() {
            for (j, tile) in row.iter().copied().enumerate() {
                if tile == target_tile {
                    return Some((i as isize, j as isize));
                }
            }
        }
        None
    }

    fn find_all(&self, target_tiles: &[Tile]) -> Vec<AbsolutePosition> {
        let mut positions = Vec::new();
        for (i, row) in self.tiles.iter().enumerate() {
            for (j, tile) in row.iter().copied().enumerate() {
                if target_tiles.contains(&tile) {
                    positions.push((i as isize, j as isize));
                }
            }
        }
        positions
    }

    fn swap_tiles(&mut self, src: AbsolutePosition, dst: AbsolutePosition) {
        let src_tile = self.tiles[src.0 as usize][src.1 as usize];
        let dst_tile = self.tiles[dst.0 as usize][dst.1 as usize];
        self.tiles[src.0 as usize][src.1 as usize] = dst_tile;
        self.tiles[dst.0 as usize][dst.1 as usize] = src_tile;
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
                    }
                }
            }
            new_tiles.push(new_row);
        }

        Self {
            tiles: new_tiles,
            area: (self.area.0, (self.area.1 .0, self.area.1 .1 * 2)),
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
            area: ((0, 0), (max_i as isize, max_j as isize)),
        };
        Ok((map, directions))
    } else {
        Err(anyhow!("no directions found"))
    }
}

#[derive(Default)]
struct Lane {
    target: Option<AbsolutePosition>,
    tiles: Vec<AbsolutePosition>,
}

fn apply_moves(mut map: TileMap, directions: &[Direction]) -> TileMap {
    let mut robot = map.find_first(Tile::Robot).expect("robot exists");

    for dir in directions.iter().copied() {
        // Leap in direction
        // Case 0: Space, stop and move all tiles to this pos. @.
        // Case 1: Wall, stop: @#
        // Case 2: Box, add to moved: @O

        let mut lanes: BTreeMap<i8, Lane> = BTreeMap::new();
        lanes.entry(0).or_default().tiles.push(robot);

        let mut step_pos = robot;
        'step: while let Some(next_pos) = aleap_in_bounds(step_pos, dir, 1, &map.area) {
            let mut n_spaces = 0;
            for lane in lanes.keys().cloned().collect_vec() {
                let lane_pos = aleap(next_pos, dir.turn_right(), lane as isize);
                let lane_tile = map.get_tile(lane_pos);

                match lane_tile {
                    Some(Tile::Box) => {
                        lanes.entry(lane).or_default().tiles.push(lane_pos);
                    }
                    Some(Tile::WBox1) | Some(Tile::WBox2) => {
                        lanes.entry(lane).or_default().tiles.push(lane_pos);

                        let new_lane = match (lane_tile, dir) {
                            (Some(Tile::WBox1), Direction::N) => Some(lane + 1),
                            (Some(Tile::WBox2), Direction::N) => Some(lane - 1),
                            (Some(Tile::WBox1), Direction::S) => Some(lane - 1),
                            (Some(Tile::WBox2), Direction::S) => Some(lane + 1),
                            _ => None,
                        };
                        if let Some(new_lane) = new_lane {
                            let new_lane_pos = aleap(next_pos, dir.turn_right(), new_lane as isize);
                            lanes.entry(new_lane).or_default().tiles.push(new_lane_pos);
                        }
                    }
                    Some(Tile::Space) => {
                        n_spaces += 1;
                        lanes.entry(lane).or_default().tiles.push(lane_pos);
                    }
                    _ => {
                        lanes.clear();
                        break 'step;
                    }
                }
            }

            if n_spaces == lanes.len() {
                break;
            }
            step_pos = next_pos;
        }
        if lanes.is_empty() {
            continue;
        }

        for lane in lanes.values() {
            for i in (1..lane.tiles.len()).rev() {
                map.swap_tiles(lane.tiles[i], lane.tiles[i - 1]);
            }
        }
        robot = aleap(robot, dir, 1);
    }

    map
}

fn part1<R: BufRead>(reader: R) -> Result<usize> {
    let (map, directions) = read_input(reader)?;
    let robot_pos = map.find_first(Tile::Robot).expect("robot found");
    println!("Map: {:?}, robot: {:?}", map.area, robot_pos);

    let map = apply_moves(map, &directions);
    let answer = map
        .find_all(&[Tile::Box])
        .into_iter()
        .map(|(x, y)| x as usize * 100 + y as usize)
        .sum();
    Ok(answer)
}

fn part2<R: BufRead>(reader: R) -> Result<usize> {
    let (map, directions) = read_input(reader)?;
    let mut map = map.widen();
    println!("Initial:\n{}", map);
    for dir in directions {
        map = apply_moves(map, &[dir]);
        println!("Move: {:?}\n{}", dir, map);
    }
    let answer = map
        .find_all(&[Tile::WBox1])
        .into_iter()
        .map(|(x, y)| x as usize * 100 + y as usize)
        .sum();
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
    assert_eq!(1497888, result);
    //endregion

    // region Part 2
    println!("\n=== Part 2 ===");
    assert_eq!(9021, part2(BufReader::new(TEST.as_bytes()))?);

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
        let answer = part2(
            r#"#######
#...#.#
#.....#
#..OO@#
#..O..#
#.....#
#######

<vv<<^^<<^^"#
                .as_bytes(),
        )
        .unwrap();
        assert_eq!(618, answer);
    }

    #[test]
    fn test_apply_moves() {
        let map = TileMap::new(
            (7, 7),
            &vec![
                (Tile::Wall, (1, 4)),
                (Tile::Box, (3, 3)),
                (Tile::Box, (3, 4)),
                (Tile::Box, (4, 3)),
                (Tile::Robot, (3, 5)),
            ],
        );

        let wide_map = map.widen();
        assert_eq!(
            wide_map.find_all(&[Tile::WBox1]),
            vec![(3, 6), (3, 8), (4, 6)]
        );

        let wide_map = apply_moves(
            wide_map,
            &vec![
                Direction::W,
                Direction::S,
                Direction::S,
                Direction::W,
                Direction::W,
            ],
        );
        assert_eq!(
            wide_map.find_all(&[Tile::WBox1]),
            vec![(3, 5), (3, 7), (4, 6)]
        );
        assert_eq!(wide_map.find_first(Tile::Robot), Some((5, 7)));

        let move_2 = vec![Direction::N, Direction::N];
        let wide_map = apply_moves(wide_map, &move_2);
        assert_eq!(
            wide_map.find_all(&[Tile::WBox1]),
            vec![(2, 5), (2, 7), (3, 6)]
        );
        assert_eq!(wide_map.find_first(Tile::Robot), Some((4, 7)));
    }
}

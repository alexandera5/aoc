use adv_code_2024::*;
use anyhow::*;
use code_timing_macros::time_snippet;
use const_format::concatcp;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::result::Result::Ok;

use std::{thread, time};
use termion::{clear, cursor, event::Key, input::TermRead, raw::IntoRawMode};

const DAY: &str = "14";
const INPUT_FILE: &str = concatcp!("input/", DAY, ".txt");

const TEST: &str = r#"p=0,4 v=3,-3
p=6,3 v=-1,-3
p=10,3 v=-1,2
p=2,0 v=2,-1
p=0,0 v=1,3
p=3,0 v=-2,-2
p=7,6 v=-1,-3
p=3,0 v=-1,-2
p=9,3 v=2,3
p=7,3 v=-1,2
p=2,4 v=2,-3
p=9,5 v=-3,-3"#;

#[derive(Debug, Default, PartialEq, Copy, Clone)]
struct Robot {
    position: (isize, isize),
    velocity: (isize, isize),
}

impl Robot {
    fn simulate(self, steps: isize, area: (isize, isize)) -> Robot {
        let mut new_x = (self.position.0 + self.velocity.0 * steps) % area.0;
        if new_x < 0 {
            new_x += area.0;
        }
        let mut new_y = (self.position.1 + self.velocity.1 * steps) % area.1;
        if new_y < 0 {
            new_y += area.1;
        }

        Robot {
            position: (new_x, new_y),
            ..self
        }
    }

    fn quadrant(&self, area: (isize, isize)) -> Option<u8> {
        let mx = area.0 / 2;
        let my = area.1 / 2;
        if self.position.0 < mx && self.position.1 < my {
            Some(0)
        } else if self.position.0 > mx && self.position.1 < my {
            Some(1)
        } else if self.position.0 < mx && self.position.1 > my {
            Some(2)
        } else if self.position.0 > mx && self.position.1 > my {
            Some(3)
        } else {
            None
        }
    }
}

fn read_input<R: BufRead>(input: R) -> Result<(Vec<Robot>, (isize, isize))> {
    let robots = input
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| match line.split_once(" ") {
            Some((position, velocity)) => {
                let position = position[2..].split_once(",")?;
                let velocity = velocity[2..].split_once(",")?;
                let robot = Robot {
                    position: (
                        position.0.parse::<isize>().ok()?,
                        position.1.parse::<isize>().ok()?,
                    ),
                    velocity: (
                        velocity.0.parse::<isize>().ok()?,
                        velocity.1.parse::<isize>().ok()?,
                    ),
                };
                Some(robot)
            }
            _ => None,
        })
        .collect_vec();

    let max_x = robots
        .iter()
        .map(|robot| robot.position.0)
        .max()
        .unwrap_or_default();
    let max_y = robots
        .iter()
        .map(|robot| robot.position.1)
        .max()
        .unwrap_or_default();
    Ok((robots, (max_x + 1, max_y + 1)))
}

fn find_pattern(
    positions: impl Iterator<Item = (isize, isize)>,
    pattern: &[(isize, isize)],
) -> bool {
    let positions: HashSet<_> = positions.collect();
    for (x, y) in positions.iter().copied() {
        if pattern
            .iter()
            .all(|(dx, dy)| positions.contains(&(x + dx, y + dy)))
        {
            return true;
        }
    }

    false
}

fn inspect_manually(robots: Vec<Robot>, area: (isize, isize)) -> Result<()> {
    let mut seconds = 0;

    let mut stdout = std::io::stdout();
    let mut stdout = stdout.lock().into_raw_mode()?;
    let stdin = std::io::stdin();
    let mut keys = stdin.keys();
    let mut buffer = String::new();

    loop {
        write!(
            stdout,
            "{}{}",
            clear::All,         // Clear the screen
            cursor::Goto(1, 1)  // Move to top-left corner
        )?;

        let positions = robots
            .iter()
            .map(|r| r.simulate(seconds, area).position)
            .collect::<HashSet<_>>();

        for j in 0..=area.1 {
            buffer.clear();
            for i in 0..=area.0 {
                if positions.contains(&(i, j)) {
                    buffer.push('#');
                } else {
                    buffer.push('.');
                }
            }
            writeln!(stdout, "\r{}", buffer)?; // Use '\r' to ensure correct alignment
        }
        writeln!(
            stdout,
            "\n\rSeconds {}. Press Left/Right to navigate, 'q' to quit.",
            seconds
        )?;
        stdout.flush()?;

        if let Some(Ok(key)) = keys.next() {
            match key {
                Key::Right => {
                    seconds += 1;
                }
                Key::Left => {
                    seconds -= 1;
                }
                Key::PageDown => {
                    seconds += 100;
                }
                Key::PageUp => {
                    seconds -= 100;
                }
                Key::Char('q') => {
                    break;
                }
                _ => {}
            }
        }

        // Add a slight delay to avoid overwhelming the terminal
        thread::sleep(time::Duration::from_millis(10));
    }

    Ok(())
}

fn main() -> Result<()> {
    start_day(DAY);

    //region Part 1
    println!("=== Part 1 ===");

    fn part1<R: BufRead>(reader: R) -> Result<isize> {
        let (robots, area) = read_input(reader)?;
        println!("Read {} robots in area {:?}", robots.len(), area);

        let mut counter = HashMap::new();
        robots
            .into_iter()
            .filter_map(|robot| robot.simulate(100, area).quadrant(area))
            .for_each(|q| {
                counter.entry(q).and_modify(|c| *c += 1).or_insert(1);
            });

        println!("Robot counts by quadrant: {:?}", counter);
        let answer = counter
            .into_values()
            .reduce(|a, b| a * b)
            .unwrap_or_default();
        Ok(answer)
    }

    assert_eq!(12, part1(BufReader::new(TEST.as_bytes()))?);

    let input_file = BufReader::new(File::open(INPUT_FILE)?);
    let result = time_snippet!(part1(input_file)?);
    println!("Result = {}", result);
    //endregion

    // region Part 2
    println!("\n=== Part 2 ===");

    fn part2<R: BufRead>(reader: R) -> Result<usize> {
        let (mut robots, area) = read_input(reader)?;
        let mut seconds = 0;

        let pattern = vec![
            (0, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
            (-2, 2),
            (-1, 2),
            (0, 2),
            (1, 2),
            (2, 2),
            (-3, 3),
            (-2, 3),
            (-1, 3),
            (0, 3),
            (1, 3),
            (2, 3),
            (2, 3),
        ];

        loop {
            seconds += 1;
            robots.iter_mut().for_each(|r| *r = r.simulate(1, area));
            let found = find_pattern(robots.iter().map(|r| r.position), &pattern);
            if found {
                return Ok(seconds);
            }
        }
    }

    let input_file = BufReader::new(File::open(INPUT_FILE)?);
    let result = time_snippet!(part2(input_file)?);
    println!("Result = {}", result);
    // endregion

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{find_pattern, Robot};

    #[test]
    fn test_simulate_robot() {
        let robot = Robot {
            position: (2, 4),
            velocity: (2, -3),
        };
        let area = (11, 7);

        // p=(2,4), v=(2,-3); area: (11, 7)
        // 1: p=(4, 1)
        // 2: p=(6, -2) -> p(6, 5)
        // ...
        // 5: p=(12, -11) -> p=(1, 3)
        assert_eq!(robot.simulate(1, area).position, (4, 1));
        assert_eq!(robot.simulate(2, area).position, (6, 5));
        assert_eq!(robot.simulate(5, area).position, (1, 3));
    }

    #[test]
    fn test_quadrant() {
        let velocity = (0, 0);
        let area = (11, 7);

        assert_eq!(
            Robot {
                position: (0, 2),
                velocity
            }
            .quadrant(area),
            Some(0)
        );
        assert_eq!(
            Robot {
                position: (6, 0),
                velocity
            }
            .quadrant(area),
            Some(1)
        );
        assert_eq!(
            Robot {
                position: (3, 5),
                velocity
            }
            .quadrant(area),
            Some(2)
        );
        assert_eq!(
            Robot {
                position: (6, 6),
                velocity
            }
            .quadrant(area),
            Some(3)
        );
    }

    #[test]
    fn test_find_pattern() {
        let pattern = vec![
            (0, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
            // (-2, 2), (-1, 2), (0, 2), (1, 2), (2, 2),
        ];
        let found = find_pattern(
            vec![(45, 23), (44, 24), (45, 24), (46, 24)].into_iter(),
            &pattern,
        );
        assert_eq!(true, found);
    }
}

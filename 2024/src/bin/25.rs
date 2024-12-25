use adv_code_2024::*;
use anyhow::*;
use code_timing_macros::time_snippet;
use const_format::concatcp;
use itertools::{Either, Itertools};
use std::fs::File;
use std::io::Read;
use std::result::Result::Ok;

const DAY: &str = "25";
const INPUT_FILE: &str = concatcp!("input/", DAY, ".txt");

const TEST: &str = r#"#####
.####
.####
.####
.#.#.
.#...
.....

#####
##.##
.#.##
...##
...#.
...#.
.....

.....
#....
#....
#...#
#.#.#
#.###
#####

.....
.....
#.#..
###..
###.#
###.#
#####

.....
.....
.....
#....
#.#..
#.#.#
#####"#;

type Heights = Vec<u8>;

fn read_schematics(input: &str) -> Result<(Vec<Heights>, Vec<Heights>)> {
    let (locks, keys): (Vec<_>, Vec<_>) = input
        .split("\n\n")
        .filter_map(|scheme| {
            let mut lines = scheme.split('\n');
            let is_lock = lines.next()?.chars().all(|c| c == '#');
            let mut heights = if is_lock { vec![0u8; 5] } else { vec![5u8; 5] };

            lines.for_each(|line| {
                line.chars()
                    .enumerate()
                    .for_each(|(j, c)| match (is_lock, c) {
                        (true, '#') => heights[j] += 1,
                        (false, '.') => heights[j] -= 1,
                        _ => {}
                    });
            });

            Some((is_lock, heights))
        })
        .partition_map(|(is_lock, heights)| {
            if is_lock {
                Either::Left(heights)
            } else {
                Either::Right(heights)
            }
        });

    Ok((locks, keys))
}

fn main() -> Result<()> {
    start_day(DAY);

    //region Part 1
    println!("=== Part 1 ===");

    fn part1(input: &str) -> Result<usize> {
        let (locks, keys): (Vec<_>, Vec<_>) = read_schematics(input)?;

        let answer = keys
            .iter()
            .map(|key| {
                locks
                    .iter()
                    .filter(|lock| key.iter().zip(lock.iter()).all(|(x, y)| x + y <= 5))
                    .count()
            })
            .sum();
        Ok(answer)
    }

    assert_eq!(3, part1(TEST)?);

    let mut input_file = File::open(INPUT_FILE)?;
    let mut contents = String::new();
    input_file.read_to_string(&mut contents)?;
    let result = time_snippet!(part1(&contents)?);
    println!("Result = {}", result);
    //endregion

    Ok(())
}

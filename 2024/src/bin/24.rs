use adv_code_2024::*;
use anyhow::*;
use code_timing_macros::time_snippet;
use const_format::concatcp;
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::result::Result::Ok;

use std::{thread, time};
use termion::{clear, cursor, event::Key, input::TermRead, raw::IntoRawMode};

const DAY: &str = "24";
const INPUT_FILE: &str = concatcp!("input/", DAY, ".txt");

const TEST: &str = r#"x00: 1
x01: 0
x02: 1
x03: 1
x04: 0
y00: 1
y01: 1
y02: 1
y03: 1
y04: 1

ntg XOR fgs -> mjb
y02 OR x01 -> tnw
kwq OR kpj -> z05
x00 OR x03 -> fst
tgd XOR rvg -> z01
vdt OR tnw -> bfw
bfw AND frj -> z10
ffh OR nrd -> bqk
y00 AND y03 -> djm
y03 OR y00 -> psh
bqk OR frj -> z08
tnw OR fst -> frj
gnj AND tgd -> z11
bfw XOR mjb -> z00
x03 OR x00 -> vdt
gnj AND wpb -> z02
x04 AND y00 -> kjc
djm OR pbm -> qhw
nrd AND vdt -> hwm
kjc AND fst -> rvg
y04 OR y02 -> fgs
y01 AND x02 -> pbm
ntg OR kjc -> kwq
psh XOR fgs -> tgd
qhw XOR tgd -> z09
pbm OR djm -> kpj
x03 XOR y03 -> ffh
x00 XOR y04 -> ntg
bfw OR bqk -> z06
nrd XOR fgs -> wpb
frj XOR qhw -> z04
bqk OR frj -> z07
y03 OR x01 -> nrd
hwm AND bqk -> z03
tgd XOR rvg -> z12
tnw OR pbm -> gnj"#;


#[derive(Debug)]
enum Gate {
    And { in1: String, in2: String, out: String },
    Or { in1: String, in2: String, out: String },
    Xor { in1: String, in2: String, out: String },
}

impl Gate {
    fn output(&self) -> &String {
        match self {
            Gate::And { out, .. } => out,
            Gate::Or { out, .. } => out,
            Gate::Xor { out, .. } => out,
        }
    }

    fn in1(&self) -> &String {
        match self {
            Gate::And { in1, .. } => in1,
            Gate::Or { in1, .. }=> in1,
            Gate::Xor { in1, .. } => in1,
        }
    }

    fn in2(&self) -> &String {
        match self {
            Gate::And { in2, .. } => in2,
            Gate::Or { in2, .. } => in2,
            Gate::Xor { in2, .. } => in2,
        }
    }
}

fn parse_input<R: BufRead>(input: R) -> Result<(HashMap<String, u8>, Vec<Gate>)> {
    let mut wires = HashMap::new();
    let mut gates = Vec::new();

    let mut lines = input.lines();
    for line in lines.by_ref() {
        let line = line?;
        if line.trim().is_empty() {
            break;
        }

        let parts: Vec<&str> = line.split(':').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let wire = parts[0].to_string();
            let value: u8 = parts[1].parse()?;
            wires.insert(wire, value);
        }
    }

    for line in lines {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 5 {
            let in1 = parts[0].to_string();
            let gate_type = parts[1];
            let in2 = parts[2].to_string();
            let out = parts[4].to_string();

            let gate = match gate_type {
                "AND" => Gate::And { in1, in2, out },
                "OR" => Gate::Or { in1, in2, out },
                "XOR" => Gate::Xor { in1, in2, out },
                _ => panic!("Unknown gate type"),
            };

            gates.push(gate);
        }
    }

    Ok((wires, gates))
}

fn evaluate_gates(
    mut wires: HashMap<String, u8>,
    gates: &Vec<Gate>,
) -> HashMap<String, u8> {
    let mut undecided_gates: HashSet<_> = (0..gates.len()).collect();
    let mut decided_gates = Vec::new();

    while !undecided_gates.is_empty() {
        decided_gates.clear();
        decided_gates.extend(
            undecided_gates
                .iter()
                .map(|i| (i, &gates[*i]))
                .filter_map(|(i, gate)| {
                    let in1 = *wires.get(gate.in1())?;
                    let in2 = *wires.get(gate.in2())?;

                    let output = match gate {
                        Gate::And { .. } => in1 & in2,
                        Gate::Or { .. } => in1 | in2,
                        Gate::Xor { .. } => in1 ^ in2,
                    };
                    wires.insert(gate.output().clone(), output);
                    Some(*i)
                })
        );

        if decided_gates.is_empty() {
            break;
        } else {
            for gate in decided_gates.iter() {
                undecided_gates.remove(gate);
            }
        }
    }

    wires
}

fn main() -> Result<()> {
    start_day(DAY);

    //region Part 1
    println!("=== Part 1 ===");

    fn part1<R: BufRead>(reader: R) -> Result<usize> {
        let (wires, gates) = parse_input(reader)?;
        let answer = evaluate_gates(wires, &gates)
            .into_iter()
            .filter(|(name, _)| name.starts_with("z"))
            .collect::<BTreeMap<_, _>>()
            .values()
            .copied()
            .enumerate()
            .map(|(i, v)| usize::from(v) * 2_usize.pow(i as u32))
            .sum();
        Ok(answer)
    }

    assert_eq!(2024, part1(BufReader::new(TEST.as_bytes()))?);

    let input_file = BufReader::new(File::open(INPUT_FILE)?);
    let result = time_snippet!(part1(input_file)?);
    println!("Result = {}", result);
    //endregion

    // // region Part 2
    // println!("\n=== Part 2 ===");
    //
    // fn part2<R: BufRead>(reader: R) -> Result<usize> {
    //     let (mut robots, area) = read_input(reader)?;
    //     let mut seconds = 0;
    //
    //     let pattern = vec![
    //         (0, 0),
    //         (-1, 1),
    //         (0, 1),
    //         (1, 1),
    //         (-2, 2),
    //         (-1, 2),
    //         (0, 2),
    //         (1, 2),
    //         (2, 2),
    //         (-3, 3),
    //         (-2, 3),
    //         (-1, 3),
    //         (0, 3),
    //         (1, 3),
    //         (2, 3),
    //         (2, 3),
    //     ];
    //
    //     loop {
    //         seconds += 1;
    //         robots.iter_mut().for_each(|r| *r = r.simulate(1, area));
    //         let found = find_pattern(robots.iter().map(|r| r.position), &pattern);
    //         if found {
    //             return Ok(seconds);
    //         }
    //     }
    // }
    //
    // let input_file = BufReader::new(File::open(INPUT_FILE)?);
    // let result = time_snippet!(part2(input_file)?);
    // println!("Result = {}", result);
    // endregion

    Ok(())
}

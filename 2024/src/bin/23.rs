use adv_code_2024::*;
use anyhow::*;
use code_timing_macros::time_snippet;
use const_format::concatcp;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::result::Result::Ok;

const DAY: &str = "23";
const INPUT_FILE: &str = concatcp!("input/", DAY, ".txt");

const TEST: &str = r#"kh-tc
qp-kh
de-cg
ka-co
yn-aq
qp-ub
cg-tb
vc-aq
tb-ka
wh-tc
yn-cg
kh-ub
ta-co
de-co
tc-td
tb-wq
wh-td
ta-ka
td-qp
aq-cg
wq-ub
ub-vc
de-ta
wq-aq
wq-vc
wh-yn
ka-de
kh-ta
co-tc
wh-qp
tb-vc
td-yn"#;

fn bron_kerbosch(
    graph: &HashMap<usize, HashSet<usize>>,
    on_clique: &mut impl FnMut(&HashSet<usize>),
    r: &mut HashSet<usize>,
    p: &mut HashSet<usize>,
    x: &mut HashSet<usize>,
) {
    if p.is_empty() && x.is_empty() {
        on_clique(r);
        return;
    }

    while !p.is_empty() {
        let Some(v) = p.iter().next().cloned() else {
            continue;
        };

        let mut new_r = r.clone();
        new_r.insert(v);

        let mut new_p = HashSet::new();
        for pv in p.iter() {
            if graph.get(&v).is_some_and(|vs| vs.contains(pv)) {
                new_p.insert(*pv);
            }
        }

        let mut new_x = HashSet::new();
        for xv in x.iter() {
            if graph.get(&v).is_some_and(|vs| vs.contains(xv)) {
                new_x.insert(*xv);
            }
        }

        bron_kerbosch(graph, on_clique, &mut new_r, &mut new_p, &mut new_x);

        p.remove(&v);
        x.insert(v);
    }
}

fn main() -> Result<()> {
    start_day(DAY);

    //region Part 1
    println!("=== Part 1 ===");

    fn part1<R: BufRead>(reader: R) -> Result<usize> {
        let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
        reader.lines().map_while(Result::ok).for_each(|line| {
            if let Some((from, to)) = line.split_once('-') {
                graph
                    .entry(from.to_string())
                    .or_default()
                    .insert(to.to_string());
                graph
                    .entry(to.to_string())
                    .or_default()
                    .insert(from.to_string());
            }
        });

        let mut answer = 0;
        for (c1, c1_bros) in graph.iter() {
            for c2 in c1_bros {
                if c1 >= c2 {
                    continue;
                }
                let Some(c2_bros) = graph.get(c2) else {
                    continue;
                };
                for c3 in c2_bros {
                    if c2 >= c3 {
                        continue;
                    }
                    let Some(c3_bros) = graph.get(c3) else {
                        continue;
                    };
                    if c3_bros.contains(c1) {
                        let has_t =
                            c1.starts_with('t') || c2.starts_with('t') || c3.starts_with('t');
                        if has_t {
                            answer += 1;
                            // println!("{c1},{c2},{c3}");
                        }
                    }
                }
            }
        }

        Ok(answer)
    }

    assert_eq!(7, part1(BufReader::new(TEST.as_bytes()))?);

    let input_file = BufReader::new(File::open(INPUT_FILE)?);
    let result = time_snippet!(part1(input_file)?);
    println!("Result = {}", result);
    //endregion

    // region Part 2
    println!("\n=== Part 2 ===");

    fn part2<R: BufRead>(reader: R) -> Result<String> {
        let mut names: Vec<String> = Vec::new();
        let mut name_registry = HashMap::new();
        let mut graph: HashMap<usize, HashSet<usize>> = HashMap::new();

        reader.lines().map_while(Result::ok).for_each(|line| {
            if let Some((from, to)) = line.split_once('-') {
                let from = from.to_string();
                let from = if let Some(from) = name_registry.get(&from) {
                    *from
                } else {
                    let idx = names.len();
                    name_registry.insert(from.clone(), idx);
                    names.push(from);
                    idx
                };

                let to = to.to_string();
                let to = if let Some(to) = name_registry.get(&to) {
                    *to
                } else {
                    let idx = names.len();
                    name_registry.insert(to.clone(), idx);
                    names.push(to);
                    idx
                };

                graph.entry(from).or_default().insert(to);
                graph.entry(to).or_default().insert(from);
            }
        });

        let mut max_clique_size = 0;
        let mut max_clique_name = String::new();

        bron_kerbosch(
            &graph,
            &mut |clique: &HashSet<usize>| {
                if clique.len() > max_clique_size {
                    max_clique_size = clique.len();
                    max_clique_name = clique
                        .iter()
                        .filter_map(|idx| names.get(*idx))
                        .sorted()
                        .join(",");
                }
            },
            &mut HashSet::new(),
            &mut graph.keys().cloned().collect(),
            &mut HashSet::new(),
        );
        Ok(max_clique_name)
    }

    assert_eq!("co,de,ka,ta", part2(BufReader::new(TEST.as_bytes()))?);

    let input_file = BufReader::new(File::open(INPUT_FILE)?);
    let result = time_snippet!(part2(input_file)?);
    println!("Result = {}", result);
    // endregion

    Ok(())
}

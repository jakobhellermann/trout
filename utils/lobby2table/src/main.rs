use std::{
    collections::BTreeMap,
    fmt::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, Context, Result};

const PLACEHOLDER: &str = "60000";
const RESTART_PENALTY: u32 = 190;
const INCLUDE_BENCHES: bool = false;
const IGNORE_BENCH_TARGETS: &[&str] = &["E", "D", "G", "F"];

fn main() {
    let mut paths: Vec<_> = std::env::args().skip(1).map(PathBuf::from).collect();
    let mut in_cwd = false;
    if paths.is_empty() {
        paths.push(std::env::current_dir().unwrap());
        in_cwd = true;
    }
    if let Err(e) = run(&paths) {
        eprintln!("{e:?}");
    }

    if in_cwd {
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}
fn run(paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        if paths.len() > 0 {
            eprintln!("{}:", path.display());
        }
        let table = construct_table(path, INCLUDE_BENCHES)?;

        #[cfg(feature = "clipboard")]
        {
            let mut clipboard = arboard::Clipboard::new().context("failed to acquire clipboard")?;
            clipboard
                .set()
                .text(&table)
                .context("failed to set clipboard")?;

            eprintln!("table copied to clipboard");
        }

        println!("{}", table);
    }

    Ok(())
}

fn construct_table(path: &Path, include_benches: bool) -> Result<String> {
    let (n, mut connections, benches) = collect_entries(path)?;

    if include_benches {
        let bench_connections: Vec<_> = benches
            .iter()
            .filter_map(|bench| match bench {
                BenchNode::To { start, end, time } => Some((start, end, time)),
                _ => None,
            })
            .flat_map(|(start, end, time)| {
                let indirect_connections =
                    benches
                        .iter()
                        .filter_map(move |bench_node| match bench_node {
                            BenchNode::From {
                                start: other_start,
                                end: other_end,
                                time: other_time,
                            } => {
                                if IGNORE_BENCH_TARGETS.contains(&other_start.as_str()) {
                                    return None;
                                }

                                if other_start != end && *other_end != *start {
                                    Some((*start, end, other_start, *other_end, time + other_time))
                                } else {
                                    None
                                }
                            }
                            BenchNode::To { .. } => None,
                        });

                let saving_time = indirect_connections
                    .filter(|(start, _, _, end, time)| {
                        let time_with_menuing = time + 0; // TODO
                        let direct_time = connections
                            .get(&start)
                            .and_then(|targets| targets.get(&end));
                        match direct_time {
                            Some(direct_time) if time_with_menuing < *direct_time => true,
                            Some(_) => false,
                            None => true,
                        }
                    })
                    .inspect(|(start, via1, via2, end, time)| {
                        eprintln!("using {start}-{via1}-{via2}-{end}: {time}");
                    })
                    .map(|(start, _, _, end, time)| (start, end, time));

                saving_time
            })
            .collect();

        for (start, end, time) in bench_connections {
            connections.entry(start).or_default().insert(end, time);
        }
    }

    let mut text = String::new();
    for (from, row) in connections {
        let row = (0..=n)
            .map(|to| {
                if to == from {
                    return "0".to_string();
                }

                if to == 0 {
                    return RESTART_PENALTY.to_string();
                }

                match row.get(&to) {
                    Some(time) => time.to_string(),
                    None => PLACEHOLDER.into(),
                }
            })
            .collect::<Vec<_>>()
            .join(",");

        let _ = writeln!(&mut text, "[{row}]");
    }

    Ok(text)
}
fn collect_entries(
    path: &Path,
) -> Result<(u32, BTreeMap<u32, BTreeMap<u32, u32>>, Vec<BenchNode>)> {
    let dir = path.read_dir()?;

    let mut nodes = Vec::new();
    let mut benches = Vec::new();

    for entry in dir {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(true, |ext| ext != "tas") {
            continue;
        }

        anyhow::ensure!(
            entry.metadata()?.is_file(),
            "{} is not a file",
            path.display()
        );

        let stem = path
            .file_stem()
            .unwrap()
            .to_str()
            .ok_or_else(|| anyhow!("non-UTF8 path: {}", path.display()))?;
        let node = node_path(stem).ok_or_else(|| anyhow!("invalid filename: {stem}"))?;

        let start = node
            .start
            .parse::<Location>()
            .context("failed to parse start node")?;
        let end = node
            .end
            .parse::<Location>()
            .context("failed to parse end node")?;

        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("could not read {}", path.display()))?;
        let time = extract_node_time(&text)
            .with_context(|| format!("could not extract time from {}", path.display()))?;

        match (start, end) {
            (Location::Map(start), Location::Map(end)) => {
                let node = Node { start, end, time };
                nodes.push(node);
            }
            (Location::Bench(start), Location::Map(end)) => {
                benches.push(BenchNode::From { start, end, time });
            }
            (Location::Map(start), Location::Bench(end)) => {
                benches.push(BenchNode::To { start, end, time });
            }
            (Location::Bench(_), Location::Bench(_)) => {
                // are these useful?
            }
        }
    }

    let n: u32 = nodes
        .iter()
        .map(|node| node.start.max(node.end))
        .max()
        .ok_or_else(|| anyhow::anyhow!("no nodes present"))?;

    let mut map = BTreeMap::<u32, BTreeMap<u32, u32>>::new();
    for node in nodes {
        map.entry(node.start)
            .or_default()
            .insert(node.end, node.time);
    }

    Ok((n, map, benches))
}

fn extract_node_time(text: &str) -> Result<u32> {
    let last_line = text
        .lines()
        .rev()
        .find(|line| {
            // TODO: use regex?
            !line.is_empty()
                && line.starts_with("#")
                && line.contains(":")
                && line.contains(".")
                && line.ends_with(")")
        })
        .ok_or_else(|| anyhow!("could not find time comment"))?;

    let (_, frames) = last_line
        .trim_end_matches(")")
        .rsplit_once("(")
        .ok_or_else(|| anyhow!("last line '{last_line}' does not contain time"))?;
    let frames = frames.parse()?;

    Ok(frames)
}

#[derive(Debug)]
struct NodePath<'a> {
    prefix: &'a str,
    start: &'a str,
    end: &'a str,
}
impl std::fmt::Display for NodePath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}-{}", self.prefix, self.start, self.end)
    }
}

fn node_path(stem: &str) -> Option<NodePath> {
    let (prefix, rest) = stem.split_once('_')?;
    let (from, rest) = rest.split_once('-')?;
    let to = rest;

    Some(NodePath {
        prefix,
        start: from,
        end: to,
    })
}

#[derive(Debug, Clone, Copy)]
struct Node {
    start: u32,
    end: u32,
    time: u32,
}

#[derive(Debug, Clone)]
enum Location {
    Bench(String),
    Map(u32),
}
impl FromStr for Location {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u32>() {
            Ok(id) => Ok(Location::Map(id)),
            Err(_) => Ok(Location::Bench(s.to_owned())),
        }
    }
}

#[derive(Debug)]
enum BenchNode {
    From { start: String, end: u32, time: u32 },
    To { start: u32, end: String, time: u32 },
}

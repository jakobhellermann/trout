#[cfg(feature = "heap_profiling")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

mod solver;

use anyhow::{anyhow, Result};

type Length = u32;
type Table = Vec<Vec<Length>>;

fn strip_around<'a>(prefix: &str, suffix: &str, input: &'a str) -> Option<&'a str> {
    input.strip_prefix(prefix)?.strip_suffix(suffix)
}

fn parse_table(table: &str) -> Result<Table> {
    let table = table
        .lines()
        .map(|line| {
            let line = strip_around("[", "]", line)
                .ok_or_else(|| anyhow!("table doesn't contain arrays"))?;
            let connections = line
                .split(',')
                .map(|val| {
                    val.trim()
                        .parse::<Length>()
                        .map_err(|e| anyhow!("failed to parse int: {e}"))
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(connections)
        })
        .collect::<Result<Vec<_>>>()?;

    let length = table.get(0).ok_or_else(|| anyhow!("no table data"))?.len();
    if table[1..].iter().any(|row| row.len() != length) {
        anyhow::bail!("not every table row has the same length (expected {length})");
    }

    Ok(table)
}

struct Node(u32);
impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node {}", self.0)
    }
}

fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow!("missing first argument"))?;
    let table = std::fs::read_to_string(&path)?;

    let table = parse_table(&table)?;

    let start = std::time::Instant::now();

    let mut n_solutions = 0;
    let mut fastest_solution = None;

    #[cfg(feature = "heap_profiling")]
    let _profiler = dhat::Profiler::new_heap();

    let stats = solver::solve(
        &table,
        solver::SolverSettings {
            max_restarts: 1000,
            required_restarts: true,
            restart_penalty: 190,
            deduplicate_solutions: false,
        },
        |solution, time| {
            n_solutions += 1;
            match fastest_solution {
                None => fastest_solution = Some((solution.to_vec(), time)),
                Some((_, prev_time)) if time < prev_time => {
                    fastest_solution = Some((solution.to_vec(), time))
                }
                Some(_) => {}
            }
        },
    );
    let duration = start.elapsed();

    if let Some((solution, time)) = fastest_solution {
        println!("{:?} - {}", solution, time);
    }

    println!("Routing took {:02}s", duration.as_secs_f32());
    println!("{} solutions", n_solutions);
    println!("Pathfind function called {} times.", stats.iterations);

    Ok(())
}

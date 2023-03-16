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

/*fn table_to_petgraph(table: &Table) -> DiGraph<Node, Length> {
    let mut graph = DiGraph::new();

    for node_idx in 0..table.len() {
        graph.add_node(Node(node_idx as u32));
    }
    for (from, row) in table.iter().enumerate() {
        let from = from as u32;
        for (to, &length) in row.iter().enumerate() {
            let to = to as u32;

            if length == 60000 || length == 0 {
                continue;
            }

            graph.add_edge(from.into(), to.into(), length);
        }
    }

    graph
}*/

fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow!("missing first argument"))?;
    let table = std::fs::read_to_string(&path)?;

    let table = parse_table(&table)?;

    let start = std::time::Instant::now();
    let (mut solutions, stats) = solver::solve(
        &table,
        solver::SolverSettings {
            max_restarts: 1000,
            required_restarts: true,
            restart_penalty: 190,
            deduplicate_solutions: false,
        },
    );
    let duration = start.elapsed();

    solutions.sort_by_key(|a| a.1);
    for solution in solutions.iter().take(10) {
        println!("{:?}", solution);
    }

    println!("Routing took {:02}s", duration.as_secs_f32());
    println!("{} solutions", solutions.len());
    println!("Pathfind function called {} times.", stats.iterations);

    Ok(())
}

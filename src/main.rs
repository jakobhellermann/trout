#[cfg(feature = "heap_profiling")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use anyhow::{Context, Result};
use std::path::PathBuf;

fn solve_table(table: &str) -> Result<()> {
    let table = trout::parse_table(&table).context("could not parse table")?;

    let start = std::time::Instant::now();

    let max_solutions = 12;
    let mut best_solutions: Vec<(Vec<usize>, u32)> = Vec::new();

    #[cfg(feature = "heap_profiling")]
    let _profiler = dhat::Profiler::new_heap();

    let settings = trout::solver::SolverSettings {
        max_restarts: None,
        only_required_restarts: false,
        restart_penalty: 190,
    };
    let stats = trout::solver::solve(
        &table,
        &settings,
        trout::solver::emit_top_n_solutions(&mut best_solutions, max_solutions),
    );
    let duration = start.elapsed();

    for (route, time) in best_solutions[0..max_solutions.min(5)].iter().rev() {
        println!("{:?} - {}", route, time);
    }

    println!("Routing took {:02}s", duration.as_secs_f32());
    println!("{} solutions", stats.solutions_found);
    println!("Pathfind function called {} times.", stats.iterations);
    println!("Branches cut: {}", stats.cut_branches);
    println!("\n-- Settings used --");
    println!(
        "Only Dead End Restarts: {}",
        settings.only_required_restarts
    );
    match settings.max_restarts {
        Some(max_restarts) => {
            println!("Max Restart Count: {}", max_restarts)
        }
        None => {
            println!("Max Restart Count: -")
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let paths: Vec<_> = std::env::args().skip(1).map(PathBuf::from).collect();
    anyhow::ensure!(paths.len() > 0, "missing argument of path to table");

    for path in paths {
        println!("Solving {}...", path.display());

        let table = std::fs::read_to_string(&path)?;
        solve_table(&table)?;
        println!();
    }

    Ok(())
}

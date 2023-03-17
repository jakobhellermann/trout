#[cfg(feature = "heap_profiling")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use anyhow::{Context, Result};
use std::path::PathBuf;

fn solve_table(table: &str) -> Result<()> {
    let table = trout::parse_table(&table).context("could not parse table")?;

    let start = std::time::Instant::now();

    let max_solutions = 100;

    let mut previous_best = u32::MAX;
    let mut previous_worst = u32::MAX;
    let mut best_solutions: Vec<(Vec<usize>, u32)> = Vec::new();

    #[cfg(feature = "heap_profiling")]
    let _profiler = dhat::Profiler::new_heap();

    let settings = trout::solver::SolverSettings {
        max_restarts: Some(1),
        only_required_restarts: true,
        restart_penalty: 190,
    };
    let stats = trout::solver::solve(&table, &settings, |solution, time| {
        // perfect compat with c# solver: accept every solution (even if obsolete) and return high worst interested time
        let is_windup = best_solutions.len() < max_solutions;

        if time < previous_worst || is_windup {
            best_solutions.push((solution.to_vec(), time));
            best_solutions.sort_by_key(|&(_, time)| time);
            best_solutions.truncate(max_solutions);

            (previous_best, previous_worst) = best_solutions
                .iter()
                .fold((std::u32::MAX, std::u32::MIN), |(min, max), &(_, time)| {
                    (min.min(time), max.max(time))
                });
        }

        if is_windup {
            u32::MAX
        } else {
            previous_worst
        }
    });
    let duration = start.elapsed();

    for (route, time) in best_solutions[0..5].iter().rev() {
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

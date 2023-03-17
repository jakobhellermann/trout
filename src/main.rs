#[cfg(feature = "heap_profiling")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use anyhow::{Context, Result};
use std::path::PathBuf;

fn solve_table(table: &str) -> Result<()> {
    let table = trout::parse_table(&table).context("could not parse table")?;

    let start = std::time::Instant::now();

    let mut n_solutions = 0;
    let mut fastest_solution = None;

    #[cfg(feature = "heap_profiling")]
    let _profiler = dhat::Profiler::new_heap();

    let settings = trout::solver::SolverSettings {
        max_restarts: Some(2),
        only_required_restarts: true,
        restart_penalty: 190,
    };
    let stats = trout::solver::solve(&table, &settings, |solution, time| {
        n_solutions += 1;
        match fastest_solution {
            None => fastest_solution = Some((solution.to_vec(), time)),
            Some((_, prev_time)) if time < prev_time => {
                fastest_solution = Some((solution.to_vec(), time))
            }
            Some(_) => {}
        }
    });
    let duration = start.elapsed();

    if let Some((solution, time)) = fastest_solution {
        println!("{:?} - {}", solution, time);
    }

    println!("Routing took {:02}s", duration.as_secs_f32());
    println!("{} solutions", n_solutions);
    println!("Pathfind function called {} times.", stats.iterations);
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

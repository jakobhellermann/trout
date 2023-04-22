use js_sys::Array;
use trout::solver::{PossibleConnection, SolverSettings};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn do_solve(
    table: &str,
    settings: SolverSettings,
    max_solutions: usize,
    update_solutions: impl Fn(&[usize], u32, usize),
) -> Result<trout::solver::Stats, anyhow::Error> {
    let table = trout::parse_table(table)?;

    let mut previous_best = u32::MAX;
    let mut previous_worst = u32::MAX;
    let mut best_solutions = Vec::new();

    let stats = trout::solver::solve_table(&table, &settings, |solution, time| {
        let is_windup = best_solutions.len() < max_solutions;

        if time < previous_worst || is_windup {
            best_solutions.push((solution.to_vec(), time));
            best_solutions.sort_by_key(|&(_, time)| time);
            best_solutions.truncate(max_solutions);

            let updated_index = best_solutions.iter().position(|&(_, t)| t == time).unwrap();

            update_solutions(&solution, time, updated_index);

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

    Ok(stats)
}

#[wasm_bindgen]
pub fn solve(
    table: String,
    max_solutions: usize,
    max_restarts: Option<u32>,
    only_required_restarts: bool,
    restart_penalty: u32,
    callback: &js_sys::Function,
) -> Result<js_sys::Object, String> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let settings = trout::solver::SolverSettings {
        max_restarts,
        only_required_restarts,
        restart_penalty,
    };

    if max_solutions == 0 {
        return Err("expected nonzero max amount of solutions".into());
    }

    log(&format!("{:?}", settings));

    let stats = do_solve(
        &table,
        settings,
        max_solutions,
        |solution, time, updated_index| {
            let route = solution
                .iter()
                .copied()
                .map(JsValue::from)
                .collect::<Array>();

            let _ = callback.call3(
                &JsValue::NULL,
                &time.into(),
                &route.into(),
                &updated_index.into(),
            );
        },
    )
    .map_err(|e| format!("{:?}", e))?;

    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"iterations".into(), &stats.iterations.into()).unwrap();
    js_sys::Reflect::set(&obj, &"solutions".into(), &stats.solutions_found.into()).unwrap();
    js_sys::Reflect::set(&obj, &"cutBranches".into(), &stats.cut_branches.into()).unwrap();

    Ok(obj)
}

fn do_suggest(
    table: &str,
    settings: SolverSettings,
    time_to_beat: u32,
    emit_solution: impl Fn(PossibleConnection<'_>),
) -> Result<(), anyhow::Error> {
    let table = trout::parse_table(table)?;

    trout::solver::find_new_connections(&table, &settings, time_to_beat, emit_solution);

    Ok(())
}

#[wasm_bindgen]
pub fn suggest_solutions(
    table: String,
    max_restarts: Option<u32>,
    only_required_restarts: bool,
    restart_penalty: u32,
    time_to_beat: u32,
    callback: &js_sys::Function,
) -> Result<(), String> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let settings = trout::solver::SolverSettings {
        max_restarts,
        only_required_restarts,
        restart_penalty,
    };

    do_suggest(&table, settings, time_to_beat, |possible_connection| {
        let route = possible_connection
            .path
            .iter()
            .copied()
            .map(JsValue::from)
            .collect::<Array>();

        let args = js_sys::Array::from_iter([
            JsValue::from(possible_connection.start),
            JsValue::from(possible_connection.end),
            JsValue::from(possible_connection.time),
            JsValue::from(route),
        ]);
        let _ = callback.apply(&JsValue::NULL, &args);
    })
    .map_err(|e| format!("{:?}", e))?;

    Ok(())
}

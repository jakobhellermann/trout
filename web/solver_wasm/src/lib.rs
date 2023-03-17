use js_sys::Array;
use trout::solver::SolverSettings;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn doit(
    table: &str,
    settings: SolverSettings,
    max_solutions: usize,
    update_solutions: impl Fn(&[(Vec<usize>, u32)]),
) -> Result<(), anyhow::Error> {
    let table = trout::parse_table(table)?;

    let mut previous_best = u32::MAX;
    let mut previous_worst = u32::MAX;
    let mut best_solutions = Vec::new();

    let mut i = 0;
    trout::solver::solve(&table, &settings, |solution, time| {
        if time < previous_worst {
            i += 1;

            best_solutions.push((solution.to_vec(), time));
            best_solutions.sort_by_key(|&(_, time)| time);
            best_solutions.truncate(max_solutions);

            update_solutions(&best_solutions);

            (previous_best, previous_worst) = best_solutions
                .iter()
                .fold((std::u32::MAX, std::u32::MIN), |(min, max), &(_, time)| {
                    (min.min(time), max.max(time))
                });
        }

        if i == 10000 {
            panic!();
        }
    });

    Ok(())
}

#[wasm_bindgen]
pub fn solve(
    table: String,
    max_solutions: usize,
    max_restarts: Option<u32>,
    only_required_restarts: bool,
    restart_penalty: u32,
    callback: &js_sys::Function,
) -> Result<(), String> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let settings = trout::solver::SolverSettings {
        max_restarts,
        only_required_restarts,
        restart_penalty,
    };

    log(&format!("{:?}", settings));

    doit(&table, settings, max_solutions, |new_solutions| {
        let new_solutions: Array = new_solutions
            .iter()
            .map(|(solution, time)| {
                let route = solution
                    .iter()
                    .copied()
                    .map(JsValue::from)
                    .collect::<Array>();

                let obj = js_sys::Object::new();
                js_sys::Reflect::set(&obj, &"time".into(), &JsValue::from(*time)).unwrap();
                js_sys::Reflect::set(&obj, &"route".into(), &route.into()).unwrap();

                JsValue::from(obj)
            })
            .collect();
        let _ = callback.call1(&JsValue::NULL, &new_solutions);
    })
    .map_err(|e| format!("{:?}", e))
}

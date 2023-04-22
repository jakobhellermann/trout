// code taking and adapted from https://github.com/TheRoboManTAS/Celeste-TAS-lobby-router/, credit goes to @TheRoboManTAS

type Time = u32;
type NodeIdx = usize;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub start: NodeIdx,
    pub end: NodeIdx,
    pub time: Time,
}

#[derive(Debug)]
struct PlaceInfo {
    node: NodeIdx,

    targets: Vec<NodeIdx>,
    targeters: Vec<NodeIdx>,

    times: Vec<Time>,
}

impl PlaceInfo {
    fn frames_to(&self, node: NodeIdx) -> Time {
        let idx = self
            .targets
            .iter()
            .position(|&target| target == node)
            .unwrap();
        self.times[idx]
    }
}

#[derive(Debug)]
pub struct SolverSettings {
    pub max_restarts: Option<u32>,
    pub only_required_restarts: bool,
    pub restart_penalty: Time,
}

pub struct Stats {
    pub iterations: u32,
    pub solutions_found: u32,
    pub cut_branches: u32,
}

/// `emit_solution` gets called for each new solution, and returns the worst new interesting time
pub fn solve_table<F>(table: &[Vec<u32>], settings: &SolverSettings, emit_solution: F) -> Stats
where
    F: FnMut(&[NodeIdx], Time) -> Time,
{
    let files: Vec<FileInfo> = collect_files(table);
    solve_files(&files, settings, emit_solution)
}

/// `emit_solution` gets called for each new solution, and returns the worst new interesting time
pub fn solve_files<F>(files: &[FileInfo], settings: &SolverSettings, emit_solution: F) -> Stats
where
    F: FnMut(&[NodeIdx], Time) -> Time,
{
    let n = files
        .iter()
        .map(|file| file.start.max(file.end))
        .max()
        .expect("empty files passed")
        + 1;

    let nodes: Vec<PlaceInfo> = collect_nodes(n, &files);

    let start = 0;
    let finish = n - 1;

    let lowest_times = collect_lowest_times(n, &nodes);

    let global_lower_bound: Time = lowest_times.iter().sum();

    let mut cx = SolverContext {
        settings,
        n,
        local_lower_bound: global_lower_bound,
        cut_branches: 0,
        lowest_times,
        worst_time_of_interest: std::u32::MAX,
        solutions_found: 0,
        emit_solution,
        start,
        finish,
        iterations: 0,
        restart_count: 0,
        can_go: vec![true; n],
        trail: vec![0; n + nodes[start].targets.len()],
        index: 0,
        visit_count: 0,
        nodes: &nodes,
    };
    cx.path_find(start);

    let stats = Stats {
        solutions_found: cx.solutions_found,
        iterations: cx.iterations,
        cut_branches: cx.cut_branches,
    };

    stats
}

pub struct PossibleConnection<'a> {
    pub start: NodeIdx,
    pub end: NodeIdx,

    pub path: &'a [NodeIdx],
    pub time: Time,
    pub frame_difference: Time,
}

/// `emit_new_connection` gets called for each possible connection
pub fn find_new_connections<F>(
    table: &[Vec<u32>],
    settings: &SolverSettings,
    mut emit_new_connection: F,
) where
    F: FnMut(PossibleConnection<'_>),
{
    let n = table[0].len();

    let files: Vec<FileInfo> = collect_files(table);
    let nodes: Vec<PlaceInfo> = collect_nodes(n, &files);

    let Some(reference_solution) = find_single_solution(&files, settings) else { return };

    let mut connections = Vec::with_capacity((nodes.len() - 1) * (nodes.len() - 1));
    for start in 0..nodes.len() - 1 {
        for end in 1..nodes.len() - 1 {
            connections.push((start, end));
        }
    }

    for (connection_start, connection_end) in connections {
        if connection_end == connection_start
            || connection_start == 0 && connection_end == nodes.len()
        {
            continue;
        }

        if files
            .iter()
            .any(|file| file.start == connection_start && file.end == connection_end)
        {
            continue;
        }

        let files = edit_files_to_test_new_connection(&files, connection_start, connection_end);
        let Some(new_solution) = find_single_solution(&files, settings) else { continue };
        if new_solution.1 > reference_solution.1 {
            continue;
        }
        let frame_difference = reference_solution.1 - new_solution.1;

        emit_new_connection(PossibleConnection {
            start: connection_start,
            end: connection_end,
            path: &new_solution.0,
            time: new_solution.1,
            frame_difference,
        });
    }
}

/// - remove start-* and *-end
/// - add start-end with time 0
fn edit_files_to_test_new_connection(
    files: &[FileInfo],
    start: NodeIdx,
    end: NodeIdx,
) -> Vec<FileInfo> {
    let mut new_files = files.to_vec();
    new_files.retain(|file| file.start != start && file.end != end);

    new_files.insert(
        if start == 0 { 0 } else { new_files.len() },
        FileInfo {
            start,
            end,
            time: 0,
        },
    );

    new_files
}

fn collect_lowest_times(n: usize, nodes: &[PlaceInfo]) -> Vec<u32> {
    let mut lowest_times = vec![Time::MAX; n];
    lowest_times[0] = 0;
    for node in nodes {
        for (&target, &time) in node.targets.iter().zip(node.times.iter()) {
            lowest_times[target] = lowest_times[target].min(time);
        }
    }
    lowest_times
}

/// extract file connections from table
fn collect_files(table: &[Vec<u32>]) -> Vec<FileInfo> {
    table
        .iter()
        .enumerate()
        .flat_map(|(start, row)| {
            let mut row = row.iter().enumerate();
            let restart = row.next().unwrap().0;
            // TODO
            assert!(restart == 190 || restart == 0);

            row.map(move |(end, &time)| FileInfo { start, end, time })
        })
        .filter(|file| file.time < 60000 && file.start != file.end)
        .collect()
}

/// collect files into a datastructure better suited for traversal
fn collect_nodes(n: usize, files: &[FileInfo]) -> Vec<PlaceInfo> {
    let mut nodes: Vec<_> = (0..n)
        .map(|node| {
            let (targets, times) = files
                .iter()
                .filter(|i| i.start == node)
                .map(|outgoing| (outgoing.end, outgoing.time))
                .unzip();
            PlaceInfo {
                node,
                targets,
                targeters: Vec::new(),
                times,
            }
        })
        .collect();
    for node_idx in 0..nodes.len() {
        let targeters = nodes
            .iter()
            .filter(|o| o.targets.contains(&node_idx))
            .map(|o| o.node)
            .collect();

        nodes[node_idx].targeters = targeters;
    }

    nodes
}

struct SolverContext<'a, F> {
    settings: &'a SolverSettings,
    nodes: &'a [PlaceInfo],

    local_lower_bound: Time,
    cut_branches: u32,
    lowest_times: Vec<Time>,
    worst_time_of_interest: Time,

    solutions_found: u32,
    emit_solution: F,

    n: usize,
    start: NodeIdx,
    finish: NodeIdx,

    iterations: u32,
    restart_count: u32,

    index: usize,
    visit_count: usize,
    can_go: Vec<bool>,

    trail: Vec<NodeIdx>,
}

impl<F> SolverContext<'_, F>
where
    F: FnMut(&[NodeIdx], Time) -> Time,
{
    fn can_restart(&self, pos: NodeIdx, must: bool) -> bool {
        if self.settings.only_required_restarts && !must {
            return false;
        }

        match self.settings.max_restarts {
            None => pos != self.start,
            Some(max_restarts) => pos != self.start && (self.restart_count < max_restarts),
        }
    }

    fn place_count(&self) -> usize {
        self.n - 1
    }

    fn emit_solution(&mut self) {
        self.solutions_found += 1;

        let solution = &self.trail[0..self.index + 1];
        let time: Time = solution
            .windows(2)
            .map(|segment| {
                let to = segment[1];
                let from = segment[0];

                if to == self.start {
                    190
                } else {
                    self.nodes[from].frames_to(to)
                }
            })
            .sum();

        self.worst_time_of_interest = (self.emit_solution)(solution, time);
    }

    fn path_find(&mut self, pos: NodeIdx) {
        self.trail[self.index] = pos;
        self.iterations += 1;

        if pos == self.finish {
            if self.visit_count == self.place_count() {
                self.emit_solution();
            }
            return;
        }

        if self.local_lower_bound >= self.worst_time_of_interest {
            self.cut_branches += 1;
            return;
        }

        let added_time = if self.index == 0 {
            0
        } else {
            if self.trail[self.index] == self.start {
                self.settings.restart_penalty
            } else {
                self.nodes[self.trail[self.index - 1]].frames_to(self.trail[self.index])
            }
        };
        let update_lower_bound = added_time - self.lowest_times[pos];

        let targets = &self.nodes[pos].targets;

        let mut dead_end = None;
        'targets: for &target in targets {
            if self.can_go[target] {
                let node = &self.nodes[target];
                for &targeter in &node.targeters {
                    if self.can_go[targeter] {
                        continue 'targets;
                    }
                }

                if dead_end.is_some() {
                    return;
                }

                dead_end = Some(target);
            }
        }

        if let Some(dead_end) = dead_end {
            self.visit_count += 1;
            self.index += 1;
            self.can_go[dead_end] = false;
            self.local_lower_bound += update_lower_bound;

            self.path_find(dead_end);

            self.local_lower_bound -= update_lower_bound;
            self.visit_count -= 1;
            self.index -= 1;
            self.can_go[dead_end] = true;
            return;
        }

        let mut must_restart = true;
        for &target in targets {
            if self.can_go[target] {
                self.visit_count += 1;
                self.index += 1;
                self.can_go[target] = false;
                self.local_lower_bound += update_lower_bound;

                self.path_find(target);

                self.local_lower_bound -= update_lower_bound;
                self.visit_count -= 1;
                self.index -= 1;
                self.can_go[target] = true;

                must_restart = false;
            }
        }

        if self.can_restart(pos, must_restart) {
            self.index += 1;
            self.restart_count += 1;
            self.local_lower_bound += update_lower_bound;

            self.path_find(self.start);

            self.local_lower_bound -= update_lower_bound;
            self.restart_count -= 1;
            self.index -= 1;
        }
    }
}

pub fn emit_only_best(
    solution: &mut Option<(Vec<NodeIdx>, Time)>,
) -> impl FnMut(&[NodeIdx], Time) -> Time + '_ {
    |s: &[NodeIdx], time: Time| -> Time {
        let solution = solution.get_or_insert_with(|| (s.to_vec(), time));
        if time < solution.1 {
            *solution = (s.to_vec(), time);
        }
        solution.1
    }
}

pub fn emit_top_n_solutions(
    best_solutions: &mut Vec<(Vec<usize>, u32)>,
    max_solutions: usize,
) -> impl FnMut(&[usize], u32) -> u32 + '_ {
    let mut previous_worst = u32::MAX;

    move |solution, time| {
        // perfect compat with c# solver: accept every solution (even if obsolete) and return high worst interested time
        let is_windup = best_solutions.len() < max_solutions;

        if time < previous_worst || is_windup {
            best_solutions.push((solution.to_vec(), time));
            best_solutions.sort_by_key(|&(_, time)| time);
            best_solutions.truncate(max_solutions);

            previous_worst = best_solutions
                .iter()
                .map(|&(_, time)| time)
                .max()
                .unwrap_or(std::u32::MAX);
        }

        if is_windup {
            u32::MAX
        } else {
            previous_worst
        }
    }
}

fn find_single_solution(
    files: &[FileInfo],
    settings: &SolverSettings,
) -> Option<(Vec<NodeIdx>, Time)> {
    let mut solution = None;
    solve_files(files, settings, emit_only_best(&mut solution));
    solution
}

// code taking and adapted from https://github.com/TheRoboManTAS/Celeste-TAS-lobby-router/, credit goes to @TheRoboManTAS

type Time = u32;
type NodeIdx = usize;

#[derive(Debug)]
struct FileInfo {
    start: NodeIdx,
    end: NodeIdx,
    time: Time,
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
}

pub fn solve<F>(table: &[Vec<u32>], settings: &SolverSettings, emit_solution: F) -> Stats
where
    F: FnMut(&[NodeIdx], Time),
{
    let n = table[0].len();

    let files: Vec<FileInfo> = table
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
        .collect();

    let mut nodes: Vec<PlaceInfo> = (0..n)
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

    let start = 0;
    let finish = n - 1;

    let mut cx = SolverContext {
        settings,
        n,
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
    };

    stats
}

struct SolverContext<'a, F> {
    settings: &'a SolverSettings,
    nodes: &'a [PlaceInfo],

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

impl<F: FnMut(&[NodeIdx], Time)> SolverContext<'_, F> {
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

        (self.emit_solution)(solution, time);
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

            self.path_find(dead_end);

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

                self.path_find(target);

                self.visit_count -= 1;
                self.index -= 1;
                self.can_go[target] = true;

                must_restart = false;
            }
        }

        if self.can_restart(pos, must_restart) {
            self.index += 1;
            self.restart_count += 1;
            self.path_find(self.start);
            self.restart_count -= 1;
            self.index -= 1;
        }
    }
}

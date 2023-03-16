type Time = u32;
type NodeIdx = usize;

struct FileInfo {
    start: NodeIdx,
    end: NodeIdx,
    time: Time,
}

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
pub struct Solution(Vec<NodeIdx>, Time);

pub struct SolverSettings {
    pub max_restarts: u32,
    pub required_restarts: bool,
    pub restart_penalty: Time,
}

pub fn solve(table: &[Vec<u32>], settings: SolverSettings) -> Vec<Solution> {
    let n = table[0].len();

    let files: Vec<FileInfo> = table
        .iter()
        .enumerate()
        .flat_map(|(start, row)| {
            row.iter()
                .enumerate()
                .map(move |(end, &time)| FileInfo { start, end, time })
        })
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
    let finish = n;

    let mut cx = SolverContext {
        settings,
        n,
        solutions: Vec::new(),
        start,
        finish,
        iterations: 0,
        restart_count: 0,
        can_go: vec![true; n],
        //inf_restarts: settings.max_restarts.is_none(),
        inf_restarts: false, // TODO
        trail: vec![0; n + nodes[start].targets.len()],
        index: 0,
        visit_count: 0,
        nodes: &nodes,
    };
    cx.path_find(start);

    cx.solutions
}

struct SolverContext<'a> {
    settings: SolverSettings,
    nodes: &'a [PlaceInfo],

    solutions: Vec<Solution>,

    n: usize,
    start: NodeIdx,
    finish: NodeIdx,

    iterations: u32,
    restart_count: u32,
    inf_restarts: bool,

    index: usize,
    visit_count: usize,
    can_go: Vec<bool>,

    trail: Vec<NodeIdx>,
}

impl SolverContext<'_> {
    fn can_restart(&self, pos: NodeIdx, must: bool) -> bool {
        match (self.inf_restarts, self.settings.required_restarts) {
            (true, true) => pos != self.start && must,
            (true, false) => pos != self.start,
            (false, true) => {
                pos != self.start && (self.restart_count < self.settings.max_restarts) && must
            }
            (false, false) => {
                pos != self.start && (self.restart_count < self.settings.max_restarts)
            }
        }
    }

    fn place_count(&self) -> usize {
        self.n - 1
    }

    fn path_find(&mut self, pos: NodeIdx) {
        println!("path_find {pos} index={}", self.index);

        self.trail[self.index] = pos;
        self.iterations += 1;

        if pos == self.finish {
            if self.visit_count == self.place_count() {
                let truncated = &self.trail[0..self.index + 1];
                let time = truncated
                    .iter()
                    .skip(1)
                    .enumerate()
                    .map(|(e, &i)| {
                        if e == self.start {
                            self.settings.restart_penalty
                        } else {
                            self.nodes[self.trail[i]].frames_to(e)
                        }
                    })
                    .sum();
                self.solutions.push(Solution(truncated.to_vec(), time));
            }
            return;
        }

        let targets = &self.nodes[pos].targets;

        let mut has_dead_end = false;
        let mut dead_end = 0;
        'targets: for &target in targets {
            if self.can_go[target] {
                let node = &self.nodes[target];
                for &targeter in &node.targeters {
                    if self.can_go[targeter] {
                        continue 'targets;
                    }
                }

                if has_dead_end {
                    return;
                }

                has_dead_end = true;
                dead_end = target;
            }
        }

        if has_dead_end {
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

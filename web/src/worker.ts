import init, { solve, suggest_solutions } from "../solver_wasm/out/solver_wasm";

export type WorkerRequest = {
    eventType: "INITIALIZE";
} | {
    eventType: "CALL",
    params: SolveParams,
} | {
    eventType: "SUGGEST",
    params: SuggestParams,
};
export type WorkerResponse = {
    eventType: "INITIALIZED";
} | {
    eventType: "EMIT",
    solution: Solution;
    updatedIndex: number,
} | {
    eventType: "EMIT_SUGGESTION",
    suggestion: Suggestion;
} | {
    eventType: "ERROR",
    error: Error,
} | {
    eventType: "FINISH";
    stats: Stats,
} | {
    eventType: "FINISH_SUGGESTION";
};

export type Stats = {
    iterations: number;
    solutions: number;
    cutBranches: number;
};

export type SolveParams = {
    table: string,
    maxRestarts: number | undefined,
    onlyRequiredRestarts: boolean,
    restartPenalty: number,

    maxSolutions: number;
};
export type SuggestParams = {
    table: string,
    maxRestarts: number | undefined,
    onlyRequiredRestarts: boolean,
    restartPenalty: number,

    timeToBeat: number;
};

export type Solution = { time: number, route: number[]; };
export type Suggestion = { start: number, end: number, time: number, route: number[]; };


function post(message: WorkerResponse) {
    self.postMessage(message);
}

self.addEventListener("message", (message: MessageEvent<WorkerRequest>) => {
    if (message.data.eventType == "INITIALIZE") {
        init().then(() => post({ eventType: "INITIALIZED" }));
    } else if (message.data.eventType == "CALL") {
        let { table, maxSolutions, maxRestarts, onlyRequiredRestarts, restartPenalty } = message.data.params;

        try {
            let stats = solve(table, maxSolutions, maxRestarts, onlyRequiredRestarts, restartPenalty, (time: number, route: number[], updatedIndex: number) => {
                post({
                    eventType: "EMIT",
                    solution: {
                        time, route
                    },
                    updatedIndex,
                });
            }) as Stats;
            post({
                eventType: "FINISH",
                stats,
            });
        } catch (error) {
            post({
                eventType: "ERROR",
                error: new Error(error),
            });
        }
    } else if (message.data.eventType == "SUGGEST") {
        let { table, maxRestarts, onlyRequiredRestarts, restartPenalty, timeToBeat } = message.data.params;

        suggest_solutions(table, maxRestarts, onlyRequiredRestarts, restartPenalty, timeToBeat, (start: number, end: number, time: number, route: number[]) => {
            post({
                eventType: "EMIT_SUGGESTION",
                suggestion: {
                    start, end, time, route
                }
            });
        });
        post({
            eventType: "FINISH_SUGGESTION",
        });
    } else {
        let _: never = message.data;
    }
});

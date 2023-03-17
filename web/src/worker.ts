import init, { solve } from "../solver_wasm/out/solver_wasm";

export type WorkerRequest = {
    eventType: "INITIALIZE";
} | {
    eventType: "CALL",
    params: Params,
};
export type WorkerResponse = {
    eventType: "INITIALIZED";
} | {
    eventType: "EMIT",
    solution: Solution;
    updatedIndex: number,
} | {
    eventType: "ERROR",
    error: Error,
} | {
    eventType: "FINISH";
};

export type Params = {
    table: string,
    maxSolutions: number;
    maxRestarts: number | undefined,
    onlyRequiredRestarts: boolean,
    restartPenalty: number,
};
export type Solution = { time: number, route: number[]; };

function post(message: WorkerResponse) {
    self.postMessage(message);
}

self.addEventListener("message", (message: MessageEvent<WorkerRequest>) => {
    if (message.data.eventType == "INITIALIZE") {
        init().then(() => post({ eventType: "INITIALIZED" }));
    } else if (message.data.eventType == "CALL") {
        let { table, maxSolutions, maxRestarts, onlyRequiredRestarts, restartPenalty } = message.data.params;

        try {
            solve(table, maxSolutions, maxRestarts, onlyRequiredRestarts, restartPenalty, (time: number, route: number[], updatedIndex: number) => {
                post({
                    eventType: "EMIT",
                    solution: {
                        time, route
                    },
                    updatedIndex,
                });
            });
            post({
                eventType: "FINISH",
            });
        } catch (error) {
            post({
                eventType: "ERROR",
                error: new Error(error),
            });
        }
    }
});

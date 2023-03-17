import type { Params, Solution, WorkerRequest, WorkerResponse } from "./worker";

let onSolutionCallback: (solutions: Solution[]) => void;
export function setOnSolutions(onSolution: (solutions: Solution[]) => void) {
    onSolutionCallback = onSolution;

}

type WorkerState = {
    initialized: false,
    running: false,
    worker: null,
} | {
    initialized: true,
    running: boolean,
    worker: Worker,
};

let workerState: WorkerState = {
    initialized: false,
    running: false,
    worker: null,
};

function createInitializedWorker(): Promise<Worker> {
    let worker = new Worker(new URL('worker.ts', import.meta.url), { type: "module" });
    let initMessage: WorkerRequest = { eventType: "INITIALIZE" };

    return new Promise((resolve, reject) => {
        worker.onerror = reject;
        worker.onmessageerror = reject;
        worker.onmessage = (message: MessageEvent<WorkerResponse>) => {
            if (message.data.eventType == "INITIALIZED") {
                worker.onmessage = function () { };
                resolve(worker);
            }
        };
        worker.postMessage(initMessage);
    });
}
let loadWorker = () => createInitializedWorker().then(worker => {
    worker.addEventListener("message", workerHandler);
    workerState = {
        initialized: true,
        running: false,
        worker,
    };
});

function workerHandler(message: MessageEvent<WorkerResponse>) {
    if (message.data.eventType == "INITIALIZED") {
        throw new Error("double initialization");
    } else if (message.data.eventType == "EMIT") {
        onSolutionCallback(message.data.solutions);
    } else if (message.data.eventType == "FINISH") {
        console.timeEnd("solve");
        workerState.running = false;
    }
}

loadWorker();


export function solve(params: Params) {
    if (!workerState.initialized) {
        console.warn("attempted to solve before initialization");
        return;
    }

    if (workerState.running) {
        console.warn("terminating worker for new request");
        console.timeEnd("solve");
        workerState.worker.terminate();
        loadWorker().then(() => {
            solve(params);
        });
        return;
    }

    console.time("solve");
    workerState.running = true;
    let message: WorkerRequest = {
        eventType: "CALL",
        params,
    };
    workerState.worker.postMessage(message);
}

import type { SolveParams, Solution, Stats, WorkerRequest, WorkerResponse, SuggestParams, Suggestion } from "./worker";

let onSolutionCallback: (solution: Solution, updatedIndex: number) => void;
export function setOnSolutions(onSolution: (solution: Solution, updatedIndex: number) => void) {
    onSolutionCallback = onSolution;
}

let onSuggestionCallback: (suggestion: Suggestion) => void;
export function setOnSuggestion(onSuggestion: (suggestion: Suggestion) => void) {
    onSuggestionCallback = onSuggestion;
}

type WorkerState = {
    initialized: false,
    runningSolve: false,
    runningSuggest: false,
    worker: null,
} | {
    initialized: true,
    runningSolve: boolean,
    runningSuggest: boolean,
    worker: Worker,
};

let workerState: WorkerState = {
    initialized: false,
    runningSolve: false,
    runningSuggest: false,
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
    worker.addEventListener("error", (e) => onError(new Error(e.message)));
    worker.addEventListener("message", workerHandler);
    workerState = {
        initialized: true,
        runningSolve: false,
        runningSuggest: false,
        worker,
    };
});

let onFinishSolve = (stats: Stats) => { };
let onFinishSuggest = () => { };
let onError: (error: Error) => void = (e) => { };

function workerHandler(message: MessageEvent<WorkerResponse>) {
    if (message.data.eventType == "INITIALIZED") {
        throw new Error("double initialization");
    } else if (message.data.eventType == "EMIT") {
        onSolutionCallback(message.data.solution, message.data.updatedIndex);
    } else if (message.data.eventType == "EMIT_SUGGESTION") {
        onSuggestionCallback(message.data.suggestion);
    } else if (message.data.eventType == "ERROR") {
        onError(message.data.error);
    } else if (message.data.eventType == "FINISH") {
        console.timeEnd("solve");
        workerState.runningSolve = false;
        onFinishSolve(message.data.stats);
    } else if (message.data.eventType == "FINISH_SUGGESTION") {
        console.timeEnd("suggest");
        workerState.runningSuggest = false;
        onFinishSuggest();
    } else {
        let _: never = message.data;
    }
}

loadWorker();

export function solve(params: SolveParams): Promise<Stats | undefined> {
    if (!workerState.initialized) {
        console.warn("attempted to solve before initialization");
        return Promise.resolve(undefined);
    }

    if (workerState.runningSolve || workerState.runningSuggest) {
        console.warn("terminating worker for new request");
        console.timeEnd("solve");
        workerState.worker.terminate();
        return loadWorker().then(() => {
            return solve(params);
        });
    }

    console.time("solve");
    workerState.runningSolve = true;
    let message: WorkerRequest = {
        eventType: "CALL",
        params,
    };
    workerState.worker.postMessage(message);

    return new Promise((resolve, reject) => {
        onFinishSolve = resolve;
        onError = reject;
    });
}

export function suggest(params: SuggestParams): Promise<void> {
    if (!workerState.initialized) {
        console.warn("attempted to solve before initialization");
        return Promise.resolve(undefined);
    }

    if (workerState.runningSolve || workerState.runningSuggest) {
        console.warn("terminating worker for new request");
        console.timeEnd("suggest");
        workerState.worker.terminate();
        return loadWorker().then(() => {
            return suggest(params);
        });
    }

    console.time("suggest");
    workerState.runningSuggest = true;
    let message: WorkerRequest = {
        eventType: "SUGGEST",
        params,
    };
    workerState.worker.postMessage(message);

    return new Promise((resolve, reject) => {
        onFinishSuggest = resolve;
        onError = reject;
    });
}

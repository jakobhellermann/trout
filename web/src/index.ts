import { solve, setOnSolutions, suggest, setOnSuggestion } from "./solver";
import type { Solution, Suggestion } from "./worker";
import "./theme";

let solveBtn = document.getElementById("solveBtn")!;
let solveOutputList = document.getElementById("solveOutputList")!;
let solveLoadingIndicator = document.getElementById("solveLoadingIndicator")!;
let suggestBtn = document.getElementById("suggestBtn")! as HTMLButtonElement;
let suggestOutputList = document.getElementById("suggestOutputList")!;
let suggestLoadingIndicator = document.getElementById("suggestLoadingIndicator")!;
let errorMessage = document.getElementById("errorMessage")!;
let statsMessage = document.getElementById("statsMessage")!;
let settingsForm = document.getElementById("settings")!;

let inputTimeTable = document.getElementById("timeTable") as HTMLTextAreaElement;
let inputNSolutions = document.getElementById("nSolutions") as HTMLInputElement;
let inputMaxRestarts = document.getElementById("maxRestarts") as HTMLInputElement;
let inputOnlyRequiredRestarts = document.getElementById("onlyRequiredRestarts") as HTMLInputElement;
let inputRestartPenalty = document.getElementById("restartPenalty") as HTMLInputElement;
let inputSuggestThreshold = document.getElementById("suggestFrameThreshold") as HTMLInputElement;

suggestBtn.disabled = true;

let bestSolution: number | undefined = undefined;

inputTimeTable.addEventListener("keypress", (e) => {
    if (e.key === "Enter" && e.ctrlKey) {
        e.preventDefault();
        solveBtn.click();
    }
});


let setSolverSpinning = (active: boolean) => solveLoadingIndicator.classList.toggle("disabled", !active);
let setSuggestSpinning = (active: boolean) => suggestLoadingIndicator.classList.toggle("disabled", !active);

function createRouteElement(route: number[]): HTMLElement {
    let routeEl = document.createElement("code");

    let routeElements = route.flatMap<Element | string>((number, index) => {
        if (index == 0) return [];

        let elements: (Element | string)[] = [];

        let pad = false;

        if (number === 0) {
            let restart = document.createElement("span");
            restart.textContent = "[R] ";
            restart.className = "restart";
            elements.push(restart);
        } else {
            let num = number.toString();
            elements.push(pad ? num.padEnd(3) : (num + " "));
            length = num.length;
        }

        return elements;
    });
    routeEl.replaceChildren(...routeElements);
    return routeEl;
}

function createSolutionLi(solution: Solution): HTMLLIElement {
    let timeEl = document.createElement("span");
    timeEl.className = "time";
    timeEl.textContent = `${formatDuration(solution.time * 17, true)} (${solution.time}): `;

    let routeEl = createRouteElement(solution.route);

    let li = document.createElement("li");
    li.className = "newSolution";
    li.replaceChildren(timeEl, " ", routeEl);

    return li;
}

function truncateChildren(element: Element, length: number) {
    while (element.childElementCount > length) {
        element.removeChild(element.lastChild!);
    }
}

function insertChildAt(parent: Element, child: Element, index: number) {
    if (index >= parent.childElementCount) {
        parent.appendChild(child);
    } else {
        solveOutputList.insertBefore(child, parent.children[index]);
    }
}

function createSuggestionLi(suggestion: Suggestion): HTMLLIElement {
    if (bestSolution === undefined) {
        throw new Error("attempted to suggest new drafts without solving first");
    }

    let frameDifference = bestSolution - suggestion.time;

    let nodeEl = document.createElement("span");
    nodeEl.textContent = `${suggestion.start}-${suggestion.end}`;
    nodeEl.className = "suggestConnection";

    let timeEl = document.createElement("span");
    timeEl.className = "time";
    timeEl.textContent = `[${frameDifference}f]`;

    let routeEl = createRouteElement(suggestion.route);

    let li = document.createElement("li");
    li.className = "newSolution";
    li.replaceChildren(nodeEl, " ", timeEl, " ", routeEl);

    return li;
}



setOnSolutions((solution, updatedIndex) => {
    let nSolutions = Number(inputNSolutions.value);

    if (bestSolution === undefined || solution.time < bestSolution) {
        bestSolution = solution.time;
    }

    let li = createSolutionLi(solution);
    insertChildAt(solveOutputList, li, updatedIndex);
    truncateChildren(solveOutputList, nSolutions);
});

setOnSuggestion((suggestion) => {
    let li = createSuggestionLi(suggestion);
    suggestOutputList.appendChild(li);
});

function formatDuration(millis: number, alwaysIncludeMinutes?: boolean) {
    let ms = millis % 1000;
    millis = (millis - ms) / 1000;
    let secs = millis % 60;
    millis = (millis - secs) / 60;
    let mins = millis % 60;
    let hrs = (millis - mins) / 60;

    let str = `${secs.toString().padStart(2, "0")}.${ms.toString().padStart(3, "0")}`;
    if (alwaysIncludeMinutes || mins != 0) {
        str = `${mins.toString().padStart(2, "0")}:${str}`;
    }
    if (hrs != 0) {
        str = `${hrs.toString().padStart(2, "0")}:${str}`;
    }

    return str;
}

solveBtn.addEventListener("click", () => {
    setSolverSpinning(true);
    setSuggestSpinning(false);
    suggestBtn.disabled = true;
    bestSolution = undefined;

    solveOutputList.replaceChildren();
    suggestOutputList.replaceChildren();
    statsMessage.textContent = "";
    errorMessage.textContent = "";

    let table = inputTimeTable.value;
    let maxSolutions = Number(inputNSolutions.value);
    let maxRestarts = inputMaxRestarts.value !== "" ? Number(inputMaxRestarts.value) : undefined;
    let onlyRequiredRestarts = inputOnlyRequiredRestarts.checked;
    // let restartPenalty = Number(inputRestartPenalty.value);
    let restartPenalty = 190;

    let start = Date.now();
    solve({
        table,
        maxSolutions,
        maxRestarts,
        onlyRequiredRestarts,
        restartPenalty,
    })
        .then(stats => {
            let end = Date.now();

            if (stats) {
                let msg = `${stats.solutions} solutions found, ${stats.iterations} calls to pathfind function and ${stats.cutBranches} cut branches, in ${formatDuration(end - start, true)} `;
                statsMessage.textContent = msg;
            } else {
                statsMessage.textContent = "Code not fully loaded yet, try again later";
            }
            suggestBtn.disabled = false;
        })
        .catch((error: Error) => {
            errorMessage.textContent = `Error: ${error.message}`;
            console.error(error);
        })
        .finally(() => {
            setSolverSpinning(false);
        });
});

suggestBtn.addEventListener("click", () => {
    if (bestSolution === undefined) {
        throw new Error("attempted to suggest new drafts without solving first");
    }

    setSuggestSpinning(true);

    suggestOutputList.replaceChildren();

    let table = inputTimeTable.value;
    let maxRestarts = inputMaxRestarts.value !== "" ? Number(inputMaxRestarts.value) : undefined;
    let onlyRequiredRestarts = inputOnlyRequiredRestarts.checked;
    // let restartPenalty = Number(inputRestartPenalty.value);
    let restartPenalty = 190;

    let frameThreshold = Number(inputSuggestThreshold.value);
    let timeToBeat = bestSolution - frameThreshold;

    suggest({
        table, maxRestarts, onlyRequiredRestarts, restartPenalty, timeToBeat
    })
        .catch((error: Error) => {
            errorMessage.textContent = `Error: ${error.message}`;
            console.error(error);
        })
        .finally(() => {
            setSuggestSpinning(false);
        });
});
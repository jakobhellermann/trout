import { solve, setOnSolutions } from "./solver";
import type { Solution } from "./worker";
import "./theme";

let solveBtn = document.getElementById("solveBtn")!;
let outputList = document.getElementById("outputList")!;
let loadingIndicator = document.getElementById("loadingIndicator")!;
let errorMessage = document.getElementById("errorMessage")!;
let statsMessage = document.getElementById("statsMessage")!;

let inputTimeTable = document.getElementById("timeTable") as HTMLTextAreaElement;
let inputNSolutions = document.getElementById("nSolutions") as HTMLInputElement;
let inputMaxRestarts = document.getElementById("maxRestarts") as HTMLInputElement;
let inputOnlyRequiredRestarts = document.getElementById("onlyRequiredRestarts") as HTMLInputElement;
let inputRestartPenalty = document.getElementById("restartPenalty") as HTMLInputElement;

inputTimeTable.addEventListener("keypress", (e) => {
    if (e.key === "Enter" && e.ctrlKey) {
        e.preventDefault();
        solveBtn.click();
    }
});

let setSpinning = (active: boolean) => loadingIndicator.classList.toggle("disabled", !active);

function createSolutionLi(solution: Solution) {
    let timeEl = document.createElement("span");
    timeEl.textContent = `${formatDuration(solution.time * 17, true)} (${solution.time}) with`;
    let routeEl = document.createElement("code");
    routeEl.textContent = solution.route.join(", ");

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
        outputList.insertBefore(child, parent.children[index]);
    }

}

setOnSolutions((solution, updatedIndex) => {
    let nSolutions = Number(inputNSolutions.value);

    let li = createSolutionLi(solution);
    insertChildAt(outputList, li, updatedIndex);
    truncateChildren(outputList, nSolutions);
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
    setSpinning(true);

    outputList.replaceChildren();
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
        })
        .catch((error: Error) => {
            errorMessage.textContent = `Error: ${error.message}`;
            console.error(error);
        })
        .finally(() => {
            setSpinning(false);
        });
});
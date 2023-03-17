import { solve, setOnSolutions } from "./solver";

let BEGINNER_TABLE = `[0,169,60000,60000,60000,60000,60000,60000,284,235,60000,196,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000]
[190,0,257,330,60000,60000,60000,60000,306,255,60000,240,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000]
[190,357,0,183,268,284,60000,60000,267,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,420]
[190,60000,213,0,222,238,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,345]
[190,60000,262,60000,0,98,236,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000]
[190,60000,309,60000,137,0,161,236,238,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000]
[190,60000,60000,60000,272,177,0,220,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000]
[190,60000,60000,60000,60000,251,229,0,312,324,231,60000,60000,190,60000,60000,60000,60000,60000,60000,60000,60000,60000]
[190,60000,306,60000,219,214,60000,269,0,169,181,60000,60000,258,60000,60000,60000,60000,60000,505,60000,60000,60000]
[190,60000,60000,60000,60000,60000,60000,327,199,0,142,60000,60000,301,60000,60000,60000,60000,60000,501,60000,60000,60000]
[190,60000,60000,60000,60000,60000,60000,239,60000,146,0,200,105,215,60000,60000,60000,60000,60000,386,60000,60000,250]
[190,60000,60000,60000,60000,60000,60000,60000,60000,267,138,0,137,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000]
[190,60000,60000,60000,60000,60000,60000,60000,60000,60000,173,188,0,60000,60000,60000,60000,60000,60000,339,60000,60000,218]
[190,60000,60000,60000,60000,60000,308,224,60000,60000,256,60000,60000,0,334,323,375,225,278,60000,60000,60000,60000]
[190,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,417,0,81,271,481,562,60000,60000,60000,60000]
[190,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,381,144,0,234,445,526,60000,60000,60000,60000]
[190,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,401,243,212,0,347,428,60000,60000,60000,60000]
[190,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,479,452,412,0,259,388,374,60000,60000]
[190,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,450,423,386,157,0,289,275,60000,60000]
[190,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,325,333,0,106,289,402]
[190,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,294,302,108,0,209,419]
[190,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,521,529,272,217,0,357]
[0,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,60000,0]
`;


let solveBtn = document.getElementById("solveBtn")!;
let outputList = document.getElementById("outputList")!;
let loadingIndicator = document.getElementById("loadingIndicator")!;
let errorMessage = document.getElementById("errorMessage")!;

let inputTimeTable = document.getElementById("timeTable") as HTMLTextAreaElement;
let inputNSolutions = document.getElementById("nSolutions") as HTMLInputElement;
let inputMaxRestarts = document.getElementById("maxRestarts") as HTMLInputElement;
let inputOnlyRequiredRestarts = document.getElementById("onlyRequiredRestarts") as HTMLInputElement;
let inputRestartPenalty = document.getElementById("restartPenalty") as HTMLInputElement;


inputOnlyRequiredRestarts.addEventListener("change", () => {
    if (!inputOnlyRequiredRestarts.checked && inputMaxRestarts.value === "") {
        inputMaxRestarts.value = "2";
    }
});

inputTimeTable.addEventListener("keypress", (e) => {
    if (e.key === "Enter" && e.ctrlKey) {
        e.preventDefault();
        solveBtn.click();
    }
});

let setSpinning = (active: boolean) => loadingIndicator.classList.toggle("disabled", !active);

setOnSolutions((solutions) => {
    let solutionElements = solutions.map(({ time, route }) => {
        let timeEl = document.createElement("span");
        timeEl.textContent = `47.220 (${time}) with`;
        let routeEl = document.createElement("code");
        routeEl.textContent = route.join(", ");

        let li = document.createElement("li");
        li.replaceChildren(timeEl, " ", routeEl);

        return li;
    });
    outputList.replaceChildren(...solutionElements);
});

solveBtn.addEventListener("click", () => {
    setSpinning(true);

    errorMessage.textContent = "";

    let table = inputTimeTable.value;
    let maxSolutions = Number(inputNSolutions.value);
    let maxRestarts = inputMaxRestarts.value !== "" ? Number(inputMaxRestarts.value) : undefined;
    let onlyRequiredRestarts = inputOnlyRequiredRestarts.checked;
    let restartPenalty = Number(inputRestartPenalty);

    solve({
        table,
        maxSolutions,
        maxRestarts,
        onlyRequiredRestarts,
        restartPenalty,
    })
        .catch((error: Error) => {
            errorMessage.textContent = `Error: ${error.message}`;
            console.error(error);
        })
        .finally(() => {
            setSpinning(false);
        });
});
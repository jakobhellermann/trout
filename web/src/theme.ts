let themeToggleButton = document.getElementById("themeToggleButton")!;
let prefersDarkScheme = window.matchMedia("(prefers-color-scheme: dark)");

let localStorageKeyTheme = "theme";

let darkThemeClass = "dark-theme";
let lightThemeClass = "light-theme";

let currentTheme = localStorage.getItem(localStorageKeyTheme);
if (currentTheme == "dark") {
    document.body.classList.toggle(darkThemeClass);
} else if (currentTheme == "light") {
    document.body.classList.toggle(lightThemeClass);
} else {
    currentTheme = prefersDarkScheme.matches ? "dark" : "light";
}

themeToggleButton.textContent = `Theme: ${currentTheme}`;

themeToggleButton.addEventListener("click", () => {
    let theme: string;
    if (prefersDarkScheme.matches) {
        document.body.classList.toggle(lightThemeClass);
        theme = document.body.classList.contains(lightThemeClass) ? "light" : "dark";
    } else {
        document.body.classList.toggle(darkThemeClass);
        theme = document.body.classList.contains(darkThemeClass) ? "dark" : "light";
    }
    localStorage.setItem(localStorageKeyTheme, theme);
    themeToggleButton.textContent = `Theme: ${theme}`;
});
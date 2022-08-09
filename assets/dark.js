const darkModePlaceholder = document.getElementById("dark-mode-placeholder");
const prefersDarkMode = window.matchMedia("(prefers-color-scheme: dark)");

const sunIcon = `<i class="light-mode-icon">Light</i>`
const moonIcon = `<i class="dark-mode-icon">Dark</i>`

const setIconFromMode = (button, mode) => {
    if (mode === undefined) {
        mode = prefersDarkMode.matches ? "dark" : "light";
    }

    button.innerHTML = (mode === "light") ?
        moonIcon : sunIcon;
};

const toggleMode = () => {
    const dataSet = document.documentElement.dataset;
    const currentMode = dataSet.colorMode;

    if (currentMode === undefined) {
        if (prefersDarkMode.matches) {
            dataSet.colorMode = "light";
        } else {
            dataSet.colorMode = "dark";
        }
        localStorage.setItem("stashed-theme", dataSet.colorMode);
    } else {
        delete dataSet.colorMode;
        localStorage.removeItem("stashed-theme");
    }

    return dataSet.colorMode;
}

const stashedTheme = localStorage.getItem("stashed-theme");
if (stashedTheme) {
    document.documentElement.dataset.colorMode = stashedTheme;
}

// Create a button and add it to the DOM to allow toggling dark mode.

const darkModeButton = document.createElement("a");
darkModeButton.addEventListener("click", () => {
    setIconFromMode(darkModeButton, toggleMode());
});
setIconFromMode(darkModeButton, document.documentElement.dataset.colorMode);

darkModePlaceholder.appendChild(darkModeButton);
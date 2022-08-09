const darkModePlaceholder = document.getElementById("dark-mode-placeholder");
const prefersDarkMode = window.matchMedia("(prefers-color-scheme: dark)");

const rootPath = document.body.dataset['root'];

const sunIcon = `sun`
const moonIcon = `moon`

const setIconFromMode = (button, mode) => {
    if (mode === undefined) {
        mode = prefersDarkMode.matches ? "dark" : "light";
    }

    button.innerHTML = `<svg
    width="24"
    height="24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <use href="${rootPath}/feather-sprite.svg#${(mode === "light") ?
            moonIcon : sunIcon}"/>
  </svg>`
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
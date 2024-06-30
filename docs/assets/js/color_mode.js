// Set theme to the user's preferred color scheme
function updateTheme() {
    const colorMode = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    document.querySelector("html").setAttribute("data-bs-theme", colorMode);
    if (colorMode == "dark") {
        document.querySelector(`link[title="hljs-dark"]`).removeAttribute("disabled");
        document.querySelector(`link[title="hljs-light"]`).setAttribute("disabled", "disabled");
    }
    else {
        document.querySelector(`link[title="hljs-light"]`).removeAttribute("disabled");
        document.querySelector(`link[title="hljs-dark"]`).setAttribute("disabled", "disabled");
    }
}

// Set theme on load
updateTheme()

// Update theme when the preferred scheme changes
window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', updateTheme)
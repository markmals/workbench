// Opens the search dialog and loads Pagefind's UI on first use. Pagefind's files exist only
// in a built site, so a site served without them says how to get search instead.
(() => {
    const dialog = document.querySelector("dialog.search");
    if (!dialog) return;

    const bundle = dialog.dataset.searchBundle;
    let loading = null;

    function load() {
        loading ??= new Promise((resolve, reject) => {
            const stylesheet = document.createElement("link");
            stylesheet.rel = "stylesheet";
            stylesheet.href = `${bundle}pagefind-ui.css`;
            document.head.append(stylesheet);

            const script = document.createElement("script");
            script.src = `${bundle}pagefind-ui.js`;
            script.onload = resolve;
            script.onerror = reject;
            document.head.append(script);
        }).then(
            () =>
                new PagefindUI({
                    element: "#search-ui",
                    baseUrl: dialog.dataset.searchBase,
                    bundlePath: bundle,
                    showImages: false,
                    showSubResults: true,
                    autofocus: true,
                }),
            () => {
                dialog.querySelector(".search__unavailable").hidden = false;
            },
        );
        return loading;
    }

    function open() {
        if (dialog.open) return;
        dialog.showModal();
        load().then(() => dialog.querySelector("input")?.focus());
    }

    for (const button of document.querySelectorAll("[data-search-open]")) {
        button.addEventListener("click", open);
    }

    document.addEventListener("keydown", event => {
        const typing = event.target.closest("input, textarea, select, [contenteditable]");
        const shortcut = (event.key === "k" && (event.metaKey || event.ctrlKey)) || (event.key === "/" && !typing);
        if (!shortcut) return;
        event.preventDefault();
        open();
    });

    // A click on the backdrop lands on the dialog element itself.
    dialog.addEventListener("click", event => {
        if (event.target === dialog) dialog.close();
    });
})();

window.addEventListener("keydown", (event) => {
    // macOS uses native menu accelerators.
    if (window.__ORX_MAC_TITLEBAR__ || !event.ctrlKey || event.metaKey || event.altKey || event.isComposing) return;
    const action = event.key === "=" || event.key === "+" ? "in"
        : event.key === "-" ? "out"
        : event.key === "0" && !event.shiftKey ? "reset" : null;
    if (!action) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    window.ipc.postMessage(`page-zoom:${action}`);
}, true);

// Restore titlebar spacing as soon as the new document is ready after a reload.
document.addEventListener("DOMContentLoaded", () => {
    window.ipc.postMessage("page-zoom:sync");
}, { once: true });

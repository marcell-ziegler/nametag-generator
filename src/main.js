import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";

const state = {
    csvPath: null, // string | null
    resourcePaths: [], // string[]
    previewUrl: null, // string | null (blob:)
};

function basename(path) {
    // Windows or Unix separators
    return path.split(/[/\\]/).pop();
}

function setBusy(isBusy) {
    document.getElementById("spinner").classList.toggle("hidden", !isBusy);

    document.getElementById("btnPickCsv").disabled = isBusy;
    document.getElementById("btnAddResources").disabled = isBusy;
    document.getElementById("btnClearResources").disabled = isBusy;
    document.getElementById("btnPreview").disabled = isBusy;
    document.getElementById("btnExport").disabled = isBusy;

    document.getElementById("templateEditor").readOnly = isBusy;
    document.getElementById("nodkontakt").readOnly = isBusy;
}

function setStatus(text, kind) {
    const el = document.getElementById("status");
    el.classList.remove("hidden", "error", "ok");
    el.textContent = text;
    if (kind === "error") el.classList.add("error");
    if (kind === "ok") el.classList.add("ok");
}

function clearStatus() {
    const el = document.getElementById("status");
    el.classList.add("hidden");
    el.textContent = "";
    el.classList.remove("error", "ok");
}

function renderResources() {
    const ul = document.getElementById("resourcesList");
    ul.innerHTML = "";

    for (const path of state.resourcePaths) {
        const li = document.createElement("li");

        const left = document.createElement("span");
        left.textContent = basename(path);

        const btn = document.createElement("button");
        btn.type = "button";
        btn.textContent = "Remove";
        btn.className = "remove";
        btn.addEventListener("click", () => {
            state.resourcePaths = state.resourcePaths.filter((p) => p !== path);
            renderResources();
        });

        li.appendChild(left);
        li.appendChild(btn);
        ul.appendChild(li);
    }
}

function normalizeBytesToUint8Array(value) {
    // Tauri may give Vec<u8> as number[] or Uint8Array depending on versions/bindings.
    if (value instanceof Uint8Array) return value;
    if (Array.isArray(value)) return new Uint8Array(value);

    // Some bindings might return an object with `data`:
    if (value && typeof value === "object" && Array.isArray(value.data)) {
        return new Uint8Array(value.data);
    }

    throw new Error("Unexpected byte array type returned from backend.");
}

function setPdfPreviewFromBytes(bytesValue) {
    const bytes = normalizeBytesToUint8Array(bytesValue);
    const blob = new Blob([bytes], { type: "application/pdf" });
    const url = URL.createObjectURL(blob);

    // Cleanup old preview URL
    if (state.previewUrl) {
        URL.revokeObjectURL(state.previewUrl);
    }
    state.previewUrl = url;

    document.getElementById("previewFrame").src = url;
}

// --- Event handlers ---

async function pickCsv() {
    const path = await open({
        multiple: false,
        filters: [{ name: "CSV", extensions: ["csv"] }],
    });

    if (!path) return;

    // `open` might return string or string[]
    const chosen = Array.isArray(path) ? path[0] : path;

    state.csvPath = chosen;
    document.getElementById("csvPath").textContent = chosen;
}

async function addResources() {
    const paths = await open({
        multiple: true,
        filters: [
            { name: "Resources", extensions: ["png", "jpg", "jpeg", "svg"] },
        ],
    });

    if (!paths) return;

    const chosen = Array.isArray(paths) ? paths : [paths];

    // Merge and dedupe
    const set = new Set(state.resourcePaths);
    for (const p of chosen) set.add(p);
    state.resourcePaths = Array.from(set);

    renderResources();
}

function clearResources() {
    state.resourcePaths = [];
    renderResources();
}

function validateInputs() {
    const template = document.getElementById("templateEditor").value.trim();
    const nodkontakt = document.getElementById("nodkontakt").value.trim();

    if (!state.csvPath) {
        return { ok: false, msg: "Please choose a CSV file first." };
    }
    if (!template) {
        return { ok: false, msg: "Template is empty." };
    }
    // nodkontakt can be empty if you want; enforce if you prefer:
    // if (!nodkontakt) return { ok: false, msg: "Nödkontakt is empty." };

    return { ok: true, template, nodkontakt };
}

async function preview() {
    clearStatus();
    const v = validateInputs();
    if (!v.ok) {
        setStatus(v.msg, "error");
        return;
    }

    setBusy(true);
    try {
        const pdfBytes = await invoke("compile_to_bytes", {
            template: v.template,
            csvPathStr: state.csvPath,
            resourcePathStrs: state.resourcePaths,
            nodkontakt: v.nodkontakt,
        });

        setPdfPreviewFromBytes(pdfBytes);
        setStatus("Preview updated.", "ok");
    } catch (err) {
        setStatus(String(err), "error");
    } finally {
        setBusy(false);
    }
}

async function exportPdf() {
    clearStatus();
    const v = validateInputs();
    if (!v.ok) {
        setStatus(v.msg, "error");
        return;
    }

    const outputPath = await save({
        defaultPath: "nametags.pdf",
        filters: [{ name: "PDF", extensions: ["pdf"] }],
    });

    if (!outputPath) return;

    setBusy(true);
    try {
        await invoke("export_pdf", {
            template: v.template,
            csvPathStr: state.csvPath,
            resourcePathStrs: state.resourcePaths,
            nodkontakt: v.nodkontakt,
            outputPathStr: outputPath,
        });

        setStatus(`Exported PDF to:\n${outputPath}`, "ok");
    } catch (err) {
        setStatus(String(err), "error");
    } finally {
        setBusy(false);
    }
}

window.addEventListener("DOMContentLoaded", () => {
    // Wire buttons
    document.getElementById("btnPickCsv").addEventListener("click", pickCsv);
    document
        .getElementById("btnAddResources")
        .addEventListener("click", addResources);
    document
        .getElementById("btnClearResources")
        .addEventListener("click", clearResources);
    document.getElementById("btnPreview").addEventListener("click", preview);
    document.getElementById("btnExport").addEventListener("click", exportPdf);

    renderResources();

    // Optional: preload your current template for convenience
    // document.getElementById("templateEditor").value = "...";
});

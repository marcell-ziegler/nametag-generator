import { readText } from "@tauri-apps/plugin-clipboard-manager";

function isEditableTextTarget(el) {
  if (!el) return false;

  // textarea
  if (el instanceof HTMLTextAreaElement) return !el.readOnly && !el.disabled;

  // input types that accept text
  if (el instanceof HTMLInputElement) {
    const t = (el.type || "").toLowerCase();
    const textLike =
      t === "text" ||
      t === "search" ||
      t === "url" ||
      t === "tel" ||
      t === "email" ||
      t === "password" ||
      t === "number";
    return textLike && !el.readOnly && !el.disabled;
  }

  // contenteditable divs (e.g. future CodeMirror wrappers)
  if (el instanceof HTMLElement && el.isContentEditable) return true;

  return false;
}

function insertAtSelection(el, text) {
  // For <textarea> and <input>
  if (el instanceof HTMLTextAreaElement || el instanceof HTMLInputElement) {
    const start = el.selectionStart ?? el.value.length;
    const end = el.selectionEnd ?? el.value.length;

    const before = el.value.slice(0, start);
    const after = el.value.slice(end);

    el.value = before + text + after;

    const newPos = start + text.length;
    el.setSelectionRange(newPos, newPos);

    // Trigger input event so your state (if any) updates
    el.dispatchEvent(new Event("input", { bubbles: true }));
    return;
  }

  // For contenteditable, do a simple insertion (good enough for MVP)
  if (el instanceof HTMLElement && el.isContentEditable) {
    document.execCommand("insertText", false, text);
  }
}

export function installCtrlVPasteHandler() {
  window.addEventListener(
    "keydown",
    async (e) => {
      const isMac = navigator.platform.toLowerCase().includes("mac");
      const isPaste =
        (isMac ? e.metaKey : e.ctrlKey) && (e.key === "v" || e.key === "V");

      if (!isPaste) return;

      const active = document.activeElement;
      if (!isEditableTextTarget(active)) return;

      // Prevent default paste (avoids double paste on platforms where native paste works)
      e.preventDefault();
      e.stopPropagation();

      try {
        const clip = await readText();
        if (!clip) return;
        insertAtSelection(active, clip);
      } catch (err) {
        // If clipboard read is denied or fails, fall back:
        // - either do nothing (native paste was prevented),
        // - or you can allow default by removing preventDefault above.
        console.error("Clipboard read failed:", err);
      }
    },
    true // capture helps intercept before other handlers
  );
}
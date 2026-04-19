// CodeMirror 6 bridge for Rust/WASM interop
// Bundled into public/app/codemirror.bundle.js via esbuild

import { EditorView, keymap, lineNumbers, highlightActiveLineGutter, highlightSpecialChars, drawSelection, dropCursor, rectangularSelection, crosshairCursor, highlightActiveLine } from "@codemirror/view";
import { EditorState, StateEffect, Compartment } from "@codemirror/state";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { syntaxHighlighting, defaultHighlightStyle, indentOnInput, bracketMatching, foldGutter, foldKeymap } from "@codemirror/language";
import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
import { autocompletion, completionKeymap, closeBrackets, closeBracketsKeymap } from "@codemirror/autocomplete";
import { lintKeymap } from "@codemirror/lint";

// Language imports
import { javascript } from "@codemirror/lang-javascript";
import { html } from "@codemirror/lang-html";
import { css } from "@codemirror/lang-css";
import { json } from "@codemirror/lang-json";
import { markdown } from "@codemirror/lang-markdown";
import { python } from "@codemirror/lang-python";
import { rust } from "@codemirror/lang-rust";
import { cpp } from "@codemirror/lang-cpp";
import { java } from "@codemirror/lang-java";
import { xml } from "@codemirror/lang-xml";
import { sql } from "@codemirror/lang-sql";
import { yaml } from "@codemirror/lang-yaml";

// Theme
import { oneDark } from "@codemirror/theme-one-dark";

const instances = new Map();

function syncEditorLayout(inst) {
  if (!inst?.container || !inst?.view) return;

  const rect = inst.container.getBoundingClientRect();
  const height = Math.max(0, Math.floor(rect.height));
  const width = Math.max(0, Math.floor(rect.width));

  // WebKit may not treat flex-distributed height as a valid basis for
  // percentage heights inside CodeMirror. Push concrete pixel sizes from JS.
  if (height > 0) {
    inst.view.dom.style.height = `${height}px`;
  }
  if (width > 0) {
    inst.view.dom.style.width = `${width}px`;
  }

  inst.view.requestMeasure();
}

/**
 * Get CodeMirror language extension by name.
 */
function getLanguageExtension(langName) {
  switch (langName) {
    case "javascript": return javascript();
    case "jsx": return javascript({ jsx: true });
    case "typescript": return javascript({ typescript: true });
    case "tsx": return javascript({ typescript: true, jsx: true });
    case "html": return html();
    case "css": return css();
    case "json": return json();
    case "markdown": return markdown();
    case "python": return python();
    case "rust": return rust();
    case "cpp": return cpp();
    case "java": return java();
    case "xml": return xml();
    case "sql": return sql();
    case "yaml": return yaml();
    default: return null;
  }
}

/**
 * Map file extension to language name.
 */
function extToLang(ext) {
  const map = {
    "js": "javascript", "mjs": "javascript", "cjs": "javascript",
    "jsx": "jsx",
    "ts": "typescript", "mts": "typescript",
    "tsx": "tsx",
    "html": "html", "htm": "html", "svelte": "html", "vue": "html",
    "css": "css", "scss": "css", "less": "css",
    "json": "json", "jsonc": "json",
    "md": "markdown", "mdx": "markdown",
    "py": "python", "pyw": "python",
    "rs": "rust",
    "c": "cpp", "h": "cpp", "cpp": "cpp", "hpp": "cpp", "cc": "cpp", "cxx": "cpp",
    "java": "java", "kt": "java", "kts": "java",
    "xml": "xml", "svg": "xml", "plist": "xml",
    "sql": "sql",
    "yaml": "yaml", "yml": "yaml",
  };
  return map[ext] || null;
}

/**
 * Build a light theme from CSS custom properties.
 */
function buildLightTheme() {
  const style = getComputedStyle(document.documentElement);
  const get = (prop) => {
    const val = style.getPropertyValue(prop).trim();
    return val || undefined;
  };

  const bg = get("--background") || "#ffffff";
  const fg = get("--foreground") || "#1a1a1a";
  const muted = get("--muted") || "#f5f5f5";
  const border = get("--border") || "#e5e5e5";
  const accent = get("--accent") || "#f0f0f0";
  const ring = get("--ring") || "#3b82f6";

  return EditorView.theme({
    "&": {
      backgroundColor: bg,
      color: fg,
    },
    ".cm-content": {
      caretColor: fg,
    },
    ".cm-cursor, .cm-dropCursor": {
      borderLeftColor: fg,
    },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection": {
      backgroundColor: accent,
    },
    ".cm-gutters": {
      backgroundColor: bg,
      color: get("--muted-foreground") || "#999",
      borderRight: `1px solid ${border}`,
    },
    ".cm-activeLineGutter": {
      backgroundColor: muted,
    },
    ".cm-activeLine": {
      backgroundColor: muted,
    },
    ".cm-foldPlaceholder": {
      backgroundColor: accent,
      border: "none",
    },
    "&.cm-focused": {
      outline: "none",
    },
  }, { dark: false });
}

/**
 * Get base extensions shared by all editor instances.
 */
function getBaseExtensions() {
  return [
    lineNumbers(),
    highlightActiveLineGutter(),
    highlightSpecialChars(),
    history(),
    foldGutter(),
    drawSelection(),
    dropCursor(),
    EditorState.allowMultipleSelections.of(true),
    indentOnInput(),
    syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
    bracketMatching(),
    closeBrackets(),
    autocompletion(),
    rectangularSelection(),
    crosshairCursor(),
    highlightActiveLine(),
    highlightSelectionMatches(),
    keymap.of([
      ...closeBracketsKeymap,
      ...defaultKeymap,
      ...searchKeymap,
      ...historyKeymap,
      ...foldKeymap,
      ...completionKeymap,
      ...lintKeymap,
      indentWithTab,
    ]),
    EditorView.lineWrapping,
  ];
}

window.CodeMirrorBridge = {
  /**
   * Create a new CodeMirror editor instance.
   * @param {string} id - Unique identifier
   * @param {HTMLElement} container - DOM element to mount into
   * @param {object} opts - { value, language, readOnly, fontSize, fontFamily }
   * @returns {boolean} true on success
   */
  create(id, container, opts = {}) {
    if (instances.has(id)) {
      this.dispose(id);
    }

    const langCompartment = new Compartment();
    const themeCompartment = new Compartment();
    const readOnlyCompartment = new Compartment();

    const isDark = document.documentElement.classList.contains("dark");

    // Determine language
    let langExt = null;
    if (opts.language) {
      langExt = getLanguageExtension(opts.language);
    }

    // Mutable callback holders — set later via onChange()/onSave()
    const callbackHolder = { onChange: null, onSave: null, suppressChange: false };

    const extensions = [
      ...getBaseExtensions(),
      langCompartment.of(langExt ? [langExt] : []),
      themeCompartment.of(isDark ? [oneDark] : [buildLightTheme()]),
      readOnlyCompartment.of(EditorState.readOnly.of(opts.readOnly || false)),
      EditorView.theme({
        "&": {
          fontSize: (opts.fontSize || 13) + "px",
          fontFamily: opts.fontFamily || "'JetBrains Mono', 'Geist Mono', 'Cascadia Code', Menlo, monospace",
          height: "100%",
        },
        ".cm-scroller": {
          overflow: "auto",
        },
      }),
      // onChange listener — registered at creation, callback set later
      EditorView.updateListener.of((update) => {
        if (update.docChanged && callbackHolder.onChange && !callbackHolder.suppressChange) {
          callbackHolder.onChange(update.state.doc.toString());
        }
      }),
      // Cmd/Ctrl+S save keybinding — registered at creation, callback set later
      keymap.of([{
        key: "Mod-s",
        run: (view) => {
          if (callbackHolder.onSave) {
            callbackHolder.onSave(view.state.doc.toString());
          }
          return true;
        },
      }]),
    ];

    const state = EditorState.create({
      doc: opts.value || "",
      extensions,
    });

    const view = new EditorView({
      state,
      parent: container,
    });

    let resizeObserver = null;
    const syncLayout = () => syncEditorLayout({ container, view });

    if (typeof ResizeObserver !== "undefined") {
      resizeObserver = new ResizeObserver(() => {
        syncLayout();
      });
      resizeObserver.observe(container);
    }

    // Run after mount and after the next frame so late layout settles.
    syncLayout();
    requestAnimationFrame(() => {
      syncLayout();
    });

    instances.set(id, {
      view,
      container,
      langCompartment,
      themeCompartment,
      readOnlyCompartment,
      callbackHolder,
      resizeObserver,
      syncLayout,
    });

    return true;
  },

  /**
   * Set the entire document content.
   * @param {string} id
   * @param {string} content
   */
  setValue(id, content) {
    const inst = instances.get(id);
    if (!inst) return;
    const { view, callbackHolder } = inst;
    if (view.state.doc.toString() === content) return;
    callbackHolder.suppressChange = true;
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: content },
    });
    callbackHolder.suppressChange = false;
    inst.syncLayout();
  },

  /**
   * Get the current document content.
   * @param {string} id
   * @returns {string}
   */
  getValue(id) {
    const inst = instances.get(id);
    if (!inst) return "";
    return inst.view.state.doc.toString();
  },

  /**
   * Switch the language extension.
   * @param {string} id
   * @param {string} langName - Language name (e.g., "rust", "javascript")
   */
  setLanguage(id, langName) {
    const inst = instances.get(id);
    if (!inst) return;
    const ext = getLanguageExtension(langName);
    inst.view.dispatch({
      effects: inst.langCompartment.reconfigure(ext ? [ext] : []),
    });
  },

  /**
   * Register a callback for document changes.
   * Callback holder was created during create() — just set the reference.
   * @param {string} id
   * @param {function} callback - Called with new document content string
   */
  onChange(id, callback) {
    const inst = instances.get(id);
    if (!inst) return;
    inst.callbackHolder.onChange = callback;
  },

  /**
   * Register a Cmd/Ctrl+S save callback.
   * Callback holder was created during create() — just set the reference.
   * @param {string} id
   * @param {function} callback - Called with current content string
   */
  onSave(id, callback) {
    const inst = instances.get(id);
    if (!inst) return;
    inst.callbackHolder.onSave = callback;
  },

  /**
   * Dispose of an editor instance.
   * @param {string} id
   */
  dispose(id) {
    const inst = instances.get(id);
    if (inst) {
      inst.resizeObserver?.disconnect();
      inst.view.destroy();
      instances.delete(id);
    }
  },

  /**
   * Update theme (call when light/dark mode changes).
   * @param {string} id - Instance id (if null, updates all)
   */
  setTheme(id) {
    const isDark = document.documentElement.classList.contains("dark");
    const newTheme = isDark ? [oneDark] : [buildLightTheme()];

    if (id) {
      const inst = instances.get(id);
      if (inst) {
        inst.view.dispatch({
          effects: inst.themeCompartment.reconfigure(newTheme),
        });
      }
    } else {
      for (const inst of instances.values()) {
        inst.view.dispatch({
          effects: inst.themeCompartment.reconfigure(newTheme),
        });
      }
    }
  },

  /**
   * Toggle readOnly state.
   * @param {string} id
   * @param {boolean} readOnly
   */
  setReadOnly(id, readOnly) {
    const inst = instances.get(id);
    if (!inst) return;
    inst.view.dispatch({
      effects: inst.readOnlyCompartment.reconfigure(EditorState.readOnly.of(readOnly)),
    });
  },

  /**
   * Focus the editor.
   * @param {string} id
   */
  focus(id) {
    const inst = instances.get(id);
    if (inst) {
      inst.view.focus();
    }
  },

  /**
   * Check if an instance exists.
   * @param {string} id
   * @returns {boolean}
   */
  has(id) {
    return instances.has(id);
  },

  syncLayout(id) {
    const inst = instances.get(id);
    if (inst) {
      inst.syncLayout();
    }
  },
};

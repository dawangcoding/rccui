// xterm.js bridge for Rust/WASM interop
// Bundled into public/app/xterm.bundle.js via esbuild

import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebLinksAddon } from "@xterm/addon-web-links";

const instances = new Map();

window.XtermBridge = {
  /**
   * Create a new terminal instance inside the given container element.
   * @param {string} id - Unique identifier for this terminal instance
   * @param {HTMLElement} container - DOM element to mount the terminal into
   * @param {object} opts - Optional configuration
   * @returns {boolean} true on success
   */
  create(id, container, opts = {}) {
    if (instances.has(id)) {
      this.dispose(id);
    }

    const term = new Terminal({
      cursorBlink: true,
      cursorStyle: "bar",
      fontSize: opts.fontSize || 13,
      fontFamily:
        opts.fontFamily ||
        "'JetBrains Mono', 'Geist Mono', 'Cascadia Code', Menlo, monospace",
      lineHeight: 1.35,
      scrollback: opts.scrollback || 5000,
      allowProposedApi: true,
      theme: getTheme(),
      convertEol: false,
      allowTransparency: true,
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);

    const webLinksAddon = new WebLinksAddon();
    term.loadAddon(webLinksAddon);

    term.open(container);

    // Initial fit
    try {
      fitAddon.fit();
    } catch (_) {
      // Container may not have dimensions yet
    }

    instances.set(id, { term, fitAddon, container });
    return true;
  },

  /**
   * Write data to a terminal.
   * @param {string} id - Terminal instance id
   * @param {string} data - Data to write (may include ANSI escape sequences)
   */
  write(id, data) {
    const inst = instances.get(id);
    if (inst) {
      inst.term.write(data);
    }
  },

  /**
   * Register a callback for user input (keystrokes).
   * @param {string} id - Terminal instance id
   * @param {function} callback - Called with the input string
   */
  onData(id, callback) {
    const inst = instances.get(id);
    if (inst) {
      inst.term.onData(callback);
    }
  },

  /**
   * Register a callback for terminal resize events.
   * @param {string} id - Terminal instance id
   * @param {function} callback - Called with { cols, rows }
   */
  onResize(id, callback) {
    const inst = instances.get(id);
    if (inst) {
      inst.term.onResize(callback);
    }
  },

  /**
   * Fit the terminal to its container and return the new dimensions.
   * @param {string} id - Terminal instance id
   * @returns {{ cols: number, rows: number } | null}
   */
  fit(id) {
    const inst = instances.get(id);
    if (!inst) return null;
    try {
      inst.fitAddon.fit();
      return { cols: inst.term.cols, rows: inst.term.rows };
    } catch (_) {
      return null;
    }
  },

  /**
   * Get the current terminal dimensions.
   * @param {string} id - Terminal instance id
   * @returns {{ cols: number, rows: number } | null}
   */
  getDimensions(id) {
    const inst = instances.get(id);
    if (!inst) return null;
    return { cols: inst.term.cols, rows: inst.term.rows };
  },

  /**
   * Clear the terminal screen.
   * @param {string} id - Terminal instance id
   */
  clear(id) {
    const inst = instances.get(id);
    if (inst) {
      inst.term.clear();
    }
  },

  /**
   * Focus the terminal.
   * @param {string} id - Terminal instance id
   */
  focus(id) {
    const inst = instances.get(id);
    if (inst) {
      inst.term.focus();
    }
  },

  /**
   * Dispose of a terminal instance and free resources.
   * @param {string} id - Terminal instance id
   */
  dispose(id) {
    const inst = instances.get(id);
    if (inst) {
      inst.term.dispose();
      instances.delete(id);
    }
  },

  /**
   * Update terminal theme (call when light/dark mode changes).
   * @param {string} id - Terminal instance id (if null, updates all)
   */
  updateTheme(id) {
    const theme = getTheme();
    if (id) {
      const inst = instances.get(id);
      if (inst) inst.term.options.theme = theme;
    } else {
      for (const inst of instances.values()) {
        inst.term.options.theme = theme;
      }
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
};

/**
 * Build xterm.js theme from CSS custom properties.
 * This reads the semantic color tokens from the current theme.
 */
function getTheme() {
  const style = getComputedStyle(document.documentElement);
  const get = (prop) => {
    const val = style.getPropertyValue(prop).trim();
    return val ? `oklch(${val})` : undefined;
  };

  // Check if dark mode is active
  const isDark = document.documentElement.classList.contains("dark");

  return {
    background: get("--background") || (isDark ? "#1a1a1a" : "#ffffff"),
    foreground: get("--foreground") || (isDark ? "#e0e0e0" : "#1a1a1a"),
    cursor: get("--foreground") || (isDark ? "#e0e0e0" : "#1a1a1a"),
    cursorAccent: get("--background") || (isDark ? "#1a1a1a" : "#ffffff"),
    selectionBackground: isDark
      ? "rgba(255, 255, 255, 0.15)"
      : "rgba(0, 0, 0, 0.15)",
    selectionForeground: undefined,
    // ANSI colors — use sensible defaults
    black: isDark ? "#1a1a1a" : "#000000",
    red: "#e55561",
    green: "#8ebd6b",
    yellow: "#e2b86b",
    blue: "#4fa6ed",
    magenta: "#bf68d9",
    cyan: "#48b0bd",
    white: isDark ? "#abb2bf" : "#383a42",
    brightBlack: isDark ? "#5c6370" : "#a0a1a7",
    brightRed: "#e06c75",
    brightGreen: "#98c379",
    brightYellow: "#e5c07b",
    brightBlue: "#61afef",
    brightMagenta: "#c678dd",
    brightCyan: "#56b6c2",
    brightWhite: isDark ? "#ffffff" : "#1a1a1a",
  };
}

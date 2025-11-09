import { app, BrowserWindow, ipcMain } from "electron";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import pty from "node-pty";
import os from "os";

// Import your DB API
import * as db from "./db";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

let win: BrowserWindow | null = null;

function resolveRendererHtml() {
  const appRoot = path.join(__dirname, "..");
  const candidates = [
    path.join(appRoot, "renderer/index.html"),
    path.join(appRoot, "dist/index.html"),
    path.join(appRoot, "index.html"),
  ];

  const existing = candidates.find((candidate) => fs.existsSync(candidate));

  if (!existing) {
    throw new Error("Renderer entry point not found. Did you run `vite build`?");
  }

  return existing;
}

async function createWindow() {
  win = new BrowserWindow({
    width: 1200,
    height: 800,
    webPreferences: {
      preload: path.join(__dirname, "preload.mjs"),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  // Load renderer
  const rendererUrl = process.env["ELECTRON_RENDERER_URL"];

  if (!app.isPackaged && rendererUrl) {
    await win.loadURL(rendererUrl);
  } else {
    const htmlPath = resolveRendererHtml();
    await win.loadFile(htmlPath);
  }

  try {
    win.webContents.openDevTools();
  } catch {}

  win.on("closed", () => (win = null));

  //
  // ✅ PTY SETUP MOVED HERE
  //
  const shell = os.platform() === "win32" ? "powershell.exe" : "bash";

  const ptyProcess = pty.spawn(shell, [], {
    name: "xterm-color",
    cols: 80,
    rows: 24,
    cwd: process.env.HOME,
    env: process.env,
  });

  // PTY → Renderer
  ptyProcess.onData((data) => {
    win?.webContents.send("terminal-data", data);
  });

  // Renderer → PTY
  ipcMain.on("terminal-write", (_, data) => {
    ptyProcess.write(data);
  });
}


// App ready
app.whenReady().then(() => {
  createWindow();

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

// Quit on all windows closed
app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

//
// ✅ IPC HANDLERS
//
ipcMain.handle("db:connect", async (_, config) => {
  try {
    return await db.connect(config);
  } catch (err: any) {
    return { error: err.message };
  }
});

ipcMain.handle("db:query", async (_, sql: string) => {
  try {
    return await db.query(sql);
  } catch (err: any) {
    return { error: err.message };
  }
});

ipcMain.handle("db:disconnect", async () => {
  try {
    await db.disconnect();
    return { success: true };
  } catch (err: any) {
    return { error: err.message };
  }
});

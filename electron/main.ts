import { app, BrowserWindow, ipcMain } from "electron";
import path from "node:path";

// Import your DB API
import * as db from "./db";

let win: BrowserWindow | null = null;

async function createWindow() {
  win = new BrowserWindow({
    width: 1200,
    height: 800,
    webPreferences: {
      preload: path.join(__dirname, "preload.js"),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  // Load your Vite-compiled UI or dev server
  if (app.isPackaged) {
    win.loadFile(path.join(__dirname, "../renderer/index.html"));
  } else {
    win.loadURL(process.env["ELECTRON_RENDERER_URL"]!);
  }

  win.on("closed", () => {
    win = null;
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

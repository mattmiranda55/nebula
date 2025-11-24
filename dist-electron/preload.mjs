"use strict";
const electron = require("electron");
electron.contextBridge.exposeInMainWorld("ipcRenderer", {
  on(...args) {
    const [channel, listener] = args;
    return electron.ipcRenderer.on(channel, (event, ...args2) => listener(event, ...args2));
  },
  off(...args) {
    const [channel, ...omit] = args;
    return electron.ipcRenderer.off(channel, ...omit);
  },
  send(...args) {
    const [channel, ...omit] = args;
    return electron.ipcRenderer.send(channel, ...omit);
  },
  invoke(...args) {
    const [channel, ...omit] = args;
    return electron.ipcRenderer.invoke(channel, ...omit);
  }
  // You can expose other APTs you need here.
  // ...
});
electron.contextBridge.exposeInMainWorld("db", {
  connect: (config) => electron.ipcRenderer.invoke("db:connect", config),
  query: (sql) => electron.ipcRenderer.invoke("db:query", sql),
  disconnect: () => electron.ipcRenderer.invoke("db:disconnect")
});
electron.contextBridge.exposeInMainWorld("terminal", {
  write: (data) => electron.ipcRenderer.send("terminal-write", data),
  onData: (callback) => electron.ipcRenderer.on("terminal-data", (_, data) => callback(data))
});

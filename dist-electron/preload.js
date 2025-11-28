"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const electron_1 = require("electron");
// --------- Expose some API to the Renderer process ---------
electron_1.contextBridge.exposeInMainWorld('ipcRenderer', {
    on(...args) {
        const [channel, listener] = args;
        return electron_1.ipcRenderer.on(channel, (event, ...args) => listener(event, ...args));
    },
    off(...args) {
        const [channel, ...omit] = args;
        return electron_1.ipcRenderer.off(channel, ...omit);
    },
    send(...args) {
        const [channel, ...omit] = args;
        return electron_1.ipcRenderer.send(channel, ...omit);
    },
    invoke(...args) {
        const [channel, ...omit] = args;
        return electron_1.ipcRenderer.invoke(channel, ...omit);
    },
    // You can expose other APTs you need here.
    // ...
});
electron_1.contextBridge.exposeInMainWorld("db", {
    connect: (config) => electron_1.ipcRenderer.invoke("db:connect", config),
    query: (sql) => electron_1.ipcRenderer.invoke("db:query", sql),
    structure: () => electron_1.ipcRenderer.invoke("db:structure"),
    disconnect: () => electron_1.ipcRenderer.invoke("db:disconnect"),
});
electron_1.contextBridge.exposeInMainWorld("terminal", {
    write: (data) => electron_1.ipcRenderer.send("terminal-write", data),
    onData: (callback) => electron_1.ipcRenderer.on("terminal-data", (_, data) => callback(data))
});
//# sourceMappingURL=preload.js.map
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const electron_1 = require("electron");
// Expose IPC methods to the renderer process in a controlled way.
electron_1.contextBridge.exposeInMainWorld("ipcRenderer", {
    on(...args) {
        const [channel, listener] = args;
        return electron_1.ipcRenderer.on(channel, (event, ...listenerArgs) => listener(event, ...listenerArgs));
    },
    off(...args) {
        const [channel, ...listenerArgs] = args;
        return electron_1.ipcRenderer.off(channel, ...listenerArgs);
    },
    send(...args) {
        const [channel, ...payload] = args;
        return electron_1.ipcRenderer.send(channel, ...payload);
    },
    invoke(...args) {
        const [channel, ...payload] = args;
        return electron_1.ipcRenderer.invoke(channel, ...payload);
    },
});
electron_1.contextBridge.exposeInMainWorld("db", {
    connect: (config) => electron_1.ipcRenderer.invoke("db:connect", config),
    query: (sql) => electron_1.ipcRenderer.invoke("db:query", sql),
    structure: () => electron_1.ipcRenderer.invoke("db:structure"),
    disconnect: () => electron_1.ipcRenderer.invoke("db:disconnect"),
});
electron_1.contextBridge.exposeInMainWorld("config", {
    saveConnection: (config) => electron_1.ipcRenderer.invoke("config:save-connection", config),
    listConnections: () => electron_1.ipcRenderer.invoke("config:list-connections"),
    getLastConnection: () => electron_1.ipcRenderer.invoke("config:get-last-connection"),
    connectSaved: (id) => electron_1.ipcRenderer.invoke("config:connect-saved", id),
});
electron_1.contextBridge.exposeInMainWorld("terminal", {
    write: (data) => electron_1.ipcRenderer.send("terminal-write", data),
    onData: (callback) => electron_1.ipcRenderer.on("terminal-data", (_, data) => callback(data)),
});
//# sourceMappingURL=preload.cjs.map
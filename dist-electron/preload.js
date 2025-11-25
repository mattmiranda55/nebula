import { ipcRenderer, contextBridge } from 'electron';
// --------- Expose some API to the Renderer process ---------
contextBridge.exposeInMainWorld('ipcRenderer', {
    on(...args) {
        const [channel, listener] = args;
        return ipcRenderer.on(channel, (event, ...args) => listener(event, ...args));
    },
    off(...args) {
        const [channel, ...omit] = args;
        return ipcRenderer.off(channel, ...omit);
    },
    send(...args) {
        const [channel, ...omit] = args;
        return ipcRenderer.send(channel, ...omit);
    },
    invoke(...args) {
        const [channel, ...omit] = args;
        return ipcRenderer.invoke(channel, ...omit);
    },
    // You can expose other APTs you need here.
    // ...
});
contextBridge.exposeInMainWorld("db", {
    connect: (config) => ipcRenderer.invoke("db:connect", config),
    query: (sql) => ipcRenderer.invoke("db:query", sql),
    structure: () => ipcRenderer.invoke("db:structure"),
    disconnect: () => ipcRenderer.invoke("db:disconnect"),
});
contextBridge.exposeInMainWorld("terminal", {
    write: (data) => ipcRenderer.send("terminal-write", data),
    onData: (callback) => ipcRenderer.on("terminal-data", (_, data) => callback(data))
});
//# sourceMappingURL=preload.js.map
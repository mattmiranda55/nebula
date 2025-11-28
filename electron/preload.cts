import { ipcRenderer, contextBridge } from "electron";

// Expose IPC methods to the renderer process in a controlled way.
contextBridge.exposeInMainWorld("ipcRenderer", {
  on(...args: Parameters<typeof ipcRenderer.on>) {
    const [channel, listener] = args;
    return ipcRenderer.on(channel, (event, ...listenerArgs) => listener(event, ...listenerArgs));
  },
  off(...args: Parameters<typeof ipcRenderer.off>) {
    const [channel, ...listenerArgs] = args;
    return ipcRenderer.off(channel, ...listenerArgs);
  },
  send(...args: Parameters<typeof ipcRenderer.send>) {
    const [channel, ...payload] = args;
    return ipcRenderer.send(channel, ...payload);
  },
  invoke(...args: Parameters<typeof ipcRenderer.invoke>) {
    const [channel, ...payload] = args;
    return ipcRenderer.invoke(channel, ...payload);
  },
});

contextBridge.exposeInMainWorld("db", {
  connect: (config: any) => ipcRenderer.invoke("db:connect", config),
  query: (sql: string) => ipcRenderer.invoke("db:query", sql),
  structure: () => ipcRenderer.invoke("db:structure"),
  disconnect: () => ipcRenderer.invoke("db:disconnect"),
});

contextBridge.exposeInMainWorld("config", {
  saveConnection: (config: any) => ipcRenderer.invoke("config:save-connection", config),
  listConnections: () => ipcRenderer.invoke("config:list-connections"),
  getLastConnection: () => ipcRenderer.invoke("config:get-last-connection"),
  connectSaved: (id: string) => ipcRenderer.invoke("config:connect-saved", id),
});

contextBridge.exposeInMainWorld("terminal", {
  write: (data: any) => ipcRenderer.send("terminal-write", data),
  onData: (callback: (data: unknown) => void) =>
    ipcRenderer.on("terminal-data", (_, data) => callback(data)),
});

import { ipcRenderer, contextBridge } from 'electron'

// --------- Expose some API to the Renderer process ---------
contextBridge.exposeInMainWorld('ipcRenderer', {
  on(...args: Parameters<typeof ipcRenderer.on>) {
    const [channel, listener] = args
    return ipcRenderer.on(channel, (event, ...args) => listener(event, ...args))
  },
  off(...args: Parameters<typeof ipcRenderer.off>) {
    const [channel, ...omit] = args
    return ipcRenderer.off(channel, ...omit)
  },
  send(...args: Parameters<typeof ipcRenderer.send>) {
    const [channel, ...omit] = args
    return ipcRenderer.send(channel, ...omit)
  },
  invoke(...args: Parameters<typeof ipcRenderer.invoke>) {
    const [channel, ...omit] = args
    return ipcRenderer.invoke(channel, ...omit)
  },

  // You can expose other APTs you need here.
  // ...
})

contextBridge.exposeInMainWorld("db", {
  connect: (config: any) => ipcRenderer.invoke("db:connect", config),
  query: (sql: string) => ipcRenderer.invoke("db:query", sql),
  structure: () => ipcRenderer.invoke("db:structure"),
  disconnect: () => ipcRenderer.invoke("db:disconnect"),
});

contextBridge.exposeInMainWorld("terminal", {
  write: (data: any) => ipcRenderer.send("terminal-write", data),
  onData: (callback: any) =>
    ipcRenderer.on("terminal-data", (_, data) => callback(data))
});
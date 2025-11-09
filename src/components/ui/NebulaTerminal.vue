import { Terminal } from "xterm";
import { FitAddon } from "xterm-addon-fit";
import "xterm/css/xterm.css";

const term = new Terminal();
const fitAddon = new FitAddon();
term.loadAddon(fitAddon);

term.open(document.getElementById("terminal"));
fitAddon.fit();

// incoming PTY data
window.terminal.onData(data => term.write(data));

// user keypress → PTY
term.onData(data => window.terminal.write(data));

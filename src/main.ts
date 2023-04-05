import {Terminal} from "xterm";
import {FitAddon} from "xterm-addon-fit";
import {WebLinksAddon} from 'xterm-addon-web-links';
import "xterm/css/xterm.css";
import {invoke} from "@tauri-apps/api";
import {Event, listen} from "@tauri-apps/api/event";

const terminalElement = document.getElementById("terminal") as HTMLElement;
const fontsfonts = [
    'Noto Mono for Powerline',
    'Roboto Mono for Powerline',
    'Jetbrains Mono',
    'Menlo',
    'Monaco',
    'Consolas',
    'Liberation Mono',
    'Courier New',
    'Noto Sans Mono CJK SC',
    'Noto Sans Mono CJK TC',
    'Noto Sans Mono CJK KR',
    'Noto Sans Mono CJK JP',
    'Noto Sans Mono CJK HK',
    'Noto Color Emoji',
    'Noto Sans Symbols',
    'monospace',
    'sans-serif',]
const fitAddon = new FitAddon();
const term = new Terminal({
    fontFamily: fontsfonts.join(","),
    theme: {
        background: "rgb(47, 47, 47)",
    }
});
term.loadAddon(new WebLinksAddon());
term.loadAddon(fitAddon);
term.open(terminalElement);

// Make the terminal fit all the window size
async function fitTerminal() {
    fitAddon.fit();
    let os=await invoke<string>("get_os_name")
    term.writeln("os:"+os);
    void invoke("async_shell", {
        shell: os=="windows"?"powershell.exe":"zsh",
    });
    void invoke<string>("async_resize_pty", {
        rows: term.rows,
        cols: term.cols,
    });
    setTimeout(()=>{
        invoke<string>("async_exec",{cmd:"pwd"})
            .then(s=>term.writeln(s)).catch(e=>term.writeln("err"+e.toString()))
    },2000)

}

// Write data from pty into the terminal
function writeToTerminal(ev: Event<string>) {
    console.log(ev.payload)
    term.write(ev.payload)
}

// Write data from the terminal to the pty
function writeToPty(data: string) {
    void invoke("async_write_to_pty", {
        data,
    });
}


term.onData(writeToPty);
addEventListener("resize", fitTerminal);
listen("data", writeToTerminal)
fitTerminal();
<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { Terminal } from 'xterm';
    import { FitAddon } from 'xterm-addon-fit';
    import { WebglAddon } from 'xterm-addon-webgl';
    import { listen } from '@tauri-apps/api/event';
    import { invoke } from '@tauri-apps/api/core';
    import 'xterm/css/xterm.css';
    import { getCurrentWindow } from '@tauri-apps/api/window';

    let terminalEl: HTMLDivElement;
    let term: Terminal;
    const fitAddon = new FitAddon();

    const catppuccinMocha = {
        background: '#1e1e2e',
        foreground: '#cdd6f4',
        cursor: '#f5e0dc',
        cursorAccent: '#1e1e2e',
        selectionBackground: 'rgba(91, 95, 122, 0.5)',
        black: '#45475a', red: '#f38ba8', green: '#a6e3a1',
        yellow: '#f9e2af', blue: '#89b4fa', magenta: '#f5c2e7',
        cyan: '#94e2d5', white: '#bac2de',
        brightBlack: '#585b70', brightRed: '#f38ba8', brightGreen: '#a6e3a1',
        brightYellow: '#f9e2af', brightBlue: '#89b4fa', brightMagenta: '#f5c2e7',
        brightCyan: '#94e2d5', brightWhite: '#a6adc8',
    };

    onMount(async () => {
        const appWindow = getCurrentWindow();
        term = new Terminal({
            fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
            fontSize: 14,
            cursorBlink: true,
            theme: catppuccinMocha,
            allowTransparency: false,
            devicePixelRatio: window.devicePixelRatio,
        });

        term.loadAddon(fitAddon);
        term.open(terminalEl);

        try {
            const webglAddon = new WebglAddon();
            term.loadAddon(webglAddon);
        } catch (e) {
            console.warn("WebGL renderer failed to load.", e);
        }

        fitAddon.fit();

        const unlisten = await listen<Uint8Array>('terminal-output', (event) => {
            term.write(event.payload);
        });

        // THE FIX: Send the data as a simple string with the key 'text'
        const onDataUnlisten = term.onData((data) => {
            invoke('write_to_pty', { text: data });
        });

        const unlistenResize = await appWindow.onResized(() => {
            setTimeout(() => fitAddon.fit(), 50);
        });

        const startupCommand = sessionStorage.getItem('startupCommand');
        if (startupCommand) {
            console.log("Found startup command:", startupCommand);
            // If a command is found, send it to the terminal
            invoke('write_to_pty', { text: startupCommand });

            // Clear the command from storage so it doesn't run again on a page refresh
            sessionStorage.removeItem('startupCommand');
        }

        onDestroy(() => {
            unlisten();
            unlistenResize();
            onDataUnlisten.dispose();
            term.dispose();
        });

        term.focus();


    });
</script>

<div bind:this={terminalEl} class="w-full h-full"></div>
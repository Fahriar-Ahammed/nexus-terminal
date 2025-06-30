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

    // The theme can be kept as is, it's great.
    const catppuccinMocha = {
        background: '#1e1e2e',
        foreground: '#cdd6f4',
        cursor: '#f5e0dc',
        cursorAccent: '#1e1e2e',
        selectionBackground: '#585b70',
        black: '#45475a',
        red: '#f38ba8',
        green: '#a6e3a1',
        yellow: '#f9e2af',
        blue: '#89b4fa',
        magenta: '#f5c2e7',
        cyan: '#94e2d5',
        white: '#bac2de',
        brightBlack: '#585b70',
        brightRed: '#f38ba8',
        brightGreen: '#a6e3a1',
        brightYellow: '#f9e2af',
        brightBlue: '#89b4fa',
        brightMagenta: '#f5c2e7',
        brightCyan: '#94e2d5',
        brightWhite: '#a6adc8',
    };

    onMount(async () => {
        const appWindow = getCurrentWindow();
        term = new Terminal({
            fontSize: 15,
            fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
            cursorBlink: true,
            theme: catppuccinMocha,
            allowProposedApi: true,
        });

        term.loadAddon(fitAddon);
        term.open(terminalEl);

        try {
            const webglAddon = new WebglAddon();
            term.loadAddon(webglAddon);
            console.log("WebGL renderer enabled.");
        } catch (e) {
            console.warn("WebGL renderer failed to load, falling back to canvas.", e);
        }

        fitAddon.fit();

        const unlisten = await listen<Uint8Array>('terminal-output', (event) => {
            term.write(event.payload);
        });

        // --- THIS IS THE CORRECTED SECTION ---
        // Handle user input (typing in the terminal)
        const onDataUnlisten = term.onData((data) => {
            const encoder = new TextEncoder();
            // 1. Call the correct command: `write_to_pty`
            // 2. Send the correct data format: `{ bytes: ... }`
            invoke('write_to_pty', { bytes: Array.from(encoder.encode(data)) });
        });

        const unlistenResize = await appWindow.onResized(() => {
            setTimeout(() => fitAddon.fit(), 50);
        });

        // Clean up all listeners when the component is destroyed
        onDestroy(() => {
            unlisten();
            unlistenResize();
            onDataUnlisten.dispose(); // Also dispose the onData listener
            term.dispose();
        });

        term.focus(); // Focus the terminal on start
    });
</script>

<div bind:this={terminalEl} class="w-full h-full"></div>
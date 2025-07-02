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
            fontSize: 16, // A slightly smaller, even font size often renders more clearly
            cursorBlink: true,
            theme: catppuccinMocha,
            // FIX #1: Add these two options for crisp fonts
            allowTransparency: false,
            letterSpacing: 1,
            lineHeight: 1,
        });

        term.loadAddon(fitAddon);
        term.open(terminalEl);

        fitAddon.fit();

        const unlisten = await listen<Uint8Array>('terminal-output', (event) => {
            term.write(event.payload);
        });

        // FIX #2: Make sure this calls the 'write_to_pty' command
        const onDataUnlisten = term.onData((data) => {
            const encoder = new TextEncoder();
            invoke('write_to_pty', { bytes: Array.from(encoder.encode(data)) });
        });

        const unlistenResize = await appWindow.onResized(() => {
            setTimeout(() => fitAddon.fit(), 50);
        });

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
<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { Terminal } from 'xterm';
    import { FitAddon } from 'xterm-addon-fit';
    import { WebglAddon } from 'xterm-addon-webgl';
    import { listen } from '@tauri-apps/api/event';
    import { invoke } from '@tauri-apps/api/core';
    import 'xterm/css/xterm.css';

    let terminalEl: HTMLDivElement;
    let term: Terminal;
    const fitAddon = new FitAddon();

    onMount(async () => {
        // Create a new Xterm.js instance
        term = new Terminal({
            fontFamily: 'JetBrains Mono, Fira Code, monospace',
            fontSize: 14,
            cursorBlink: true,
            theme: {
                background: '#0d1117',
                foreground: '#e6edf3',
                cursor: '#e6edf3',
                selectionBackground: '#3392FF',
                black: '#484f58',
                red: '#ff7b72',
                green: '#3fb950',
                yellow: '#d29922',
                blue: '#58a6ff',
                magenta: '#bc8cff',
                cyan: '#39c5cf',
                white: '#b1bac4',
            },
        });

        // Load addons
        term.loadAddon(fitAddon);

        // Open the terminal in our div
        term.open(terminalEl);

        // Load and activate the WebGL renderer
        const webglAddon = new WebglAddon();
        term.loadAddon(webglAddon);

        // Make the terminal fit the container
        fitAddon.fit();

        // Listen for output from the Rust backend
        const unlisten = await listen<Uint8Array>('terminal-output', (event) => {
            term.write(event.payload);
        });

        // Handle user input (typing in the terminal)
        term.onData((data) => {
            invoke('write_to_shell', { text: data });
        });

        // Handle resizing
        window.addEventListener('resize', () => fitAddon.fit());

        // Clean up when the component is destroyed
        onDestroy(() => {
            unlisten();
            term.dispose();
            window.removeEventListener('resize', () => fitAddon.fit());
        });
    });
</script>

<div bind:this={terminalEl} class="w-full h-full"></div>
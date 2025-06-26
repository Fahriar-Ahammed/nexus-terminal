<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { Terminal } from 'xterm';
    import { FitAddon } from 'xterm-addon-fit';
    import { WebglAddon } from 'xterm-addon-webgl';
    import { listen } from '@tauri-apps/api/event';
    import { invoke } from '@tauri-apps/api/core';
    import 'xterm/css/xterm.css';
    import { getCurrentWindow } from '@tauri-apps/api/window'; // Good for window operations

    const appWindow = getCurrentWindow(); // Useful for window-related tasks

    let terminalEl: HTMLDivElement;
    let term: Terminal;
    const fitAddon = new FitAddon();

    // A modern theme like Catppuccin (Mocha) for better organization
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
        // Create a new Xterm.js instance
        term = new Terminal({
            fontSize: 15,
            fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
            cursorBlink: true,
            theme: catppuccinMocha,
            allowProposedApi: true, // Needed for some modern renderer features
        });

        // Load addons
        term.loadAddon(fitAddon);

        // Open the terminal in our div
        term.open(terminalEl);

        // Load and activate the WebGL renderer for performance
        try {
            const webglAddon = new WebglAddon();
            term.loadAddon(webglAddon);
            console.log("WebGL renderer enabled.");
        } catch (e) {
            console.warn("WebGL renderer failed to load, falling back to canvas.", e);
        }

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

        // Handle resizing using Tauri's event for better accuracy
        const unlistenResize = await appWindow.onResized(() => {
            // Use a small timeout to ensure the DOM has updated
            setTimeout(() => fitAddon.fit(), 50);
        });

        // Clean up when the component is destroyed
        onDestroy(() => {
            unlisten();
            unlistenResize(); // Unlisten from the window resize event
            term.dispose();
        });
    });
</script>

<div bind:this={terminalEl} class="w-full h-full"></div>
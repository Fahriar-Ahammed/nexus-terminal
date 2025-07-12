<!-- src/lib/components/Titlebar.svelte -->
<script lang="ts">
    import { getCurrentWindow } from '@tauri-apps/api/window';

    export let onSshClick: () => void;

    const appWindow = getCurrentWindow();
    const minimizeWindow = () => appWindow.minimize();
    const toggleMaximizeWindow = () => appWindow.toggleMaximize();
    const closeWindow = () => appWindow.close();
</script>

<!--
    The header now uses a robust CSS Grid layout to ensure the drag
    region is never blocked by other elements.
-->
<header class="grid grid-cols-[auto_1fr_auto] h-11 items-center select-none">
    <!-- Left side: macOS-style window controls -->
    <div class="flex items-center gap-2 px-4 group">
        <!-- Added aria-label to each button for accessibility -->
        <button on:click={closeWindow} aria-label="Close" class="w-3.5 h-3.5 bg-[#ff5f57] rounded-full flex justify-center items-center">
            <svg class="w-2 h-2 opacity-0 group-hover:opacity-100 transition-opacity" viewBox="0 0 8 8" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M1 1L7 7M1 7L7 1" stroke="#410000" stroke-width="1.5"/>
            </svg>
        </button>

        <button on:click={minimizeWindow} aria-label="Minimize" class="w-3.5 h-3.5 bg-[#febc2e] rounded-full flex justify-center items-center">
            <svg class="w-2 h-2 opacity-0 group-hover:opacity-100 transition-opacity" viewBox="0 0 8 2" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M1 1H7" stroke="#5a3b00" stroke-width="1.5"/>
            </svg>
        </button>

        <button on:click={toggleMaximizeWindow} aria-label="Maximize" class="w-3.5 h-3.5 bg-[#28c840] rounded-full flex justify-center items-center">
            <svg class="w-2 h-2 opacity-0 group-hover:opacity-100 transition-opacity" viewBox="0 0 8 8" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M1.5 1.5H6.5V6.5H1.5V1.5Z" stroke="#004100" stroke-width="1.5"/>
            </svg>
        </button>
    </div>

    <!-- Center: Draggable Title Area -->
    <div data-tauri-drag-region class="h-full flex justify-center items-center text-gray-400 text-sm font-medium">
        <span>Nexus Terminal</span>
    </div>

    <!-- Right side: SSH Manager Button -->
    <div class="px-4 flex items-center">
        <button on:click={onSshClick} aria-label="Open SSH Manager" class="w-8 h-8 flex justify-center items-center text-gray-400 hover:text-white">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"></polyline><line x1="12" y1="19" x2="20" y2="19"></line></svg>
        </button>
    </div>
</header>
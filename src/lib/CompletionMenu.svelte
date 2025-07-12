<script lang="ts">
    import { onMount, afterUpdate } from 'svelte';
    import { File, Folder, History, Terminal as TerminalIcon } from 'lucide-svelte';

    export let options: CompletionOption[] = [];
    export let selectedIndex = 0;
    export let position = { x: 0, y: 0 };

    interface CompletionOption {
        value: string;
        display: string;
        completion_type: 'History' | 'Executable' | 'File' | 'Directory';
    }

    let menuElement: HTMLDivElement;

    // Scroll the selected item into view when it changes
    afterUpdate(() => {
        const selectedEl = menuElement?.querySelector('.selected');
        if (selectedEl) {
            selectedEl.scrollIntoView({ block: 'nearest' });
        }
    });
</script>

<div
        bind:this={menuElement}
        class="absolute bg-[#2a2a3e] border border-gray-600 rounded-md shadow-lg
           max-h-60 overflow-y-auto text-sm text-gray-200 z-50 p-1"
        style="left: {position.x}px; top: {position.y}px;"
>
    <ul>
        {#each options as option, i}
            <li
                    class="flex items-center gap-2 px-3 py-1.5 rounded-md cursor-pointer whitespace-nowrap {selectedIndex === i ? 'bg-[#4a4a6e] text-white selected' : 'hover:bg-[#3a3a5e]'}"
            >
                {#if option.completion_type === 'File'}
                    <File class="w-4 h-4 text-gray-400" />
                {:else if option.completion_type === 'Directory'}
                    <Folder class="w-4 h-4 text-cyan-400" />
                {:else if option.completion_type === 'History'}
                    <History class="w-4 h-4 text-purple-400" />
                {:else if option.completion_type === 'Executable'}
                    <TerminalIcon class="w-4 h-4 text-green-400" />
                {/if}
                <span>{option.display}</span>
            </li>
        {/each}
    </ul>
</div>


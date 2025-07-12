<script lang="ts">
    import Terminal from './lib/Terminal.svelte';
    import SshManager from './lib/SshManager.svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { addSshTab } from './store/tabStore';

    let showSshManager = true;

    async function handleConnectSsh(event: CustomEvent) {
        const { host, user } = event.detail;
        console.log(`Connecting to SSH: ${user}@${host}`);
        try {
            const result = await invoke('connect_ssh', { host, user });
            console.log(result);
            addSshTab(host, user);
            showSshManager = false;
        } catch (error) {
            console.error('Failed to connect SSH:', error);
        }
    }
</script>

<main class="w-screen h-screen bg-[#0d1117]">
    {#if showSshManager}
        <SshManager on:connectSsh={handleConnectSsh} />
    {:else}
        <Terminal />
    {/if}
</main>
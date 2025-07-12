<script lang="ts">
  import '../app.css';
  import Terminal from "$lib/Terminal.svelte";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import SshManager from "$lib/SshManager.svelte";
  import { invoke } from '@tauri-apps/api/core';

  let showSsh = false;

  function toggleSshManager() {
    showSsh = !showSsh;
  }

  async function initiateSshConnection(connection: any) {
    console.log('Attempting to connect to:', connection);
    showSsh = false; // Close the modal
    // TODO: Call Tauri backend command to establish SSH connection
    await invoke('connect_ssh', { host: connection.host, user: connection.user, password: connection.password });
  }
</script>


<div class="h-screen w-screen bg-[#24273a] flex flex-col rounded-xl overflow-hidden shadow-2xl">
  <TitleBar onSshClick={toggleSshManager} />

  <main class="flex-grow overflow-y-auto relative">
    <Terminal />

    {#if showSsh}
      <!-- Modal Overlay for SSH Manager -->
      <div class="absolute inset-0 z-10 flex items-center justify-center bg-black bg-opacity-70"
           on:click|self={() => (showSsh = false)}>
        <div class="w-full max-w-5xl h-full max-h-[85vh] bg-gray-800 rounded-xl shadow-2xl overflow-hidden"
             on:click|stopPropagation>
          <SshManager on:connectSsh={e => initiateSshConnection(e.detail)} />
        </div>
      </div>
    {/if}
  </main>
</div>
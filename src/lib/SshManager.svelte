<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  const dispatch = createEventDispatcher();

  let connections: any[] = [];
  let formName = '';
  let formHost = '';
  let formUser = '';
  let formPassword = '';

  onMount(async () => {
    console.log("SshManager: onMount - Loading connections...");
    connections = await invoke('load_connections');
    console.log("SshManager: Connections loaded:", connections);
  });

  async function addConnection() {
    if (formName && formHost && formUser) {
      const newConnection = {
        name: formName,
        host: formHost,
        user: formUser,
      };
      console.log("SshManager: Adding new connection:", newConnection);
      await invoke('save_connection', { host: formHost, user: formUser, password: formPassword });
      connections = await invoke('load_connections');
      console.log("SshManager: Connections saved and reloaded.");

      formName = '';
      formHost = '';
      formUser = '';
      formPassword = '';
    }
  }

  async function deleteConnection(connectionToDelete: any) {
    console.log("SshManager: Deleting connection:", connectionToDelete);
    await invoke('delete_connection', { host: connectionToDelete.host, user: connectionToDelete.user, password: connectionToDelete.password });
    connections = await invoke('load_connections');
    console.log("SshManager: Connections deleted and reloaded.");
  }

  function connect(connection: any) {
    console.log("SshManager: Dispatching connectSsh event with connection:", connection);
    dispatch('connectSsh', { ...connection, password: formPassword });
  }
</script>

<div class="p-4 bg-gray-800 text-white h-full">
  <h2 class="text-2xl font-bold mb-4">SSH Connections</h2>

  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    <!-- Add new connection form -->
    <div class="bg-gray-700 p-4 rounded-lg">
      <h3 class="text-xl font-semibold mb-2">Add New Connection</h3>
      <form on:submit|preventDefault={addConnection}>
        <div class="mb-2">
          <label for="name" class="block text-sm font-medium">Name</label>
          <input type="text" id="name" bind:value={formName} class="w-full p-2 bg-gray-600 rounded" required />
        </div>
        <div class="mb-2">
          <label for="host" class="block text-sm font-medium">Host</label>
          <input type="text" id="host" bind:value={formHost} class="w-full p-2 bg-gray-600 rounded" required />
        </div>
        <div class="mb-2">
          <label for="user" class="block text-sm font-medium">User</label>
          <input type="text" id="user" bind:value={formUser} class="w-full p-2 bg-gray-600 rounded" required />
        </div>
        <div class="mb-2">
          <label for="password" class="block text-sm font-medium">Password</label>
          <input type="password" id="password" bind:value={formPassword} class="w-full p-2 bg-gray-600 rounded" />
        </div>
        <button type="submit" class="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
          Add Connection
        </button>
      </form>
    </div>

    <!-- Connection list -->
    <div class="bg-gray-700 p-4 rounded-lg">
        <h3 class="text-xl font-semibold mb-2">Saved Connections</h3>
        <div class="space-y-2">
            {#each connections as connection (connection.id)}
            <div class="flex items-center justify-between bg-gray-600 p-2 rounded">
                <div>
                    <p class="font-semibold">{connection.name}</p>
                    <p class="text-sm text-gray-400">{connection.user}@{connection.host}</p>
                </div>
                <div class="flex items-center gap-2">
                    <button on:click={() => connect(connection)} class="bg-green-600 hover:bg-green-700 text-white font-bold py-1 px-3 rounded">
                        Connect
                    </button>
                    <button on:click={() => deleteConnection(connection.id)} class="bg-red-600 hover:bg-red-700 text-white font-bold py-1 px-3 rounded">
                        Delete
                    </button>
                </div>
            </div>
            {:else}
            <p>No SSH connections saved yet.</p>
            {/each}
        </div>
    </div>
  </div>
</div>

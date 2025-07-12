<script lang="ts">
    import { onMount } from 'svelte';
    import { invoke } from '@tauri-apps/api/core';
    import { goto } from '$app/navigation';

    interface SshConnection {
        id: string;
        name: string;
        host: string;
        user: string;
        password?: string;
    }

    let connections: SshConnection[] = [];
    let formName = '';
    let formHost = '';
    let formUser = '';
    let formPassword = '';

    onMount(async () => {
        connections = await invoke('get_connections');
    });

    async function addConnection() {
        if (!formName || !formHost || !formUser) return;

        const newConnection: SshConnection = {
            id: Date.now().toString(),
            name: formName,
            host: formHost,
            user: formUser,
            password: formPassword || undefined,
        };

        const updatedConnections = [...connections, newConnection];
        await invoke('update_connections', { connections: updatedConnections });
        connections = updatedConnections;

        formName = '';
        formHost = '';
        formUser = '';
        formPassword = '';
    }

    async function deleteConnection(idToDelete: string) {
        const updatedConnections = connections.filter(c => c.id !== idToDelete);
        await invoke('update_connections', { connections: updatedConnections });
        connections = updatedConnections;
    }

    function connect(connection: SshConnection) {
        let command: string;
        if (connection.password) {
            command = `sshpass -p '${connection.password}' ssh ${connection.user}@${connection.host}\r`;
        } else {
            command = `ssh ${connection.user}@${connection.host}\r`;
        }

        // --- DEBUGGING LINES ---
        console.log("Preparing to connect with command:", command);
        //alert(`Navigating to terminal with command: ${command}`);
        // -----------------------

        sessionStorage.setItem('startupCommand', command);
        goto('/');
    }
</script>

<div class="p-4 bg-gray-800 text-white h-full overflow-y-auto">
    <h2 class="text-2xl font-bold mb-4">SSH Connections</h2>
    <p class="text-sm text-yellow-400 bg-yellow-900/50 p-3 rounded-md mb-4">
        <strong>Security Warning:</strong> Passwords are saved in plaintext. For production use, please rely on SSH keys. Password connections also require `sshpass` to be installed.
    </p>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
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
                    <label for="password" class="block text-sm font-medium">Password (Optional)</label>
                    <input type="password" id="password" bind:value={formPassword} class="w-full p-2 bg-gray-600 rounded" />
                </div>
                <button type="submit" class="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">
                    Add Connection
                </button>
            </form>
        </div>

        <div class="bg-gray-700 p-4 rounded-lg">
            <h3 class="text-xl font-semibold mb-2">Saved Connections</h3>
            <div class="space-y-2 max-h-[400px] overflow-y-auto">
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
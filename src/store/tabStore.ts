import { writable } from 'svelte/store';

export interface Tab {
    id: string; // Use string for ID for more flexibility later
    title: string;
    workingDir: string;
}

// This store will hold an array of our open tabs
export const tabStore = writable<{ tabs: Tab[] }>({
    tabs: []
});

// Function to add a new tab
export function addTab(title: string, workingDir: string) {
    tabStore.update(current => {
        const newTab: Tab = {
            id: crypto.randomUUID(), // Generate a unique ID for the tab
            title,
            workingDir
        };
        return { tabs: [...current.tabs, newTab] };
    });
}

// Function to add a new SSH tab
export function addSshTab(host: string, user: string) {
    tabStore.update(current => {
        const newTab: Tab = {
            id: crypto.randomUUID(),
            title: `${user}@${host}`,
            workingDir: `ssh://${user}@${host}`
        };
        return { tabs: [...current.tabs, newTab] };
    });
}

// Function to remove a tab by its ID
export function removeTab(id: string) {
    tabStore.update(current => {
        return { tabs: current.tabs.filter(tab => tab.id !== id) };
    });
}

// Function to update a tab's properties (e.g., title)
export function updateTab(id: string, newTitle: string) {
    tabStore.update(current => {
        return {
            tabs: current.tabs.map(tab =>
                tab.id === id ? { ...tab, title: newTitle } : tab
            )
        };
    });
}

// Function to get a tab by its ID (useful for active tab management)
export function getTab(id: string) {
    let tab: Tab | undefined;
    tabStore.subscribe(current => {
        tab = current.tabs.find(t => t.id === id);
    })(); // Immediately invoke to get current value
    return tab;
}

// Function to set the active tab (if you implement active tab logic)
// For now, we'll just return the ID of the newly added tab
export function setActiveTab(id: string) {
    // This function would typically update a separate 'activeTabId' store
    // or a property within the tabStore itself.
    // For this example, we'll just log it.
    console.log(`Active tab set to: ${id}`);
}

// Initial tab setup (optional, you might want to load from storage)
// addTab('Local Terminal', '~');
// addTab('SSH Session', 'ssh://user@host');
// addTab('Another Terminal', '/tmp');

// Example usage:
// addTab('New Session', '/home/user');
// removeTab(someTabId);
// updateTab(someTabId, 'Renamed Session');
// const myTab = getTab(someTabId);
// setActiveTab(myTab.id);

// Note: This is a basic store. For more complex state management,
// consider libraries like Redux, Zustand, or Svelte's built-in context API.

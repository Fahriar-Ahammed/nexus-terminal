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
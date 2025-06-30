<script lang="ts">
  import '../app.css';
import Terminal from "$lib/Terminal.svelte";
  import TitleBar from "$lib/components/TitleBar.svelte";
</script>


<div class="h-screen w-screen bg-[#24273a] flex flex-col rounded-xl overflow-hidden shadow-2xl">
  <TitleBar />

  <main class="flex-grow overflow-y-auto">
    <Terminal />
  </main>
</div>
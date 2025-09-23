<script lang="ts">
  import Footer from "./Footer.svelte";
  import Header from "./Header.svelte";

  let { children } = $props();
</script>

<!-- flex flex-col min-h-screen → makes the page stretch full height and stack header, body, and footer. -->
<!-- flex-1 on <main> → makes the body expand to fill leftover space, pushing the footer down. -->
<div class="flex flex-col h-screen text-sm text-white">
  <Header />
  <main class="flex-1 overflow-y-auto">
    {@render children()}
  </main>
  <Footer/>
</div>

<style>
  :global {
    /* Hide scrollbars globally but keep scrolling functional */
    * {
      -ms-overflow-style: none; /* IE and Edge */
      scrollbar-width: none; /* Firefox */
    }
    *::-webkit-scrollbar {
      display: none; /* Chrome, Safari, Edge */
    }
  }
</style>

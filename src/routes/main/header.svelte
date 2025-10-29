<script lang="ts">
  import { Separator } from "$lib/components/ui/separator/index.js";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";

  import { page } from "$app/state";

  import { SIDEBAR_ROUTES } from "./routes.svelte";

  let currentPage = $derived(() => {
    const pathname = page.url.pathname;
    // Check for exact match first
    if (SIDEBAR_ROUTES[pathname]) {
      return SIDEBAR_ROUTES[pathname];
    }
    // Check if it's a dynamic route under /main/history
    if (pathname.startsWith("/main/history/")) {
      return SIDEBAR_ROUTES["/main/history"];
    }
    return null;
  });
</script>

<header class="flex h-14 shrink-0 items-center gap-2 border-b transition-[width,height] duration-200 ease-linear">
  <div class="flex w-full items-center gap-1 px-4 lg:px-6">
    <Sidebar.Trigger class="-ml-1" />
    <Separator orientation="vertical" class="mx-1 h-4" />
    <h1 class="text-lg font-medium">{currentPage()?.label ?? "History"}</h1>
  </div>
</header>

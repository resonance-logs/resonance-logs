<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { Button } from "$lib/components/ui/button/index.js";
    import { toast } from "svelte-sonner";

    async function openLogDir() {
        try {
            await invoke("open_log_dir");
        } catch (e) {
            console.error(e);
            toast.error("Failed to open log directory: " + e);
        }
    }

    async function createDiagnosticsBundle() {
        try {
            const path = await invoke<string>("create_diagnostics_bundle");
            try {
                await navigator.clipboard.writeText(path);
                toast.success("Diagnostics bundle created (path copied): " + path);
            } catch {
                toast.success("Diagnostics bundle created: " + path);
            }
        } catch (e) {
            console.error(e);
            toast.error("Failed to create diagnostics bundle: " + e);
        }
    }
</script>

<div class="space-y-3">
    <div
        class="overflow-hidden rounded-lg border border-border/60 bg-card/40 shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]"
    >
        <div class="px-4 py-3">
            <h2 class="mb-4 text-base font-semibold text-foreground">
                Debug
            </h2>

            <div class="flex items-center justify-between">
                <div class="text-sm text-muted-foreground">
                    <div class="font-medium text-foreground">Log Files</div>
                    Open the folder containing application logs
                </div>
                <Button variant="outline" onclick={openLogDir}>
                    Open Logs
                </Button>
            </div>

            <div class="mt-4 flex items-center justify-between">
                <div class="text-sm text-muted-foreground">
                    <div class="font-medium text-foreground">Diagnostics Bundle</div>
                    Create a ZIP with recent logs + settings (redacted) for support
                </div>
                <Button variant="outline" onclick={createDiagnosticsBundle}>
                    Create Bundle
                </Button>
            </div>
        </div>
    </div>
</div>

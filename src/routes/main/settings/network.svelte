<script lang="ts">
    import SettingsSelect from "./settings-select.svelte";
    import SettingsDropdown from "./settings-dropdown.svelte";
    import { SETTINGS } from "$lib/settings-store";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    type Device = {
        name: string;
        description: string | null;
    };

    let devices = $state<Device[]>([]);
    let npcapInstalled = $state(false);
    let loading = $state(false);

    async function loadDevices() {
        loading = true;
        try {
            npcapInstalled = await invoke("check_npcap_status");
            if (npcapInstalled) {
                devices = await invoke("get_network_devices");
            }
        } catch (e) {
            console.error("Failed to load network info", e);
        }
        loading = false;
    }

    onMount(() => {
        loadDevices();
    });

    let deviceOptions = $derived(
        devices.map((d) => ({
            value: d.name,
            label: d.description || d.name,
        })),
    );
</script>

<div class="space-y-3">
    <div
        class="rounded-lg border bg-card/40 border-border/60 overflow-hidden shadow-[inset_0_1px_0_0_rgba(255,255,255,0.02)]"
    >
        <div class="px-4 py-3">
            <h2 class="text-base font-semibold text-foreground mb-2">
                Packet Capture
            </h2>

            <SettingsSelect
                bind:selected={SETTINGS.packetCapture.state.method}
                label="Capture Method"
                description="Select the method used to capture network packets. Requires application restart."
                values={["WinDivert", "Npcap"]}
            />

            {#if SETTINGS.packetCapture.state.method === "Npcap"}
                {#if !npcapInstalled}
                    <div
                        class="mt-2 p-3 bg-destructive/10 text-destructive rounded-md text-sm"
                    >
                        Npcap is not detected. Please install Npcap from <a
                            href="https://npcap.com/"
                            target="_blank"
                            class="underline">npcap.com</a
                        > to use this feature.
                    </div>
                {:else}
                    <SettingsDropdown
                        bind:selected={SETTINGS.packetCapture.state.npcapDevice}
                        label="Network Device"
                        description="Select the network adapter to capture traffic from."
                        options={deviceOptions}
                        placeholder={loading
                            ? "Loading devices..."
                            : "Select a device"}
                    />
                {/if}
            {/if}
        </div>
    </div>
</div>

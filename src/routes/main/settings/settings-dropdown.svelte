<script lang="ts">
    let {
        label = "",
        description = "",
        selected = $bindable(""),
        options = [],
        placeholder = "Select an option",
        disabled = false,
    }: {
        label: string;
        description?: string;
        selected: string;
        options: { value: string; label: string }[];
        placeholder?: string;
        disabled?: boolean;
    } = $props();
</script>

<label
    class="flex flex-col gap-1.5 py-2.5 px-3 rounded-md hover:bg-popover/50 transition-colors"
>
    <div class="flex flex-col gap-0.5">
        <div class="text-sm font-medium text-foreground">{label}</div>
        {#if description}
            <div class="text-xs text-muted-foreground leading-relaxed">
                {description}
            </div>
        {/if}
    </div>
    <select
        bind:value={selected}
        {disabled}
        class="w-full px-3 py-2 text-sm bg-popover border border-border rounded-md
           focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary
           text-foreground
           disabled:opacity-50 disabled:cursor-not-allowed
           transition-colors appearance-none"
        style="background-image: url('data:image/svg+xml;charset=US-ASCII,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20width%3D%22292.4%22%20height%3D%22292.4%22%3E%3Cpath%20fill%3D%22%23a1a1aa%22%20d%3D%22M287%2069.4a17.6%2017.6%200%200%200-13-5.4H18.4c-5%200-9.3%201.8-12.9%205.4A17.6%2017.6%200%200%200%200%2082.2c0%205%201.8%209.3%205.4%2012.9l128%20127.9c3.6%203.6%207.8%205.4%2012.8%205.4s9.2-1.8%2012.8-5.4L287%2095c3.5-3.5%205.4-7.8%205.4-12.8%200-5-1.9-9.2-5.5-12.8z%22%2F%3E%3C%2Fsvg%3E'); background-repeat: no-repeat; background-position: right 0.7rem top 50%; background-size: 0.65rem auto; padding-right: 2.5rem;"
    >
        {#if placeholder}
            <option value="" disabled selected={selected === ""}
                >{placeholder}</option
            >
        {/if}
        {#each options as option}
            <option value={option.value}>{option.label}</option>
        {/each}
    </select>
</label>

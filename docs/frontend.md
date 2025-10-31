# Frontend Developer Guide

## Stack Overview
- [Svelte 5](https://svelte.dev/) with SvelteKit 2 provides routing, layouts, and server integration.
- [Tauri Plugin Svelte](https://github.com/tauri-apps/tauri-plugin-svelte) is used for persistent RuneStore state shared with Rust.
- Tailwind-based styling utilities live in `src/lib/components/ui`. Components follow a variant of the shadcn-svelte pattern.

## App Layouts and Windows
- `src/routes/+layout.svelte` loads global CSS (`app.css`) and disables the default context menu to avoid accidental copy/paste during overlay use.
- `src/routes/main/+layout.svelte` renders the sidebar layout, header, and `children()` slot. It registers global shortcuts via `$effect.pre()` hook calling `setupShortcuts()` to ensure keyboard commands stay in sync with backend settings. @src/routes/main/+layout.svelte#1-38
- `src/routes/live/+page.svelte` immediately redirects to `/live/dps/` after mount, ensuring the live window always opens on the DPS leaderboard. @src/routes/live/+page.svelte#1-12

### Routing Structure
```
src/routes
├── +layout.svelte        # global layout
├── live/                 # overlay window
│   ├── +layout.svelte    # live window shell
│   ├── +page.svelte      # redirects to /live/dps
│   ├── dps/              # DPS tables with skill breakdowns
│   ├── heal/             # heal tables with skill breakdowns
│   └── tanked/           # damage taken tables with skill breakdowns
└── main/                 # settings/history window
    ├── +layout.svelte    # sidebar + header
    ├── history/          # encounter history explorer
    ├── settings/         # settings tabs & components
    ├── changelog/        # release notes (WIP)
    ├── performance/      # (empty, reserved for future)
    └── performance-benchmark/  # (empty, reserved for future)
```
- Each live route consumes backend events to populate tables; see `src/lib/stores/live-meter-store.svelte.ts` for RuneStore wrappers that manage live player data (configured with `saveOnChange: false` for performance).
- The history page (`main/history/+page.svelte`) requests paginated data via `commands.getRecentEncountersFiltered` and manages filters/search chips with local state. @src/routes/main/history/+page.svelte#1-379

## UI Components
- All shared UI primitives live under `src/lib/components/ui`. Each folder exports an `index.ts` plus Svelte components (e.g., `button`, `dialog`, `sidebar`, `tabs`). Folders mimic the shadcn structure for consistent API surface.
- Higher-level widgets: `unified-search.svelte`, `filter-chips.svelte`, `player-info.svelte`, etc. They encapsulate specific behaviour (autocomplete, chip lists, player detail tooltips).

## State & Stores
- **Settings persistence**: `settings-store.ts` uses RuneStore (from `@tauri-store/svelte`) for persisted settings across app restarts. Exposes typed stores for general/accessibility/shortcuts/live/misc sections. @src/lib/settings-store.ts#1-147
- **Live meter data**: `live-meter-store.svelte.ts` holds transient player lists for DPS/Heal/Tanked overlays using RuneStore wrappers configured with `saveOnChange: false` for performance. Exported helpers (`setDpsPlayers`, `getDpsPlayers`, `clearMeterData`, etc.) are used by event handlers in `src/routes/live/+layout.svelte`. @src/lib/stores/live-meter-store.svelte.ts#1-60
- Svelte's `$state`, `$effect.pre`, and `@render` macros are heavily used because the project targets Svelte 5's runes API.

## Shortcut Management Flow
1. `settings/shortcuts.svelte` renders buttons per shortcut, capturing modifier combinations and saving to RuneStore. @src/routes/main/settings/shortcuts.svelte#1-178
2. `settings/shortcuts.ts` registers shortcuts with `@tauri-apps/plugin-global-shortcut`, mapping actions (show/hide/toggle overlay, click-through, reset encounter). @src/routes/main/settings/shortcuts.ts#1-109
3. Changes persist automatically via RuneStore and survive restarts.

## Consuming Backend Events
- `src/lib/api.ts` defines TypeScript types for encounter payloads and is the main entry point for invoking commands (`invoke` via `@tauri-apps/api/core`) or listening to events.
- Auto-generated `bindings.ts` exposes typed wrappers. Import `commands` or event streams from this file to avoid manual `invoke` usage for backend commands. @src/lib/bindings.ts#1-163
- Frontend sections typically:
  1. Call `commands.subscribePlayerSkills` or similar to request initial data from Rust.
  2. Listen for `players-update` / `skills-update` via Tauri events and push data into RuneStores.

## Development Notes
- Run `npm run tauri -- dev` (or use the `tauri dev` command directly) to start both the Vite dev server and Tauri shell. The Svelte app hot reloads while Rust restarts when backend code changes.
- TypeScript bindings are auto-generated in debug builds via Specta/tauri-specta into `src/lib/bindings.ts`.
- Linting is configured via ESLint/Prettier; use `npm run lint` and `npm run format`.
- Tailwind classes are merged via `tailwind-merge` to avoid duplication.
- When adding new routes to the sidebar, update `src/routes/main/routes.svelte.ts` with the route path, label, and icon.

## Testing the Overlay
- Use the global shortcuts to show/hide the live window.
- To simulate data, use debug commands or feed recorded packet streams (consult backend docs). Live tables update automatically when events emit.

/**
 * @file This file configures the rendering mode for the application.
 * Since Tauri doesn't have a Node.js server to do proper SSR,
 * we will use adapter-static to prerender the app (SSG).
 * See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
 */
export const prerender = true;
export const ssr = false;
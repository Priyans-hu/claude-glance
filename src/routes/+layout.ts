// Tauri doesn't have a Node.js server, so we use adapter-static with a fallback
// to put SvelteKit into SPA mode.
// https://svelte.dev/docs/kit/single-page-apps
// https://v2.tauri.app/start/frontend/sveltekit/
export const prerender = true;
export const ssr = false;

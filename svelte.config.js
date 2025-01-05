// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter(),
    // todo: add prerender.entries temporarily
    //       should route parameter `filepath` be as either path parameter, store or query parameter (ssr required...) ?
    prerender: {
      entries: ['/dashboard'],
    },
  },
};

export default config;

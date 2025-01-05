// Tauri doesn't have a Node.js server to do proper SSR
// so we will use adapter-static to prerender the app (SSG)
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
// todo: set false at prerender temporarily
//       route parameter `filepath` as path parameter, store or query parameter (ssr required...)
export const prerender = false
export const ssr = false

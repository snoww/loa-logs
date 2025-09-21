import { sveltekit } from "@sveltejs/kit/vite";
import Icons from "unplugin-icons/vite";
import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";
import devtoolsJson from 'vite-plugin-devtools-json';

export default defineConfig({
  plugins: [
    devtoolsJson(),
    tailwindcss(),
    sveltekit(),
    Icons({
      compiler: "svelte"
    })
  ],
  server: {
    watch: { ignored: ["**/src-tauri/**"] }
  }
});

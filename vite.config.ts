import { preprocessMeltUI, sequence } from "@melt-ui/pp";
import adapter from "@sveltejs/adapter-static";
import { sveltekit } from "@sveltejs/kit/vite";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import Icons from "unplugin-icons/vite";
import { defineConfig } from "vite";
import devtoolsJson from "vite-plugin-devtools-json";

export default defineConfig({
  plugins: [
    devtoolsJson(),
    tailwindcss(),
    sveltekit({
      preprocess: sequence([vitePreprocess(), preprocessMeltUI({ svelteConfigPath: false })]),
      adapter: adapter({ fallback: "index.html" }),
      version: { pollInterval: 0 }
    }),
    Icons({ compiler: "svelte" })
  ],
  server: {
    watch: { ignored: ["**/src-tauri/**"] }
  }
});

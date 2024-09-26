import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
import topLevelAwait from "vite-plugin-top-level-await";
import wasm from "vite-plugin-wasm";

// https://vitejs.dev/config/
export default defineConfig({
  base: "/rufus/",
  plugins: [react(), topLevelAwait(), wasm()],
})

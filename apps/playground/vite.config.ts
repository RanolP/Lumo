import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";

export default defineConfig({
  // Relative base so the bundle works under GitHub Pages project paths.
  base: "./",
  plugins: [solidPlugin()],
});

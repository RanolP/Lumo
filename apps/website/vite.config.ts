import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import mdx from "@mdx-js/rollup";

export default defineConfig({
  // "/" locally; CI sets BASE_PATH to "/<repo>/" for GitHub Pages project paths.
  base: process.env.BASE_PATH ?? "/",
  plugins: [
    {
      ...mdx({
        jsx: true,
        jsxImportSource: "solid-js",
        elementAttributeNameCase: "html",
      }),
      enforce: "pre",
    },
    solidPlugin({ extensions: [".mdx"] }),
  ],
});

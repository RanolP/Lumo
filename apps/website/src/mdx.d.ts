declare module "*.mdx" {
  import type { Component } from "solid-js";
  export const title: string | undefined;
  const MDXContent: Component;
  export default MDXContent;
}

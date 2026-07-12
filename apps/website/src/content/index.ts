import type { Component } from "solid-js";

type MdxModule = { default: Component; title?: string };

export type Entry = { slug: string; title: string; Body: Component };

function toEntries(modules: Record<string, MdxModule>): Entry[] {
  return Object.entries(modules)
    .map(([path, module]) => {
      const slug = path.replace(/^.*\//, "").replace(/\.mdx$/, "");
      return { slug, title: module.title ?? slug, Body: module.default };
    })
    .sort((a, b) => a.slug.localeCompare(b.slug));
}

export const docs = toEntries(
  import.meta.glob<MdxModule>("./docs/*.mdx", { eager: true }),
);
export const rfcs = toEntries(
  import.meta.glob<MdxModule>("./rfcs/*.mdx", { eager: true }),
);

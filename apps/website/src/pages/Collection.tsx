import { For, Show, type Component } from "solid-js";
import { Dynamic } from "solid-js/web";
import { A, useParams } from "@solidjs/router";
import type { Entry } from "../content";

// MDX compiles native tags to component references (strings by default),
// which Solid's createComponent cannot render — map each tag to Dynamic.
const MDX_TAGS = [
  "a", "blockquote", "code", "em", "h1", "h2", "h3", "h4", "h5", "h6",
  "hr", "img", "li", "ol", "p", "pre", "strong", "table", "tbody", "td",
  "th", "thead", "tr", "ul",
] as const;
const mdxComponents: Record<string, Component<Record<string, unknown>>> =
  Object.fromEntries(
    MDX_TAGS.map((tag) => [
      tag,
      (props: Record<string, unknown>) => <Dynamic component={tag} {...props} />,
    ]),
  );

// One component serves both the index (/docs) and an entry (/docs/:slug).
export function CollectionPage(props: {
  title: string;
  base: string;
  intro: string;
  entries: Entry[];
}) {
  const params = useParams();
  const entry = () => props.entries.find((candidate) => candidate.slug === params.slug);

  return (
    <main class="page">
      <Show
        when={params.slug}
        fallback={
          <>
            <h1>{props.title}</h1>
            <p class="hint">{props.intro}</p>
            <ul class="entry-list">
              <For each={props.entries}>
                {(item) => (
                  <li>
                    <A href={`${props.base}/${item.slug}`}>{item.title}</A>
                  </li>
                )}
              </For>
            </ul>
          </>
        }
      >
        <Show
          when={entry()}
          fallback={
            <p>
              No such entry. <A href={props.base}>Back to {props.title}.</A>
            </p>
          }
        >
          {(found) => (
            <>
              <p class="breadcrumb">
                <A href={props.base}>{props.title}</A> / {found().title}
              </p>
              <article class="prose">
                {found().Body({ components: mdxComponents })}
              </article>
            </>
          )}
        </Show>
      </Show>
    </main>
  );
}

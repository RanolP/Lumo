import { Suspense, type ParentProps } from "solid-js";
import { A } from "@solidjs/router";

export default function Layout(props: ParentProps) {
  return (
    <div class="site">
      <nav class="site-nav">
        <A href="/" class="brand" end>
          Lumo
        </A>
        <div class="nav-links">
          <A href="/playground">Playground</A>
          <A href="/docs">Docs</A>
          <A href="/rfcs">RFCs</A>
          <A href="/formalization">Formalization</A>
          <a href="https://github.com/RanolP/Lumo" rel="external">
            GitHub
          </a>
        </div>
      </nav>
      <Suspense>{props.children}</Suspense>
    </div>
  );
}

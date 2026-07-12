import { lazy, Suspense } from "solid-js";
import { A } from "@solidjs/router";

const Playground = lazy(() => import("../components/Playground"));

export default function Home() {
  return (
    <main class="home">
      <section class="hero">
        <p class="hero-kicker">experimental · under active development</p>
        <h1>
          Effects are <span class="accent">capabilities</span>.
        </h1>
        <p class="hero-tagline">
          Lumo is an experimental programming language where every side effect is a
          capability you pass, handle, and type — compiled to plain JavaScript through a
          call-by-push-value core.
        </p>
        <div class="hero-actions">
          <A class="btn btn-primary" href="/playground">
            Open the playground
          </A>
          <A class="btn" href="/docs">
            Read the docs
          </A>
        </div>
        <ul class="hero-features">
          <li>
            <h3>Capability-passing effects</h3>
            <p>
              Effect rows compile to ordinary parameters: handlers are lexically scoped
              bindings, with no continuations and no magic runtime.
            </p>
          </li>
          <li>
            <h3>A language defined, not written</h3>
            <p>
              Grammar, elaboration, and typing judgments live in one{" "}
              <code>.langue</code> definition; the whole compiler pipeline is generated
              from it.
            </p>
          </li>
          <li>
            <h3>Typed by judgments, optimized by e-graphs</h3>
            <p>
              A relational type system checks the CBPV intermediate representation
              directly, and egglog rewrite rules optimize it.
            </p>
          </li>
        </ul>
      </section>

      <section class="home-playground">
        <h2>Try it live</h2>
        <p class="hint">
          The full compiler runs in your browser via WebAssembly — edit the source and
          watch the JS, MIR, inferred types, and parse tree update.
        </p>
        <div class="home-playground-frame">
          <Suspense fallback={<p class="hint">Loading playground…</p>}>
            <Playground />
          </Suspense>
        </div>
      </section>
    </main>
  );
}

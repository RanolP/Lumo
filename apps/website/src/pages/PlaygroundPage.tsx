import { lazy, Suspense } from "solid-js";

const Playground = lazy(() => import("../components/Playground"));

export default function PlaygroundPage() {
  return (
    <main class="playground-page">
      <Suspense fallback={<p class="hint">Loading playground…</p>}>
        <Playground />
      </Suspense>
    </main>
  );
}

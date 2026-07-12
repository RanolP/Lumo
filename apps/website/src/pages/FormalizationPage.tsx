import { createResource, Show } from "solid-js";

// Generated from /formalization.typ by scripts/build-formalization.mjs
// (pnpm typst:build) into public/formalization.body.html.
async function fetchFormalization(): Promise<string> {
  const response = await fetch(`${import.meta.env.BASE_URL}formalization.body.html`);
  if (!response.ok) {
    throw new Error(`failed to load formalization (HTTP ${response.status})`);
  }
  return response.text();
}

export default function FormalizationPage() {
  const [body] = createResource(fetchFormalization);

  return (
    <main class="formalization-page">
      <Show when={!body.error} fallback={<p class="error">{String(body.error)}</p>}>
        <Show when={body()} fallback={<p class="hint">Loading formalization…</p>}>
          {(html) => <article class="formalization" innerHTML={html()} />}
        </Show>
      </Show>
    </main>
  );
}

import { A } from "@solidjs/router";

export default function NotFound() {
  return (
    <main class="page">
      <h1>404</h1>
      <p>
        This page does not exist. <A href="/">Back to the home page.</A>
      </p>
    </main>
  );
}

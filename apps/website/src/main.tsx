import { lazy } from "solid-js";
import { render } from "solid-js/web";
import { Route, Router } from "@solidjs/router";

import Layout from "./Layout";
import "./styles.css";

const Home = lazy(() => import("./pages/Home"));
const PlaygroundPage = lazy(() => import("./pages/PlaygroundPage"));
const FormalizationPage = lazy(() => import("./pages/FormalizationPage"));
const DocsPage = lazy(() => import("./pages/DocsPage"));
const RfcsPage = lazy(() => import("./pages/RfcsPage"));
const NotFound = lazy(() => import("./pages/NotFound"));

render(
  () => (
    <Router base={import.meta.env.BASE_URL.replace(/\/$/, "")} root={Layout}>
      <Route path="/" component={Home} />
      <Route path="/playground" component={PlaygroundPage} />
      <Route path="/formalization" component={FormalizationPage} />
      <Route path="/docs/:slug?" component={DocsPage} />
      <Route path="/rfcs/:slug?" component={RfcsPage} />
      <Route path="*" component={NotFound} />
    </Router>
  ),
  document.getElementById("root")!,
);

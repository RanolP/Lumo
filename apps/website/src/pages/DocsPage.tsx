import { docs } from "../content";
import { CollectionPage } from "./Collection";

export default function DocsPage() {
  return (
    <CollectionPage
      title="Docs"
      base="/docs"
      intro="Guides and reference material for Lumo. Managed as MDX files under apps/website/src/content/docs/."
      entries={docs}
    />
  );
}

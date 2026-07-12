import { rfcs } from "../content";
import { CollectionPage } from "./Collection";

export default function RfcsPage() {
  return (
    <CollectionPage
      title="RFCs"
      base="/rfcs"
      intro="Design proposals for the Lumo language. Managed as MDX files under apps/website/src/content/rfcs/."
      entries={rfcs}
    />
  );
}

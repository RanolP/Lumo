// Compiles /formalization.typ to an HTML body fragment served as a static
// asset (public/formalization.body.html) and rendered by FormalizationPage.
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const site = dirname(dirname(fileURLToPath(import.meta.url)));
const repoRoot = dirname(dirname(site));
const publicDir = join(site, "public");
const fullHtml = join(publicDir, "formalization.full.html.tmp");
const output = join(publicDir, "formalization.body.html");

mkdirSync(publicDir, { recursive: true });
execFileSync(
  "typst",
  [
    "compile",
    "--features",
    "html",
    "--format",
    "html",
    "--root",
    repoRoot,
    join(site, "typst", "formalization-html.typ"),
    fullHtml,
  ],
  { stdio: "inherit" },
);

const html = readFileSync(fullHtml, "utf8");
const start = html.indexOf("<body>");
const end = html.lastIndexOf("</body>");
if (start === -1 || end === -1) {
  throw new Error("typst output has no <body> — HTML export format changed?");
}
writeFileSync(output, html.slice(start + "<body>".length, end));
rmSync(fullHtml);
console.log(`wrote ${output}`);

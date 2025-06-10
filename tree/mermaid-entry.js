// This is the entry for the bundle of mermaid which is imported by the server to compile the tree on the backend
// The bundle is created via esbuild, with `pnpm bundle_mermaid`, and is output at `mermaid.min.js`
import mermaid from "mermaid";
import elkLayouts from "@mermaid-js/layout-elk";
mermaid.registerLayoutLoaders(elkLayouts);
window.mermaid = mermaid;

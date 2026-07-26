import * as esbuild from "esbuild";
import { mkdirSync } from "node:fs";

mkdirSync("dist/webview", { recursive: true });

await esbuild.build({
  entryPoints: ["webview/src/main.tsx"],
  bundle: true,
  outfile: "dist/webview/main.js",
  format: "iife",
  platform: "browser",
  target: "es2022",
  sourcemap: true,
  jsx: "automatic",
  loader: { ".tsx": "tsx", ".ts": "ts", ".css": "css" },
});

console.log("webview bundle → dist/webview/main.js");

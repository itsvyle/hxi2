#!/usr/bin/env node
// TODO: add a watch on scss files as well!
// TODO: clear output dir before watching, as there are unecessary/old files
import { BuildAssetInserter } from "../global-frontend-dependencies/_build-asset-inserter";
import * as esbuild from "esbuild";
import http from "node:http";
import { sassPlugin } from "esbuild-sass-plugin";
import path from "path";
import fs from "fs";
import { getEntries } from "./build-utils";
import { exec } from "child_process";
import { Transform } from "node:stream";
import mime from "mime";
import { AssertionError } from "node:assert";
import { assert } from "node:console";

const SERVE_PORT = 8749;
const staticsPath = "../static/assets";

const args = process.argv.slice(2);
const watch = args.includes("--watch");
const runHugo = watch && args.includes("--hugo");

function getEntrypoints(): Array<{ in: string; out: string }> {
    const entries = getEntries();
    const r: Array<{ in: string; out: string }> = [];
    for (const entry in entries) {
        r.push({ in: entries[entry], out: entry });

        if (entry.startsWith("shared/")) continue;

        const html = entries[entry].replace(/\.ts$/, ".html");
        if (fs.existsSync(html)) {
            r.push({ in: html, out: entry });
        }
    }
    return r;
}
const entrypoints = getEntrypoints();
console.log("esbuild entry points: ", entrypoints);

// TODO: Add a watch to scss files
function htmlReplacementPlugin(): esbuild.Plugin {
    return {
        name: "html-transform",
        setup(build) {
            const outdir = build.initialOptions.outdir;
            const assetInserter = new BuildAssetInserter({
                addHash: false,
                env: "development",
            });
            assetInserter.customReplacement.set(
                /<\/head>/g,
                `<script>new EventSource('/esbuild').addEventListener('change', () => window.location.reload())</script></head>`,
            );
            assetInserter.customReplacement.set(
                /<!--\s*dist-path\s*-->/gi,
                (ctx) => {
                    return "/dist/" + path.dirname(ctx.filePath);
                },
            );

            if (!outdir) {
                console.warn(
                    "[html-transform-plugin] build.initialOptions.outdir is not set. Plugin may not work as expected.",
                );
            }

            build.onLoad({ filter: /\.html$/ }, async (args) => {
                let content = await fs.promises.readFile(args.path, "utf-8");
                let { changed, source } = assetInserter.applyReplacements(
                    content,
                    args.path.substring(__dirname.length + 1),
                );
                return {
                    contents: source,
                    loader: "copy",
                };
            });
        },
    };
}

const config: Parameters<typeof esbuild.build>[0] = {
    bundle: true,
    entryPoints: entrypoints,
    outdir: path.join(__dirname, "dist"),
    plugins: [sassPlugin(), htmlReplacementPlugin()],
    outExtension: { ".js": ".bundle.js", ".css": ".bundle.css" },
    minify: false,
    sourcemap: true,
    tsconfig: path.join(__dirname, "../tsconfig.json"),
    define: {
        "window.isDev": "true",
        "window.domain": JSON.stringify("hxi2.fr"),
    },
};

//prettier-ignore
function openUrl(url) { const start = process.platform === 'darwin' ? 'open' : process.platform === 'win32' ? 'start' : 'xdg-open'; exec(`${start} ${url}`, (err) => { if (err) { console.error('Error opening URL:', err); } }); }

function getMimeType(path: string): string {
    return mime.getType(path);
}

async function runBuild() {
    if (!watch) return await esbuild.build(config);

    if (runHugo) {
        const prefixStream = (prefix) =>
            new Transform({
                transform(chunk, encoding, callback) {
                    const lines = chunk
                        .toString()
                        .split("\n")
                        .map((line) => (line ? `${prefix} ${line}` : ""))
                        .join("\n");
                    callback(null, lines);
                },
            });

        const hugoProcess = exec("hugo --watch --logLevel debug");
        hugoProcess.stdout?.pipe(prefixStream("[hugo]")).pipe(process.stdout);
        hugoProcess.stderr?.pipe(prefixStream("[hugo]")).pipe(process.stderr);
        process.on("exit", () => {
            hugoProcess.kill();
        });
    }

    const ctx = await esbuild.context(config);
    await ctx.watch();
    const serving = await ctx.serve({
        port: SERVE_PORT + 1,
        servedir: __dirname,
    });

    // Then start a proxy server on port 3000
    http.createServer((req, res) => {
        const options = {
            hostname: serving.hosts[0],
            port: serving.port,
            path: req.url,
            method: req.method,
            headers: req.headers,
        };

        const proxyReq = http.request(options, (proxyRes) => {
            // If esbuild returns "not found", send a custom 404 page
            if (
                proxyRes.statusCode === 404 &&
                proxyReq.path.startsWith("/static/")
            ) {
                let finalPath = path.resolve(
                    staticsPath,
                    proxyReq.path.substring("/static/".length),
                );
                if (fs.existsSync(finalPath)) {
                    res.writeHead(200, {
                        "Content-Type": getMimeType(finalPath),
                    });
                    fs.createReadStream(finalPath).pipe(res);
                    return;
                } else {
                    console.log("STATIC FILE NOT FOUND:", finalPath);
                }
            }

            res.writeHead(proxyRes.statusCode, proxyRes.headers);
            proxyRes.pipe(res, { end: true });
        });

        req.pipe(proxyReq, { end: true });
    }).listen(SERVE_PORT);

    console.log(
        `Serving on port ${SERVE_PORT} (http://localhost:${SERVE_PORT}), watching for changes in ${entrypoints.map((e) => e.in).join(", ")}`,
    );
    openUrl(`http://localhost:${SERVE_PORT}/dist/`);
}

runBuild();

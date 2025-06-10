import path from "path";
import "webpack-dev-server";
import { getDefaultConfig } from "../global-frontend-dependencies/_webpack-utils";
import fs from "fs";
import { getEntries } from "./build-utils";
import { BuildAssetInserter } from "../global-frontend-dependencies/_build-asset-inserter";
import { parseArgs } from "util";
import CopyPlugin from "copy-webpack-plugin";

export default (env, argv) => {
    if (!argv) {
        argv = { mode: "production" };
    } else if (!argv.mode) {
        argv.mode = "production";
    }
    if (argv.mode != "production") {
        throw new Error(
            "Development mode is not supported with webpack here, please run `just dev` (which uses esbuild) instead.",
        );
    }

    let assetInserter = new BuildAssetInserter({
        addHash: argv.mode === "production",
        env: argv.mode,
    });
    assetInserter.customReplacement.set(/<!--\s*dist-path\s*-->/gi, (ctx) => {
        return "/dist/" + path.dirname(ctx.filePath);
    });

    const entries = getEntries();

    let config = getDefaultConfig({
        mode: argv.mode,
        dirname: __dirname,
        devPort: 8749,
        devProxy: [],
        entries: entries,
        outputDirName: "dist",
        srcDir: "",
        assetInserter: assetInserter,
    });
    if (!config.plugins) config.plugins = [];

    let patterns: CopyPlugin.Pattern[] = [];

    for (const entry in entries) {
        if (entry.startsWith("shared/")) continue;
        let outpath = entries[entry];
        let htmlPath = outpath.replace(/\.ts$/, ".html");
        if (fs.existsSync(htmlPath)) {
            patterns.push({
                from: htmlPath,
                to: htmlPath.substring(__dirname.length + 1),
            });
        }
    }

    config.plugins.push(
        //@ts-expect-error
        new CopyPlugin({
            patterns: patterns,
        }),
    );

    return config;
};

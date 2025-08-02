import path from "path";
import fs from "fs";
import "webpack-dev-server";
import { getDefaultConfig } from "../global-frontend-dependencies/_webpack-utils";

const SRCDIR = path.resolve(__dirname, "frontend-src");

export default (env, argv) => {
    if (!argv) {
        argv = { mode: "production" };
    } else if (!argv.mode) {
        argv.mode = "production";
    }
    return getDefaultConfig({
        mode: argv.mode,
        dirname: __dirname,
        devPort: 8748,
        devProxy: [
            {
                context: ["/api"],
                target: "http://localhost:42005",
            },
        ],
        entries: {
            main: SRCDIR + "/main.ts",
            edit: SRCDIR + "/edit.ts",
        },
        extraDefines: {
            "window.activePromotion": fs
                .readFileSync("promo-active.txt", "utf-8")
                .trim(),
        },
        outputDirName: "dist",
        srcDir: SRCDIR,
    });
};

import path from "path";
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
                target: "http://localhost:8080",
            },
            {
                context: ["/auth.v2.AuthService"],
                target: "http://localhost:8080",
            },
        ],
        entries: {
            login: SRCDIR + "/login.ts",
            loginv2: SRCDIR + "/loginv2.ts",
        },
        outputDirName: "dist",
        srcDir: SRCDIR,
    });
};

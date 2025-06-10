import "webpack-dev-server";
import path from "path";
import express from "express";
import webpack from "webpack";
import zlib from "zlib";
import CompressionPlugin from "compression-webpack-plugin";
import MiniCssExtractPlugin from "mini-css-extract-plugin";
import HtmlMinimizerPlugin from "html-minimizer-webpack-plugin";
import CopyPlugin from "copy-webpack-plugin";
import tsLoader from "ts-loader";
import { EsbuildPlugin } from "esbuild-loader";
import { BuildAssetInserter } from "./_build-asset-inserter";
import { exec } from "child_process";

export function getDevServer(
    port: number,
    __dirname_: string,
    proxy: Array<any>,
    staticsPath: string,
) {
    return {
        liveReload: true,
        port: 8748,
        static: {
            directory: path.join(__dirname_, "/"), // the home of the dev server
        },
        compress: true,
        client: {
            overlay: true,
            progress: true,
            reconnect: true,
        },
        hot: false,
        devMiddleware: {
            index: true,
            publicPath: "/dist",
            writeToDisk: true, // doesn't work otherwise, idk why, but doesn't rly matter anyways
        },
        proxy: proxy,
        setupMiddlewares: (middlewares, server) => {
            const app = server.app!;
            app.set("json spaces", 2);
            middlewares.unshift({
                name: "static",
                path: "/static",
                middleware: express.static(staticsPath),
            });
            return middlewares;
        },
    };
}

export function chunkFilename(pathData, assetInfo) {
    if (!pathData.chunk || !pathData.chunk.id || !pathData.chunk.hash)
        throw new Error("Invalid pathData for chunkname generation");
    return `${pathData.chunk.id}_${pathData.chunk.hash}__hash__.chunk.js`;
}

export function pluginExitOnDone() {
    return {
        apply: (compiler: webpack.Compiler) => {
            compiler.hooks.afterDone.tap("DonePlugin", (stats) => {
                console.log("Compile is done! Exiting watch mode...");
                setTimeout(() => {
                    process.exit(0);
                });
            });
        },
    };
}

export function pluginCompression(
    outputDir: string,
): webpack.WebpackPluginInstance[] {
    const checkCommands = () => {
        const commands = ["fd", "gzip", "brotli"];
        const missingCommands = commands.filter((cmd) => {
            try {
                require("child_process").execSync(`${cmd} --version`);
                return false;
            } catch (e) {
                return true;
            }
        });

        return missingCommands.length === 0;
    };

    // Using my custom compression doesn't seem to gain any speed
    if (true || !checkCommands()) {
        return [
            new CompressionPlugin({
                ...defaultCompressionOptions,
                algorithm: "gzip",
                filename: "[path][base].gz",
            }),
            new CompressionPlugin({
                ...defaultCompressionOptions,
                algorithm: "brotliCompress",
                filename: "[path][base].br",
                compressionOptions: {
                    // @ts-expect-error
                    params: {
                        [zlib.constants.BROTLI_PARAM_QUALITY]: 11,
                    },
                },
            }),
        ];
    } else {
        return [
            {
                apply: (compiler: webpack.Compiler) => {
                    compiler.hooks.done.tapPromise(
                        "CustomCompressPlugin",
                        async (stats) => {
                            const commandFd = `fd --type f --exclude '*.br' --exclude '*.gz'`;
                            const commandGzip = `${commandFd} -x gzip -k -f`;
                            const commandBrotli = `${commandFd} -x brotli -k -f -q 11`;
                            const executeCommand = (
                                command: string,
                                name: string,
                            ) => {
                                return new Promise((resolve, reject) => {
                                    const startTime = Date.now();
                                    exec(
                                        command,
                                        { cwd: outputDir, maxBuffer: 0 },
                                        (
                                            error: any,
                                            stdout: any,
                                            stderr: any,
                                        ) => {
                                            if (error) {
                                                console.error(
                                                    `Error executing ${name}: ${error}`,
                                                );
                                                reject(error);
                                                return;
                                            }
                                            const endTime = Date.now();
                                            const executionTime =
                                                (endTime - startTime) / 1000;
                                            console.log(
                                                `${name} executed in ${executionTime}s`,
                                            );
                                            resolve(stdout);
                                        },
                                    );
                                });
                            };
                            try {
                                await Promise.all([
                                    executeCommand(commandGzip, "gzip"),
                                    executeCommand(commandBrotli, "brotli"),
                                ]);
                            } catch (e) {
                                console.error("Error during compression:", e);
                            }
                        },
                    );
                },
            } as webpack.WebpackPluginInstance,
        ];
    }
}

let defaultCompressionOptions = {
    threshold: 1000, // 1KB+
    minRatio: 0.8,
    deleteOriginalAssets: false,
};
export function getDefaultConfig({
    dirname,
    devProxy,
    devPort,
    entries,
    mode,
    outputDirName = "dist",
    srcDir,
    excludeFolders = [/(node_modules)/, /(mail)/, /(tests)/],
    generateCompressed = true,
    extraDefines = {},
    assetInserter,
}: {
    dirname: string;
    devProxy?: Array<any>;
    devPort: number;
    entries: Record<string, string>;
    mode: "development" | "production";
    outputDirName: string;
    srcDir: string;
    excludeFolders?: RegExp[];
    generateCompressed?: boolean;
    extraDefines?: Record<string, string>;
    assetInserter?: BuildAssetInserter;
}): webpack.Configuration {
    const loader = (s: string) => path.resolve(__dirname, "node_modules/" + s);
    const outputDir = path.resolve(dirname, outputDirName);
    let config: webpack.Configuration = {
        mode: mode,
        entry: entries,
        output: {
            filename: "[name].bundle.js",
            path: outputDir,
            clean: true,
            chunkFilename,
        },
        module: {
            rules: [
                {
                    test: /\.[jt]sx?$/,
                    use: loader("esbuild-loader"),
                    exclude: excludeFolders,
                },
                {
                    test: /\.s[ac]ss$/i,
                    use: [
                        // fallback to style-loader in development
                        mode === "production"
                            ? MiniCssExtractPlugin.loader
                            : loader("style-loader"),
                        loader("css-loader"),
                        loader("sass-loader"),
                    ],
                    exclude: excludeFolders,
                },
                {
                    test: /\.html$/i,
                    type: "asset/resource",
                    exclude: excludeFolders,
                },
            ],
        },
        resolve: {
            extensions: [".ts", ".js"],
        },
        plugins: [
            new webpack.DefinePlugin({
                "window.isDev": JSON.stringify(mode === "development"),
                "window.domain": JSON.stringify("hxi2.fr"),
                ...extraDefines,
            }),
            new MiniCssExtractPlugin({
                filename: "[name].bundle.css",
                chunkFilename: "[id].css",
            }),
            assetInserter ||
                new BuildAssetInserter({
                    addHash: mode === "production",
                    env: mode,
                }),
        ],
        devServer: getDevServer(
            devPort,
            dirname,
            devProxy || [],
            path.resolve(__dirname, "../static/assets/"),
        ),
    };
    if (srcDir) {
        config.plugins!.push(
            new CopyPlugin({
                patterns: [
                    {
                        from: srcDir + "/*.html",
                        to: "[name].html",
                        globOptions: {
                            ignore: ["**/components.html", "assets/**"],
                        },
                    },
                ],
            }),
        );
    }
    if (mode === "production") {
        config.optimization = {
            minimize: true,
            minimizer: [
                new EsbuildPlugin({
                    target: "ES6",
                    css: true,
                }),
                new HtmlMinimizerPlugin({
                    minify: HtmlMinimizerPlugin.swcMinify,
                    minimizerOptions: {},
                }),
            ],
        };
        if (generateCompressed) {
            config.plugins!.push(...pluginCompression(outputDir));
        }

        // https://stackoverflow.com/questions/71193896/how-to-let-webpack-exit-after-build-the-project
        config.plugins!.push(pluginExitOnDone());
    } else if (mode == "development") {
        config.devtool = "eval-cheap-module-source-map";
        config.stats = {
            ...(config.stats as object),
            // enable the logging on @debug in sass
            loggingDebug: ["sass-loader"],
        };
    }

    return config;
}

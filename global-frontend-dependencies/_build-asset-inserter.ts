import { Compilation } from "webpack";
import crypto from "crypto";

// <!-- favicon -->
const faviconRegex = /<!--\s*favicon\s*-->/i;
// <!-- iconify-import -->
const iconifyRegex = /<!--\s*iconify-import\s*-->/i;
// <!-- menu-import -->
const menuRegex = /<!--\s*menu-import\s*-->/i;

const scriptsRegex =
    /<script(?:(?!\s*\/?>)[^>])*?\ssrc\s*=\s*["']([^"']*\.js)["'](?:(?!\s*\/?>)[^>])*\s*>\s*<\/script>/g;
const stylesRegex =
    /<link(?:(?!\s*\/?>)[^>])*?\shref\s*=\s*["']([^"']*\.css)["'](?:(?!\s*\/?>)[^>])*\s*\/?>/g;
function getImgRegex() {
    return /<img(?:(?!\s*\/?>)[^>])*?\ssrc\s*=\s*["']([^"']*)["'](?:(?!\s*\/?>)[^>])*\s*\/?>/g;
}
function getHyperlinkRegex() {
    return /<a(?:(?!\s*\/?>)[^>])*?\shref\s*=\s*["']([^"']*)["'](?:(?!\s*\/?>)[^>])*\s*\/?>/g;
}

function getMetaContentRegex() {
    return /<meta\s+(?:name|property)=["'][^"']+["']\s+content=["']([^"']+)["']\s*\/?>/g;
}

function hashContent(content: string): string {
    // just do a md5 hash
    const hash = crypto.createHash("md5");
    hash.update(content);
    return hash.digest("hex");
}

const iconifyVersion = "2.3.0";
const HXI2_TLD = process.env.HXI2_TLD || "hxi2.fr";

const staticUrl = `https://static.${HXI2_TLD}`;

interface CustomReplacementCtx {
    filePath: string;
    matches: RegExpExecArray[];
}

class BuildAssetInserter {
    hashes: Record<string, string> = {};
    addHashes = false;
    htmlDistDir = "/dist";

    customReplacement: Map<
        RegExp,
        string | ((ctx: CustomReplacementCtx) => string)
    > = new Map();
    private env: "production" | "development" = "production";
    constructor(
        options: { addHash: boolean; env: "production" | "development" } = {
            addHash: false,
            env: "production",
        },
    ) {
        this.env = options.env;
        this.addHashes = options.addHash;
    }

    apply(compiler) {
        compiler.hooks.compilation.tap(
            "BuildAssetInserter",
            (compilation: Compilation) => {
                compilation.hooks.processAssets.tap(
                    {
                        name: "BuildAssetInserter",
                        stage: Compilation.PROCESS_ASSETS_STAGE_PRE_PROCESS,
                    },
                    (assets) => {
                        let assetFiles = Object.keys(assets);
                        if (this.addHashes) {
                            for (let file of assetFiles) {
                                if (
                                    file.endsWith(".js") ||
                                    file.endsWith(".css")
                                ) {
                                    const content = assets[file].source();
                                    if (typeof content === "string") {
                                        const hash = hashContent(content);
                                        this.hashes[file] = hash;
                                    } else if (Buffer.isBuffer(content)) {
                                        const hash = hashContent(
                                            content.toString("utf-8"),
                                        );
                                        this.hashes[file] = hash;
                                    }
                                }
                            }
                        }

                        for (let file of assetFiles) {
                            if (!file.endsWith(".html")) {
                                continue;
                            }

                            let source = assets[file].source();
                            if (Buffer.isBuffer(source)) {
                                source = source.toString("utf-8");
                            }

                            if (typeof source !== "string") continue;

                            const { changed, source: newSource } =
                                this.applyReplacements(source, file);
                            if (changed) {
                                //@ts-expect-error
                                assets[file] = {
                                    source: () => newSource,
                                    size: () => newSource.length,
                                };
                            }
                        }
                    },
                );
            },
        );
    }

    applyReplacements(
        source: string,
        filePath: string,
    ): {
        changed: boolean;
        source: string;
    } {
        let changed = false;
        if (faviconRegex.test(source)) {
            if (this.env === "development") {
                source = source.replace(
                    faviconRegex,
                    `<link rel="icon" href="/static/favicon.webp" />`,
                );
            } else {
                source = source.replace(
                    faviconRegex,
                    `<link rel="icon" href="${staticUrl}/favicon.webp" />`,
                );
            }
            changed = true;
        }
        if (menuRegex.test(source)) {
            source = source.replace(
                menuRegex,
                `<script src="static:dist/menu.bundle.js"></script><link rel="stylesheet" href="static:dist/menu.bundle.css" />`,
            );
            changed = true;
        }
        if (iconifyRegex.test(source)) {
            source = source.replace(
                iconifyRegex,
                `<script src="https://cdn.jsdelivr.net/npm/iconify-icon@${iconifyVersion}/dist/iconify-icon.min.js"></script>`,
            );
            changed = true;
        }

        const applyStaticReplacements = (matches: RegExpExecArray[]) => {
            for (let replacement of matches) {
                const fullTag = replacement[0];
                const path = replacement[1];
                if (!fullTag || !path) continue;
                if (!path.startsWith("static:")) continue;
                let newPath;
                if (this.env === "production") {
                    newPath = path.replace("static:", staticUrl + "/");
                } else {
                    newPath = path.replace("static:", `/static/`);
                }

                source = source.replace(
                    fullTag,
                    fullTag.replace(path, newPath),
                );
                changed = true;
            }
        };

        for (let f of [getImgRegex, getHyperlinkRegex, getMetaContentRegex]) {
            if (f().test(source)) {
                const reg = f();
                const matches = [];
                let match;
                while ((match = reg.exec(source))) {
                    matches.push([match[0], match[1]]);
                }
                applyStaticReplacements(matches);
            }
        }

        for (let [expr, replaceWith] of this.customReplacement) {
            const matches = Array.from(source.matchAll(expr));
            if (matches.length > 0) {
                changed = true;
                if (typeof replaceWith !== "string") {
                    replaceWith = replaceWith({
                        filePath: filePath,
                        matches: matches,
                    });
                }
                source = source.replaceAll(expr, replaceWith as string);
            }
        }

        {
            const scriptMatches = Array.from(source.matchAll(scriptsRegex));
            const styleMatches = Array.from(source.matchAll(stylesRegex));
            const allMatches = [...scriptMatches, ...styleMatches];

            for (let replacement of allMatches) {
                const fullTag = replacement[0];
                let path = replacement[1];
                let newTag = "";
                if (!fullTag || !path) continue;

                if (path.startsWith("static:")) {
                    if (this.env === "production") {
                        path = path.replace("static:", staticUrl + "/");
                    } else {
                        path = path.replace("static:", `/static/`);
                    }
                    newTag = fullTag.replace(replacement[1], path);
                }

                if (this.addHashes && path.startsWith(this.htmlDistDir)) {
                    const hash =
                        this.hashes[
                            path.substring(this.htmlDistDir.length + 1)
                        ]; // +1 for the extra /
                    if (hash) {
                        newTag = fullTag.replace(
                            path,
                            path + "?" + hash + "__hash__",
                        );
                    } else {
                        console.warn(`No hash found for ${path}`);
                    }
                }
                if (newTag !== "") {
                    source = source.replace(fullTag, newTag);
                    changed = true;
                }
            }
        }

        return { changed, source };
    }
}
export { BuildAssetInserter };

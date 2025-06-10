import path from "path";
import fs from "fs";
import SectionsJson from "./sections.json";
export function getEntries(): Record<string, string> {
    const WARN = "\x1b[33m";
    const RESET = "\x1b[0m";

    const Sections = SectionsJson as Record<string, Record<string, string>>;
    for (const section in Sections) {
        const dir = path.resolve(__dirname, section);
        if (!fs.existsSync(dir)) {
            console.warn(
                `${WARN}[hxi2-pages-sections] Directory ${dir} doesn't exist, will not compile ${section}${RESET}`,
            );
            delete Sections[section];
            continue;
        }
        for (const file in Sections[section]) {
            const filePath = path.resolve(dir, Sections[section][file]);
            if (!fs.existsSync(filePath)) {
                console.warn(
                    `${WARN}[hxi2-pages-sections] File ${filePath} doesn't exist, will not compile file ${file} in ${section}${RESET}`,
                );
                delete Sections[section][file];
                continue;
            }
            Sections[section][file] = filePath;
        }
    }

    const compile_section =
        "PAGES_SECTION" in process.env ? process.env.PAGES_SECTION : null;
    if (compile_section && !(compile_section in Sections)) {
        console.error(
            `Section ${compile_section} not found in sections.json, available sections are: ${Object.keys(
                Sections,
            ).join(", ")}`,
        );
        process.exit(1);
    }
    const entries = {};
    for (const section in Sections) {
        if (compile_section && section !== compile_section) {
            continue;
        }
        for (const file in Sections[section]) {
            const filePath = Sections[section][file];
            const fileName = path.join(
                section,
                path.basename(filePath, path.extname(filePath)),
            );
            entries[fileName] = filePath;
        }
    }

    return entries;
}

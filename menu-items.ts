import { MenuItem, MenuSection } from "./global-frontend-dependencies/iMenu";
import { Perms } from "./global-frontend-dependencies/perms";

export const menuSections: Record<string, MenuSection> = {
    home: {
        label: "Home",
        requirePerms: 0,
        path: "//hxi2.fr/",
    },
    pulls: {
        label: "Pulls de classe",
        requirePerms: 0,
        path: "//hxi2.fr/pulls",
    },
    tree: {
        label: "Tree",
        path: "//tree.hxi2.fr/tree",
        requirePerms: Perms.Student | Perms.Admin,
    },
    parrainsup: {
        label: "Parrainsup",
        path: "//parrainsup.hxi2.fr/",
        requirePerms: Perms.Student | Perms.Admin,
    },
};

export const menuSectionChildren: Record<string, Array<MenuItem>> = {};

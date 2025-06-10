import "./tree.scss";
import { FuzzyInputList } from "../../global-frontend-dependencies/ui_fuzzy_input";
import {
    CustomWindow,
    getGlobalTree,
    getUserTree,
    GlobalTreeResponse,
    listUsers,
    listUsersMap,
    OtherUser,
    otherUserDisplayName,
} from "./api";
import {
    ICustomTab,
    UITabs,
    UITabsTab,
} from "../../global-frontend-dependencies/ui_tabs";
import {
    fillWindowUserData,
    getLocalUserData,
} from "../../global-frontend-dependencies/authUtils";
import Panzoom, { PanzoomEvent, PanzoomObject } from "@panzoom/panzoom";
import loadingManager from "../../global-frontend-dependencies/ui_loader";
import dialog from "../../global-frontend-dependencies/ui_dialog";

interface FuzzyWindow extends CustomWindow {
    fu: FuzzyInputList<OtherUser>;
}
declare var window: FuzzyWindow;

class SpecificStudentTab implements ICustomTab {
    id: string;
    initialTitle: string;
    tab: UITabsTab;
    closable = true;
    element: HTMLDivElement;
    innerElement: HTMLDivElement;
    pz: PanzoomObject | undefined;
    constructor(studentID: number) {
        this.id = `student-${studentID}`;
        this.initialTitle = otherUserDisplayName(allUsers.get(studentID));

        this.tab = new UITabsTab(this);
        this.element = document.createElement("div");
        this.element.classList.add("student-tab");
        document.getElementById("main-view")?.appendChild(this.element);

        this.innerElement = document.createElement("div");
        this.innerElement.classList.add("student-tab-inner");
        this.element.appendChild(this.innerElement);
    }

    onSelect() {
        this.element.classList.toggle("visible", true);
    }
    onUnselect() {
        this.element.classList.toggle("visible", false);
    }
    onClose() {
        this.pz?.destroy();
        this.element.remove();
    }
}

let mainTabs: UITabs | undefined = undefined;
let allUsers: Map<number, OtherUser> | undefined = undefined;
document.addEventListener("DOMContentLoaded", async () => {
    fillWindowUserData();
    registerSidebar();

    document.getElementById("add-button").addEventListener("click", () => {
        window.location.href = "/add";
    });

    mainTabs = document.getElementById("main-tabs") as UITabs;
    try {
        loadingManager.show();
        await Promise.all([
            displayGlobalTree(),

            listUsersMap()
                .then((users) => {
                    allUsers = users;
                    const g = (item: OtherUser) => {
                        return otherUserDisplayName(item);
                    };
                    window.fu = new FuzzyInputList(
                        Array.from(allUsers.values()),
                        {
                            getItemID: (item) => {
                                return String(item.id);
                            },
                            getItemText: g,
                            fuzzyOptions: {
                                getText: (item) => {
                                    return [g(item), item.username];
                                },
                            },
                        },
                    );
                    let searchInput = document.getElementById(
                        "search-user",
                    ) as HTMLInputElement;
                    window.fu.addWatcher(searchInput);
                    searchInput.addEventListener(
                        "fuzzyResultChose",
                        (e: CustomEvent) => {
                            const id = parseInt(e.detail.id);
                            if (allUsers.has(id)) {
                                onUserClick(id);
                            } else {
                                throw new Error(
                                    `User with id ${id} not found in allUsers`,
                                );
                            }
                        },
                    );
                })
                .catch(console.error),
        ]);
    } finally {
        loadingManager.hide();
    }
});

function registerSidebar() {
    const sidebar = document.getElementById("sidebar");
    if (!sidebar) throw new Error("No sidebar found");
    document
        .getElementById("sidebar-header-toggle")
        .addEventListener("click", () => {
            sidebar.classList.toggle("hidden");
        });
}

let globalTree: GlobalTreeResponse;
let globalPanzoom: ReturnType<typeof Panzoom>;
let globalPanSize: {
    width: number;
    height: number;
};

function treeClickEvent(treeDiv: HTMLDivElement, e: MouseEvent) {
    if (!e.target) return;
    let t = e.target as HTMLElement;
    let tc = t.classList;
    let i = 0;

    while (
        t &&
        t !== treeDiv &&
        !tc.contains("node") &&
        !tc.contains("nodes") &&
        !tc.contains("clickable") &&
        i++ < 10
    ) {
        t = t.parentElement;
        tc = t.classList;
    }
    if (!tc.contains("clickable") || !t.id) return;
    let ids = t.id.split("-");
    if (ids.length < 2) return;
    let id = parseInt(ids[1]);
    if (!allUsers.has(id)) return;
    onUserClick(id);
}

async function displayGlobalTree() {
    let [tree, { default: Panzoom }] = await Promise.all([
        getGlobalTree(),
        import("@panzoom/panzoom"),
    ]);
    globalTree = tree;

    if (!globalTree) throw new Error("No tree found");
    let treeDiv = document.getElementById("main-tree");
    if (!treeDiv) throw new Error("No tree div found");

    treeDiv.innerHTML = globalTree.svg;
    globalPanzoom = addPanzoom(treeDiv as HTMLDivElement);

    const svg = treeDiv.querySelector("svg")!;
    globalPanSize = {
        width: svg.getBBox().width,
        height: svg.getBBox().height,
    };

    //@ts-expect-error
    window.pan = globalPanzoom;
}

function addPanzoom(treeDiv: HTMLDivElement): PanzoomObject {
    treeDiv.addEventListener("click", treeClickEvent.bind(null, treeDiv));
    const pz = Panzoom(treeDiv, {
        maxScale: 15,
    });
    treeDiv.parentElement.addEventListener("wheel", pz.zoomWithWheel);

    // Pan to center
    setTimeout(() => {
        const container = treeDiv.parentElement;

        const containerRect = container.getBoundingClientRect();
        const contentRect = treeDiv.getBoundingClientRect();

        const scale = pz.getScale();

        const scaledWidth = contentRect.width * scale;
        const scaledHeight = contentRect.height * scale;

        const x = (containerRect.width - scaledWidth) / 2;
        const y = (containerRect.height - scaledHeight) / 2;

        pz.pan(x, y);
    }, 1);

    return pz;
}

let wasMermaidInitialized = false;
async function onUserClick(id: number) {
    if (
        !globalTree ||
        !globalPanzoom ||
        !globalPanSize ||
        !(id in globalTree.elements)
    )
        return dialog.error("User not found in tree");
    const tabID = `student-${id}`;
    let tab: SpecificStudentTab | undefined;
    let firstInit = false;
    if (mainTabs!.hasTab(tabID)) {
        tab = mainTabs!.tabs.get(tabID).customBackend as SpecificStudentTab;
        tab.tab.select();
    } else {
        tab = new SpecificStudentTab(id);
        mainTabs!.insertTab(tab.tab);
        tab.tab.select();
        firstInit = true;
    }
    try {
        loadingManager.show();
        const [mermaidCode, { default: mermaid }, { default: elkLayouts }] =
            await Promise.all([
                getUserTree(id),
                import("mermaid"),
                import("@mermaid-js/layout-elk"),
            ]);
        if (!wasMermaidInitialized) {
            mermaid.registerLayoutLoaders(elkLayouts);
            // @ts-expect-error
            mermaid.initialize(globalTree.mermaidConfig);
            wasMermaidInitialized = true;
        }
        const { svg, bindFunctions } = await mermaid.render(
            "student-graph-" + id,
            mermaidCode,
        );
        tab.innerElement.innerHTML = svg;

        if (firstInit) {
            tab.pz = addPanzoom(tab.innerElement);
        }
    } finally {
        loadingManager.hide();
    }
}
//@ts-expect-error
window.scro = onUserClick;

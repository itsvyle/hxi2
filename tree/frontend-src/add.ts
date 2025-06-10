import "./add.scss";
import { LitElement, html } from "lit-element";
import { property, customElement } from "lit-element/decorators.js";
import { IUIDialog } from "../../global-frontend-dependencies/ui_interfaces";
import dialog from "../../global-frontend-dependencies/ui_dialog";
import {
    createRelation,
    CustomWindow,
    deleteRelation,
    listRelations,
    listUsers,
    listUsersMap,
    OtherUser,
    Relation,
} from "./api";
import { FuzzyInputList } from "../../global-frontend-dependencies/ui_fuzzy_input";
import loadingManager from "../../global-frontend-dependencies/ui_loader";
import { loadEnvFile } from "process";
import {
    fillWindowUserData,
    getLocalUserData,
} from "../../global-frontend-dependencies/authUtils";
import { Perms } from "../../global-frontend-dependencies/perms";
export interface CustomAddWindow extends CustomWindow {
    isDev: boolean;
    Dialog: IUIDialog;
    myID: number;
}
declare const window: CustomAddWindow;

@customElement("current-relation")
class ElemCurrentRelation extends LitElement {
    @property({ type: Number, attribute: true })
    relationID: number;
    @property({ type: String, attribute: true })
    relationName?: string = undefined;

    constructor(relation?: Relation) {
        super();
        if (!relation) return;
        this.relationID = relation.ID;
        if (window.myID == relation.parrainID) {
            const filleul = allUsers?.get(relation.filleulID);
            if (!filleul) {
                this.relationName = "Utilisateur inconnu";
            } else {
                this.relationName = `${filleul.firstName}${filleul.lastName.Valid ? " " + filleul.lastName.String : ""} [${filleul.promotion}]`;
            }
        } else if (window.myID == relation.filleulID) {
            const parrain = allUsers?.get(relation.parrainID);
            if (!parrain) {
                this.relationName = "Utilisateur inconnu";
            } else {
                this.relationName = `${parrain.firstName}${parrain.lastName.Valid ? " " + parrain.lastName.String : ""} [${parrain.promotion}]`;
            }
        } else {
            const filleul = allUsers?.get(relation.filleulID);
            const parrain = allUsers?.get(relation.parrainID);
            if (!filleul || !parrain) {
                this.relationName = "Utilisateur inconnu";
            } else {
                this.relationName = `${parrain.firstName}${parrain.lastName.Valid ? " " + parrain.lastName.String : ""} [${parrain.promotion}] ==> ${filleul.firstName}${filleul.lastName.Valid ? " " + filleul.lastName.String : ""} [${filleul.promotion}]`;
            }
        }
    }
    createRenderRoot() {
        return this; // Render template in light DOM instead of shadow dom
    }
    render() {
        /* <div class="option" style="color: #00a2ad">
                <iconify-icon icon="mdi:edit" title="Edit"></iconify-icon>
            </div> */
        return html`<div class="current-relation">
            <span class="name">${this.relationName}</span>
            <div
                class="option"
                style="color: var(--red-color)"
                @click=${this._handleRemoveClick}>
                <iconify-icon
                    icon="mdi:trash-can-outline"
                    title="Delete"></iconify-icon>
            </div>
        </div>`;
    }

    private _handleRemoveClick(e: MouseEvent) {
        if (e.shiftKey) {
            return this.dispatchEvent(
                new CustomEvent("elementRemove", {
                    detail: this,
                }),
            );
        }
        dialog.confirm(
            `Supprimer la relation avec '${this.relationName}'?`,
            () => {
                this.dispatchEvent(
                    new CustomEvent("elementRemove", {
                        detail: this,
                    }),
                );
            },
        );
    }
}

var usersFuzzyFinding: FuzzyInputList<OtherUser> | null = null;
var allUsers: Map<number, OtherUser> | null = null;
var allRelations: Map<number, Relation> | null = null;

var isAdmin = false;

document.addEventListener("DOMContentLoaded", async () => {
    loadingManager.show();
    fillWindowUserData();
    window.myID = window.userData.userID;
    isAdmin = (window.userData.permissions & Perms.Admin) === 8;

    document.getElementById("back").addEventListener("click", () => {
        window.location.href = "/tree";
    });

    document.getElementById("add-parrain").addEventListener("click", () => {
        openAddModal("parrain");
    });
    document.getElementById("add-filleul").addEventListener("click", () => {
        openAddModal("filleul");
    });

    if (isAdmin) {
        document.getElementById("select-my-user-container").style.display = "";
    }

    try {
        [allUsers, allRelations] = await Promise.all([
            listUsersMap(),
            listRelations(),
        ]);
    } catch (e) {
        dialog.error(e, true);
        return;
    } finally {
        loadingManager.hide();
    }

    const g = (item: OtherUser) => {
        return `${item.firstName}${item.lastName.Valid ? " " + item.lastName.String : ""} [${item.promotion}]`;
    };
    usersFuzzyFinding = new FuzzyInputList(Array.from(allUsers.values()), {
        getItemID: (item) => {
            return String(item.id);
        },
        getItemText: g,
        fuzzyOptions: {
            getText: (item) => {
                return [g(item), item.username];
            },
        },
    });

    fillLists();

    if (isAdmin) {
        let resetMyUser = () => {
            window.myID = window.userData.userID;
            (document.getElementById("my-user") as HTMLInputElement).value =
                usersFuzzyFinding.getItemText(allUsers.get(window.myID)!);
            fillLists();
        };

        (
            document.getElementById("select-my-user") as HTMLButtonElement
        ).addEventListener("click", () => {
            const sel = usersFuzzyFinding.getSelectedID("my-user");
            if (!sel) return dialog.error("Utilisateur invalide", false);
            window.myID = parseInt(sel);
            fillLists();
        });

        document
            .getElementById("reset-my-user")
            .addEventListener("click", resetMyUser);

        usersFuzzyFinding.addWatcher(
            document.getElementById("my-user") as HTMLInputElement,
        );
        const t = usersFuzzyFinding.getItemText(allUsers.get(window.myID))!;
        (document.getElementById("my-user") as HTMLInputElement).value = t;

        usersFuzzyFinding.watchingInputs["my-user"].latestSelectedItemID =
            String(window.myID);
        usersFuzzyFinding.watchingInputs["my-user"].latestSelectedItemText = t;
    }
});

function handleRemoveRelation(e: CustomEvent) {
    const element = e.detail as ElemCurrentRelation;
    if (!element || !element.relationID) return;

    loadingManager.show();
    deleteRelation(element.relationID)
        .then((a) => {
            if (a.success) {
                element.remove();
            }
        })
        .catch((e) => {
            dialog.error(e, false);
        })
        .finally(() => loadingManager.hide());
}

function fillLists() {
    const parrainsContainer = document.getElementById("current-list-parrains")!;
    const filleulsContainer = document.getElementById("current-list-filleuls")!;

    parrainsContainer.innerHTML = "";
    filleulsContainer.innerHTML = "";

    document.getElementById("title-parrains").innerText =
        window.userData.userID === window.myID
            ? "Mes parrains"
            : `Parrains de ${usersFuzzyFinding.getItemText(allUsers?.get(window.myID)!)}`;
    document.getElementById("title-filleuls").innerText =
        window.userData.userID === window.myID
            ? "Mes bizuths"
            : `Bizuths de ${usersFuzzyFinding.getItemText(allUsers?.get(window.myID)!)}`;

    for (let [_, re] of allRelations) {
        if (re.parrainID !== window.myID && re.filleulID !== window.myID)
            continue;
        const t = new ElemCurrentRelation(re);
        t.addEventListener("elementRemove", handleRemoveRelation);
        if (re.parrainID === window.myID) {
            filleulsContainer.appendChild(t);
        } else {
            parrainsContainer.appendChild(t);
        }
    }
}

type RelationType = "parrain" | "filleul";

function openAddModal(target: RelationType) {
    const container = dialog.openEmpty({
        title:
            "Ajouter " + (target === "filleul" ? "un filleul" : "un parrain"),
        message: "",
        buttons: [
            {
                text: "Annuler",
                className: "_dialog-button-cancel",
            },
            {
                bgColor: "var(--bccent-color)",
                text: "Ajouter",
                onclick: (e) => {
                    const sel =
                        usersFuzzyFinding.getSelectedID("add-fuzzy-searcher");
                    if (!sel) return;
                    addRelation(target, parseInt(sel));
                    if (e.shiftKey) {
                        setTimeout(() => {
                            openAddModal(target);
                        }, 1);
                    }
                },
            },
        ],
    });

    const input = document.createElement("input");
    input.type = "text";
    input.placeholder = "Prénom/nom d'utilisateur:";
    input.classList.add("add-fuzzy-searcher");
    input.id = "add-fuzzy-searcher";

    usersFuzzyFinding.addWatcher(input);

    container.appendChild(input);
}

function addRelation(rt: RelationType, otherID: number) {
    if (otherID < 1) {
        return;
    }
    const parrainID = rt === "parrain" ? otherID : window.myID;
    const filleulID = rt === "filleul" ? otherID : window.myID;

    loadingManager.show();
    createRelation(parrainID, filleulID)
        .then((r) => {
            if ("success" in r && !r.success) {
                dialog.error(r.error, false);
                return;
            }
            r = r as Relation;
            if (!r.ID || !r.filleulID || !r.parrainID)
                return console.error("Invalid relation returned", r);

            allRelations?.set(r.ID, r);
            fillLists();
        })
        .catch((e) => dialog.error(e, false))
        .finally(() => loadingManager.hide());
}

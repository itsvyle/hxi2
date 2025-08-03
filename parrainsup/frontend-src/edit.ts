import "./edit.scss";
import { fillWindowUserData } from "../../global-frontend-dependencies/authUtils";
import { CustomWindow } from "../../tree/frontend-src/api";
import { getMyself, listUsers, MainUser, updateMyself } from "./api";
import { MainUserCard } from "./user-card";
import dialog from "../../global-frontend-dependencies/ui_dialog";
import loadingManager from "../../global-frontend-dependencies/ui_loader";
export interface CustomAddWindow extends CustomWindow {
    activePromotion: number;
}
declare const window: CustomAddWindow;

let originalData: MainUser | null = null;
let newData: MainUser | null = null;
let previewCard: MainUserCard | null = null;

document.addEventListener("DOMContentLoaded", () => {
    fillWindowUserData();
    if (
        !window.userData ||
        window.userData.promotion !== window.activePromotion
    ) {
        dialog.error(
            "Vous n'êtes pas autorisé à modifier vos informations car votre promotion n'est pas sur parrainsup.",
            true,
        );
    }

    document.getElementById("return-button")?.addEventListener("click", () => {
        window.location.href = `/`;
    });

    getMyself().then((myself) => {
        originalData = myself;
        newData = { ...myself };
        previewCard = new MainUserCard(newData);
        document.getElementById("preview-container")?.appendChild(previewCard);

        fillForm(newData);

        if (!newData.display_name) {
            dialog.success(
                "Bienvenue sur votre page de modification de profil ! Commencez par remplir votre nom, puis les autres informations que vous souhaitez partager avec les bizuths.",
            );
        }
    });

    let saveButton = document.getElementById("save-container");
    saveButton.addEventListener("click", () => {
        saveChanges();
    });

    for (let field of [
        "display_name",
        "surnom",
        "origine",
        "voeu",
        "c_or_ocaml",
        "fun_fact",
        "conseil",
        "algebre_or_analyse",
        "pronouns",
        "couleur",
        "linux_distro",
    ] as (keyof MainUser)[]) {
        let input = document.getElementById(
            `field-${field}`,
        ) as HTMLInputElement;
        if (!input) {
            console.warn(`Input field for ${field} not found.`);
            continue;
        }
        input.addEventListener("input", () => {
            if (!newData || !previewCard) return;
            if (originalData) {
                // @ts-expect-error
                newData[field] = input.value;
                previewCard!.updateData(newData);
                saveButton!.classList.toggle("visible", hasChanged());
            }
        });
    }

    document.getElementById("field-hide")?.addEventListener("change", (e) => {
        if (!newData || !previewCard) return;
        newData.hide = (e.target as HTMLInputElement).checked;
        previewCard.updateData(newData);
        saveButton!.classList.toggle("visible", hasChanged());
    });

    window.addEventListener("beforeunload", (e) => {
        if (hasChanged()) {
            e.preventDefault();
            e.returnValue = "";
        }
    });
});

function fillForm(data: MainUser) {
    let g = (id: string) => document.getElementById(id) as HTMLInputElement;
    g("field-hide").checked = data.hide;
    g("field-display_name").value = data.display_name;
    g("field-surnom").value = data.surnom;
    g("field-pronouns").value = data.pronouns;
    g("field-couleur").value = data.couleur || "#000000";
    g("field-origine").value = data.origine;
    g("field-voeu").value = data.voeu;
    g("field-c_or_ocaml").value = data.c_or_ocaml;
    g("field-algebre_or_analyse").value = data.algebre_or_analyse;
    g("field-fun_fact").value = data.fun_fact;
    g("field-conseil").value = data.conseil;
    g("field-linux_distro").value = data.linux_distro;
}

function hasChanged(): boolean {
    if (!originalData || !newData) return false;
    for (let key of Object.keys(originalData) as (keyof MainUser)[]) {
        if (originalData[key] !== newData[key]) {
            return true;
        }
    }
    return false;
}

function saveChanges() {
    loadingManager.show();
    if (!newData) {
        dialog.error("Aucune donnée à enregistrer.", false);
        loadingManager.hide();
        return;
    }
    let suc = (updatedData: MainUser) => {
        originalData = updatedData;
        newData = { ...updatedData };
        previewCard?.updateData(newData);
        fillForm(newData);
        dialog.success("Vos données ont été mises à jour avec succès.");
        document.getElementById("save-container")!.classList.remove("visible");
    };
    updateMyself(newData)
        .then((updatedData) => {
            suc(updatedData);
        })
        .catch((error) => {
            if (typeof error === "string") {
                if (error.includes("Some fields are restricted")) {
                    newData = null;
                    dialog.error(
                        "Certains champs sont restreints et ne peuvent pas être modifiés - veuillez recharger la page.",
                        true,
                    );
                    return;
                }
            }
            console.error("Erreur lors de la mise à jour des données :", error);
            dialog.error(
                "Une erreur est survenue lors de la mise à jour: " + error,
                false,
            );
        })
        .finally(() => {
            loadingManager.hide();
        });
}

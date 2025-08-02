import { fillWindowUserData } from "../../global-frontend-dependencies/authUtils";
import { CustomWindow } from "../../tree/frontend-src/api";
import { listUsers } from "./api";
import "./main.scss";
import { MainUserCard } from "./user-card";
export interface CustomAddWindow extends CustomWindow {
    activePromotion: number;
}
declare const window: CustomAddWindow;

document.addEventListener("DOMContentLoaded", () => {
    fillWindowUserData();
    if (
        !window.userData ||
        window.userData.promotion !== window.activePromotion
    ) {
        document.getElementById("edit-button").style.display = "none";
    } else {
        document
            .getElementById("edit-button")
            ?.addEventListener("click", () => {
                window.location.href = `edit`;
            });
    }
    listUsers().then((users) => {
        for (const userId in users) {
            const user = users[userId];
            const card = new MainUserCard(user);
            document.querySelector("#cards")?.appendChild(card);
        }
    });
});

import { fillWindowUserData } from "../../global-frontend-dependencies/authUtils";
import { CustomWindow } from "../../tree/frontend-src/api";
import { listUsers } from "./api";
import "./main.scss";
import { MainUserCard } from "./user-card";
declare const window: CustomWindow;

document.addEventListener("DOMContentLoaded", () => {
    fillWindowUserData();
    if (!window.userData || window.userData.promotion !== 2024) {
        document.getElementById("edit-button").style.display = "none";
    }
    listUsers().then((users) => {
        for (const userId in users) {
            const user = users[userId];
            const card = new MainUserCard(user);
            document.querySelector("#cards")?.appendChild(card);
        }
    });
});

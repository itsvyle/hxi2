import {
    fillWindowUserData,
    getLocalUserData,
} from "../../../global-frontend-dependencies/authUtils";
import { menuSections, menuSectionChildren } from "../../../menu-items";
import { CustomWindow } from "../../../tree/frontend-src/api";
import "./menu.scss";
declare var window: CustomWindow;

let menuContainer: HTMLDivElement | null = null;
let menuElements: HTMLDivElement | null = null; // This will store the reference to the div with class "menu-elements"
let menuButtonBar: HTMLDivElement | null = null; // This will store the reference to the div with class "menu-button-bar"

let currentMenuSection: string = "";
let selectedMenuElementPath: string | undefined;

function drawMenu() {
    if (!menuContainer) {
        console.error("Menu container is not initialized. Cannot draw menu.");
        return;
    }

    // Clear any existing content from menuContainer to prevent duplication
    // if drawMenu is called multiple times. This uses DOM methods, not innerHTML.
    while (menuContainer.firstChild) {
        menuContainer.removeChild(menuContainer.firstChild);
    }

    if (
        menuContainer.classList.contains("integrated-bar") &&
        !document.getElementById("menu-button")
    ) {
        // Create the menu button if it doesn't exist
        const menuButton = document.createElement("div");
        menuButton.id = "menu-button";
        menuContainer.appendChild(menuButton);
        initButton();
    }

    // Create menu-head (as in the original code)
    const menuHead = document.createElement("div");
    menuHead.classList.add("menu-head");

    const closeButton = document.createElement("div");
    closeButton.classList.add("close-button");
    closeButton.setAttribute("role", "button");

    // const closeIconClose = document.createElement(
    //     "iconify-icon",
    // ) as HTMLElement; // Cast for setAttribute
    // closeIconClose.classList.add("close-icon-close");
    // closeIconClose.setAttribute("icon", "mdi:hamburger-open");
    // closeIconClose.setAttribute("width", "35");
    // closeIconClose.setAttribute("height", "35");
    // closeButton.appendChild(closeIconClose);

    const closeIconOpen = document.createElement("iconify-icon") as HTMLElement; // Cast for setAttribute
    closeIconOpen.classList.add("close-icon-open");
    closeIconOpen.setAttribute("icon", "mdi:hamburger-close");
    closeIconOpen.setAttribute("width", "35");
    closeIconOpen.setAttribute("height", "35");
    closeButton.appendChild(closeIconOpen);

    menuHead.appendChild(closeButton);
    menuContainer.appendChild(menuHead);

    const sectionsHostDiv = document.createElement("div");
    sectionsHostDiv.classList.add("menu-elements");

    for (const sectionKey in menuSections) {
        if (!Object.prototype.hasOwnProperty.call(menuSections, sectionKey))
            continue;
        const sectionData = menuSections[sectionKey];
        if (
            sectionData.requirePerms !== 0 &&
            (!window.userData ||
                (window.userData.permissions & sectionData.requirePerms) === 0)
        )
            continue;

        if (sectionData.path) {
            const menuItemAnchor = document.createElement("a");
            menuItemAnchor.classList.add("menu-element", "as-section");
            menuItemAnchor.href = sectionData.path;
            menuItemAnchor.setAttribute("role", "button");
            menuItemAnchor.setAttribute("aria-label", sectionData.label);

            if (selectedMenuElementPath === sectionData.path) {
                menuItemAnchor.classList.add("selected");
            }

            const itemSpan = document.createElement("span");
            itemSpan.textContent = sectionData.label;
            menuItemAnchor.appendChild(itemSpan);

            const expandMoreIcon = document.createElement(
                "iconify-icon",
            ) as HTMLElement;
            expandMoreIcon.setAttribute("icon", "mdi:menu-right");
            expandMoreIcon.setAttribute("width", "24");
            expandMoreIcon.setAttribute("height", "24");
            menuItemAnchor.appendChild(expandMoreIcon);

            sectionsHostDiv.appendChild(menuItemAnchor);
            continue;
        }

        const menuSectionDiv = document.createElement("div");
        menuSectionDiv.classList.add("menu-section");
        menuSectionDiv.setAttribute("role", "button");
        menuSectionDiv.addEventListener("click", () => {
            menuSectionDiv.classList.toggle("visible");
            const isExpanded = menuSectionDiv.classList.contains("visible");
            menuSectionDiv.setAttribute("aria-expanded", String(isExpanded));
            if (isExpanded) {
                const els = sectionsHostDiv.getElementsByClassName("visible");
                for (let i = 0; i < els.length; i++) {
                    const c = els[i] as HTMLDivElement;
                    if (c !== menuSectionDiv) {
                        c.classList.remove("visible");
                        c.setAttribute("aria-expanded", "false");
                    }
                }
            }
        });
        if (currentMenuSection === sectionKey) {
            menuSectionDiv.classList.add("visible");
            menuSectionDiv.setAttribute("aria-expanded", "true");
        } else {
            menuSectionDiv.setAttribute("aria-expanded", "false");
        }

        const menuSectionTitleDiv = document.createElement("div");
        menuSectionTitleDiv.classList.add("menu-section-title");

        const titleSpan = document.createElement("span");
        titleSpan.textContent = sectionData.label;
        menuSectionTitleDiv.appendChild(titleSpan);

        const expandMoreIcon = document.createElement(
            "iconify-icon",
        ) as HTMLElement;
        expandMoreIcon.classList.add("menu-section-expand-more");
        expandMoreIcon.setAttribute("icon", "mdi:expand-more");
        expandMoreIcon.setAttribute("width", "24");
        expandMoreIcon.setAttribute("height", "24");
        menuSectionTitleDiv.appendChild(expandMoreIcon);

        const expandLessIcon = document.createElement(
            "iconify-icon",
        ) as HTMLElement;
        expandLessIcon.classList.add("menu-section-expand-less");
        expandLessIcon.setAttribute("icon", "mdi:expand-less");
        expandLessIcon.setAttribute("width", "24");
        expandLessIcon.setAttribute("height", "24");
        menuSectionTitleDiv.appendChild(expandLessIcon);

        menuSectionDiv.appendChild(menuSectionTitleDiv);

        // Create <div class="menu-section-content">
        const menuSectionContentDiv = document.createElement("div");
        menuSectionContentDiv.classList.add("menu-section-content");

        const childrenItems = menuSectionChildren[sectionKey];
        if (childrenItems && childrenItems.length > 0) {
            childrenItems.forEach((itemData) => {
                if (
                    itemData.requirePerms !== 0 &&
                    (window.userData ||
                        (window.userData.permissions &
                            itemData.requirePerms) ===
                            0)
                )
                    return;
                // Create <a class="menu-element">
                const menuItemAnchor = document.createElement("a");
                menuItemAnchor.classList.add("menu-element");
                menuItemAnchor.href = itemData.path;

                if (selectedMenuElementPath === itemData.path) {
                    menuItemAnchor.classList.add("selected");
                }

                const itemSpan = document.createElement("span");
                itemSpan.textContent = itemData.label;
                menuItemAnchor.appendChild(itemSpan);

                menuSectionContentDiv.appendChild(menuItemAnchor);
            });
        }

        menuSectionDiv.appendChild(menuSectionContentDiv);
        sectionsHostDiv.appendChild(menuSectionDiv); // Add completed section to the host
    }

    menuContainer.appendChild(sectionsHostDiv); // Add the host of all sections to the main menu container

    const menuAccountDiv = document.createElement("div");
    menuAccountDiv.classList.add("menu-account");

    if (window.userData) {
        // const avatarDiv = document.createElement("div");
        // avatarDiv.classList.add("menu-account-avatar");
        // const avatarImg = document.createElement("img");
        // avatarImg.src = window.userData?.avatar || "";
        // avatarImg.alt = "Avatar";
        // avatarDiv.appendChild(avatarImg);
        // menuAccountDiv.appendChild(avatarDiv);

        const nameDiv = document.createElement("div");
        nameDiv.classList.add("menu-account-name");
        const firstNameSpan = document.createElement("span");
        firstNameSpan.classList.add("menu-account-first-name");
        firstNameSpan.textContent = window.userData?.firstName || "";
        nameDiv.appendChild(firstNameSpan);
        const usernameSpan = document.createElement("span");
        usernameSpan.classList.add("menu-account-username");
        usernameSpan.textContent = window.userData?.username || "";
        nameDiv.appendChild(usernameSpan);
        menuAccountDiv.appendChild(nameDiv);

        const logoutLink = document.createElement("a");
        logoutLink.classList.add("menu-account-logout");
        logoutLink.setAttribute("role", "button");
        logoutLink.title = "Logout";
        logoutLink.href = `https://auth.${window.domain}/logout`;

        const logoutIcon = document.createElement(
            "iconify-icon",
        ) as HTMLElement;
        logoutIcon.setAttribute("icon", "mdi:logout");
        logoutIcon.setAttribute("width", "24");
        logoutIcon.setAttribute("height", "24");
        logoutLink.appendChild(logoutIcon);
        menuAccountDiv.appendChild(logoutLink);
    } else {
        const connectLink = document.createElement("a");
        connectLink.classList.add("menu-account-connect");
        connectLink.href = `https://auth.${window.domain}/login?redirect=${encodeURIComponent(window.location.href)}`;

        const connectSpan = document.createElement("span");
        connectSpan.textContent = "Se connecter";
        connectLink.appendChild(connectSpan);

        const connectIcon = document.createElement(
            "iconify-icon",
        ) as HTMLElement;
        connectIcon.setAttribute("icon", "mdi:login");
        connectIcon.setAttribute("width", "24");
        connectIcon.setAttribute("height", "24");
        connectLink.appendChild(connectIcon);

        menuAccountDiv.appendChild(connectLink);
    }

    menuContainer.appendChild(menuAccountDiv);

    if (
        menuContainer.classList.contains("integrated-bar") &&
        !document.getElementById("menu-integrated-widener")
    ) {
        const widener = document.createElement("div");
        widener.id = "menu-integrated-widener";
        if (menuContainer.nextSibling) {
            menuContainer.parentElement.insertBefore(
                widener,
                menuContainer.nextSibling,
            );
        } else {
            menuContainer.parentElement.appendChild(widener);
        }
    }
}

function initButton() {
    let menuButton = document.getElementById("menu-button");
    if (menuButton) {
        menuButton.role = "button";
        menuButton.title = "Open Menu";
        menuButton.innerHTML = `<iconify-icon class="close-icon-close" icon="mdi:hamburger-close" width="35" height="35"></iconify-icon>`;

        menuButton?.addEventListener("click", () => {
            toggleMenu();
        });
    }
}

function initMenu() {
    fillWindowUserData();
    let t = window.localStorage.getItem("menuVisible");
    if (window.innerWidth < 600) t = "false"; // Force menu to be hidden on small screens on initial load
    const wasMenuVisible: boolean | null = t === null ? null : t === "true";

    menuContainer = document.getElementById(
        "menu-container",
    ) as HTMLDivElement | null;
    menuButtonBar = document.getElementById(
        "menu-button-bar",
    ) as HTMLDivElement | null;

    initButton();

    if (!menuContainer) {
        console.error("Menu container #menu-container not found in the DOM.");
        return;
    }

    if (menuContainer.hasAttribute("data-initial-section")) {
        currentMenuSection = menuContainer.getAttribute(
            "data-initial-section",
        ) as string;
    }
    if (menuContainer.hasAttribute("data-selected-path")) {
        selectedMenuElementPath = menuContainer.getAttribute(
            "data-selected-path",
        ) as string;
    }

    drawMenu();

    menuContainer.classList.forEach((c) => {
        if (c.startsWith("default-visible-")) {
            const width = c.split("-")[2];
            if (
                window.innerWidth >= parseInt(width) &&
                (wasMenuVisible === null || wasMenuVisible === true)
            ) {
                toggleMenu(true);
            }
            menuContainer.classList.remove(c);
        } else if (c === "default-visible") {
            if (
                window.innerWidth >= 600 &&
                (wasMenuVisible === null || wasMenuVisible === true)
            ) {
                toggleMenu(true);
            }
            menuContainer.classList.remove(c);
        }
    });

    const menuElementsDiv = menuContainer.querySelector(".menu-elements");
    if (menuElementsDiv) {
        menuElements = menuElementsDiv as HTMLDivElement;
    } else {
        console.warn(
            ".menu-elements div was not found after drawing the menu.",
        );
        menuElements = null;
    }

    menuContainer
        .getElementsByClassName("close-button")[0]
        .addEventListener("click", () => {
            toggleMenu();
        });

    if (
        wasMenuVisible === true &&
        !menuContainer.classList.contains("visible")
    ) {
        toggleMenu(true);
    }
}

function toggleMenu(newVal?: boolean) {
    if (!menuContainer) {
        console.error("Menu container is not initialized. Cannot toggle menu.");
        return;
    }
    menuContainer.classList.toggle("visible", newVal);
    menuButtonBar?.classList.toggle("hidden", newVal);
    localStorage.setItem(
        "menuVisible",
        String(menuContainer.classList.contains("visible")),
    );
}

function init() {
    initMenu();
}

window.addEventListener("DOMContentLoaded", init);

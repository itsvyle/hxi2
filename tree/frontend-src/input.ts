import "./input.scss";
import { faker } from "@faker-js/faker";
import { FuzzyInputList } from "../../global-frontend-dependencies/ui_fuzzy_input";
import { CustomWindow, listUsers, OtherUser } from "./api";
import {
    fillWindowUserData,
    getLocalUserData,
} from "../../global-frontend-dependencies/authUtils";
import {
    ICustomTab,
    UITabs,
    UITabsTab,
} from "../../global-frontend-dependencies/ui_tabs";

interface FuzzyWindow extends CustomWindow {
    fu: FuzzyInputList<OtherUser>;
}
declare var window: FuzzyWindow;

var tabsI = 0;
class CustomTestTab implements ICustomTab {
    id: string;
    initialTitle: string;
    tab: UITabsTab;
    closable = true;
    constructor(title: string) {
        this.id = String(tabsI++);
        this.initialTitle = title;
        this.tab = new UITabsTab(this);
    }

    onSelect() {
        console.log("Selected");
    }
    onUnselect() {
        console.log("Unselected");
    }
    onClose() {
        console.log("Closed");
    }
}

// const list = [];
// for (let i = 0; i < 200; i++) {
//     list.push(faker.person.fullName());
// }
document.addEventListener("DOMContentLoaded", () => {
    fillWindowUserData();

    const tabs: UITabs = document.querySelector("ui-tabs") as UITabs;

    const cust = new CustomTestTab("Tab custom 1");
    tabs.insertTab(cust.tab);

    const cust2 = new CustomTestTab("Tab custom 2");
    tabs.insertTab(cust2.tab);

    listUsers()
        .then((users) => {
            const g = (item: OtherUser) => {
                return `${item.firstName}${item.lastName.Valid ? " " + item.lastName.String : ""} [${item.promotion}]`;
            };
            window.fu = new FuzzyInputList(users, {
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
            window.fu.addWatcher(
                document.getElementById("input") as HTMLInputElement,
            );
        })
        .catch(console.error);
});

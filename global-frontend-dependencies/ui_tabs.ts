// A tabs system:
// - Tabs manager is custom element
// - Has permanent tabs (just associated to an element with an ID)
// - Has "temporary" tabs, associated to a class that fits an interface, can allow to be closed, etc.
// - (maybe) Has a way to save the state of the tabs and restore it
import "./ui_tabs.scss";
import { LitElement, html } from "lit-element";
import { property, customElement } from "lit-element/decorators.js";

enum TabType {
    Permanent = "permanent",
    Temporary = "temporary",
}

interface ICustomTab {
    id: string;

    // To change the title of the tab after it has been created, access the created tab element
    initialTitle: string;
    closable?: boolean;

    onSelect: () => void;
    onUnselect: () => void;
    onClose: () => void;

    beforeClose?: () => boolean;
    beforeUnselect?: () => boolean;
}

@customElement("ui-tabs-tab")
class UITabsTab extends LitElement {
    @property({ type: String, attribute: true })
    tabType: TabType = TabType.Permanent;
    @property({ type: String, attribute: true })
    title: string;

    @property({ type: String, attribute: true })
    elementId: string;

    @property({ type: Boolean, attribute: true })
    selected: boolean = false;

    @property({ type: Boolean })
    disabled: boolean = false;
    parentTabs: UITabs = this.parentElement as UITabs;

    customBackend: ICustomTab;

    constructor(back?: ICustomTab) {
        super();
        if (back) {
            this.tabType = TabType.Temporary;
            this.title = back.initialTitle;
            this.elementId = back.id;
            this.customBackend = back;
        }
        this.addEventListener("click", this._onClick);
        this.addEventListener("mousedown", (e) => {
            if (e.button === 1) {
                e.preventDefault();
                this._onCloseClick();
            }
        });
    }
    createRenderRoot() {
        return this; // Render template in light DOM instead of shadow dom
    }
    render() {
        this.classList.toggle("selected", this.selected);
        this.classList.toggle("disabled", this.disabled);
        if (this.customBackend?.closable) {
            return html`${this.title}
                <iconify-icon
                    icon="mdi:close"
                    title="Close"
                    @click=${this._onCloseClick}></iconify-icon>`;
        }
        return html`${this.title}`;
    }

    _onClick(e: MouseEvent) {
        if (this.disabled) return;
        if (e.target !== this) return;
        this.select();
    }

    _onCloseClick() {
        if (!this.customBackend?.closable) return;
        if (this.customBackend.beforeClose) {
            if (!this.customBackend.beforeClose()) return;
        }
        this.parentTabs?.removeTab(this.elementId);
        this.customBackend.onClose();
    }

    select() {
        if (this.selected) return;
        this.selected = true;
        this.parentTabs?.tabs.forEach((tab) => {
            if (tab !== this) tab.unselect();
        });
        if (this.tabType === TabType.Permanent) {
            let el = document.getElementById(this.elementId);
            if (el) {
                el.style.display = "";
            }
        } else if (this.tabType === TabType.Temporary) {
            this.customBackend.onSelect();
        }
        this.parentTabs?.tabChange(this);
    }

    unselect() {
        if (!this.selected) return;
        if (this.customBackend?.beforeUnselect) {
            if (!this.customBackend.beforeUnselect()) return;
        }
        this.selected = false;
        if (this.tabType === TabType.Permanent) {
            let el = document.getElementById(this.elementId);
            if (el) {
                el.style.display = "none";
            }
        } else if (this.tabType === TabType.Temporary) {
            this.customBackend.onUnselect();
        }
    }

    remove() {
        if (this.selected) {
            let next = this.nextElementSibling as UITabsTab;
            if (next) {
                try {
                    next.select();
                } catch (e) {
                    console.error(e);
                }
            } else {
                let prev = this.previousElementSibling as UITabsTab;
                if (prev) {
                    try {
                        prev.select();
                    } catch (e) {
                        console.error(e);
                    }
                }
            }
        }
        super.remove();
    }
}

@customElement("ui-tabs")
class UITabs extends LitElement {
    tabs: Map<string, UITabsTab> = new Map();

    @property({ type: Boolean, attribute: true })
    hideIfSingleTab: boolean;
    @property({ type: Boolean, attribute: true })
    appendHistory: boolean = false;

    currentlySelected?: string;

    private docLoaded = false;
    private _isHandlingPopstate = false;

    constructor() {
        super();
        document.addEventListener(
            "DOMContentLoaded",
            this._onDocumentReady.bind(this),
        );
        if (this.appendHistory) {
            if (
                !this.id &&
                this.ownerDocument &&
                Array.from(
                    this.ownerDocument.querySelectorAll(
                        "ui-tabs[append-history]",
                    ),
                ).filter((el) => el !== this && !el.id).length > 0
            ) {
                console.warn(
                    "UITabs: Multiple ui-tabs instances with 'append-history' are present without unique IDs. History behavior may be unpredictable. Please assign unique IDs to each <ui-tabs> element.",
                    this,
                );
            }
        }
    }

    connectedCallback() {
        super.connectedCallback();
        window.addEventListener("popstate", this._handlePopstate.bind(this));
    }

    private _onDocumentReady() {
        this.currentlySelected = undefined;
        let ch = this.children;
        for (let i = 0; i < ch.length; i++) {
            let tab = ch[i] as UITabsTab;
            this.tabs.set(tab.elementId, tab);
            tab.parentTabs = this;
            if (tab.selected) {
                if (this.currentlySelected) {
                } else {
                    this.currentlySelected = tab.elementId;
                }
            } else {
                if (tab.tabType === TabType.Permanent) {
                    let el = document.getElementById(tab.elementId);
                    if (el) {
                        el.style.display = "none";
                    }
                }
            }
        }
        this.docLoaded = true;
        if (this.hideIfSingleTab && this.tabs.size <= 1) {
            // prevent the flashing of the tabs on load
            this.style.display = "none";
            setTimeout(() => {
                this.style.display = "";
            }, 300);
        }
        this.onTabCountChange();
    }

    createRenderRoot() {
        return this; // Render template in light DOM instead of shadow dom
    }
    render() {
        this.onTabCountChange();
        return super.render();
    }

    onTabCountChange() {
        if (this.docLoaded) {
            this.classList.toggle(
                "hidden",
                this.hideIfSingleTab && this.tabs.size <= 1,
            );
        }
    }

    insertTab(tab: UITabsTab, before?: UITabsTab) {
        if (this.tabs.has(tab.elementId)) {
            throw new Error(`Tab with id ${tab.elementId} already exists`);
        }
        if (!tab.elementId) {
            throw new Error("Tab must have an elementId");
        }
        if (before) {
            this.insertBefore(tab, before.nextSibling);
        } else {
            this.appendChild(tab);
        }
        this.tabs.set(tab.elementId, tab);
        tab.parentTabs = this;
        this.onTabCountChange();
    }

    hasTab(id: string): boolean {
        return this.tabs.has(id);
    }

    removeTab(id: string) {
        let tab = this.tabs.get(id);
        if (!tab) {
            throw new Error(`Tab with id ${id} does not exist`);
        }
        this.tabs.delete(id);
        tab.remove();
        this.onTabCountChange();
    }

    tabChange(newSelectedTab: UITabsTab) {
        const oldSelectedId = this.currentlySelected;

        // If called with the same tab that's already considered selected,
        // still proceed if initializing to ensure history is set.
        if (
            oldSelectedId === newSelectedTab.elementId &&
            !this.docLoaded &&
            newSelectedTab.selected
        ) {
            return; // No actual change, and not initializing.
        }

        this.currentlySelected = newSelectedTab.elementId;

        this.dispatchEvent(
            new CustomEvent("selectChange", {
                detail: {
                    newTab: newSelectedTab,
                    oldTabId: oldSelectedId,
                },
                bubbles: true,
                composed: true,
            }),
        );

        if (
            this.appendHistory &&
            !this._isHandlingPopstate &&
            newSelectedTab.elementId
        ) {
            // Use null for tabsId in state if this.id is empty string, for consistency with _handlePopstate
            const state = {
                tabId: newSelectedTab.elementId,
                tabsId: this.id || null,
            };
            window.history.pushState(state, "", ``);
        }
    }

    private _handlePopstate = (event: PopStateEvent) => {
        if (!this.appendHistory || !this.docLoaded) return;

        this._isHandlingPopstate = true;
        let tabIdToSelect: string | null = null;
        const state = event.state as {
            tabId: string;
            tabsId: string | null;
        } | null;

        if (state && state.tabsId === (this.id || null)) {
            tabIdToSelect = state.tabId;
        }

        if (tabIdToSelect && this.tabs.has(tabIdToSelect)) {
            const tabToSelect = this.tabs.get(tabIdToSelect);
            if (tabToSelect && !tabToSelect.selected && !tabToSelect.disabled) {
                tabToSelect.select();
            }
        } else if (!tabIdToSelect && this.currentlySelected) {
            //@ts-expect-error
            this.getElementsByTagName("ui-tabs-tab")[0]?.select();
        }
        this._isHandlingPopstate = false;
    };
}

export { ICustomTab, UITabsTab, UITabs };

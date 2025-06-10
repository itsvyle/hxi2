import "./ui_loader.scss";
import type { IUILoader } from "./ui_interfaces";
const loadingManager = new (class implements IUILoader {
    private loadingCreated = false;
    private showTimeout: any = null;
    private delay = 400;
    private create() {
        if (document.getElementById("_loading-container")) {
            this.loadingCreated = true;
            return console.warn("Trying to create loading screen twice!");
        }
        const loadingContainer = document.createElement("div");
        loadingContainer.id = "_loading-container";
        const loadingBox = document.createElement("div");
        loadingBox.id = "_loading-box";
        const loadingIcon = document.createElement("div");
        loadingBox.appendChild(loadingIcon);
        loadingBox.appendChild(document.createTextNode("Loading..."));
        loadingContainer.appendChild(loadingBox);

        document.body.appendChild(loadingContainer);
        this.loadingCreated = true;
    }

    /**
     * @param {number} customDelay - An override for the delay before the loading screen is shown
     */
    public show(customDelay?: number): Promise<void> {
        return new Promise((resolve) => {
            if (!this.loadingCreated) this.create();
            if (!this.showTimeout) {
                document.body.classList.toggle("_loading-started", true);
                this.showTimeout = setTimeout(() => {
                    document.body.classList.toggle("_loading-started", false);
                    document.body.classList.toggle("_loading", true);
                    resolve();
                }, customDelay ?? this.delay);
            }
        });
    }

    public hide(): Promise<void> {
        return new Promise((resolve) => {
            if (this.showTimeout) {
                clearTimeout(this.showTimeout);
                this.showTimeout = null;
            }
            document.body.classList.toggle("_loading-started", false);
            document.body.classList.toggle("_loading", false);
            resolve();
        });
    }
})();

// @ts-expect-error
if (window.isDev) window.Loader = loadingManager;

export default loadingManager;

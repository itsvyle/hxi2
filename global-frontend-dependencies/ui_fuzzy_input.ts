import createFuzzySearch, {
    FuzzySearcher,
    FuzzySearchOptions,
} from "./microfuzz";

interface FuzzyInputListOptions<T> {
    fuzzyOptions?: FuzzySearchOptions;
    // for a given item, return a unique identifier
    getItemID: (item: T) => string;

    // for a given item, return the text to be shown
    getItemText: (item: T) => string;
}

class FuzzyInputList<T> {
    list: T[];
    fuzzySearch: FuzzySearcher<T>;
    watchingInputs: Record<
        string,
        {
            input: HTMLInputElement;
            latestSelectedItemID: string;
            latestSelectedItemText: string;
        }
    > = {};

    fuzzyResultsContainer: HTMLDivElement;

    private currentInput: HTMLInputElement | null = null;

    private fuzzyOptions?: FuzzySearchOptions;
    private getID: (item: T) => string;
    getItemText: (item: T) => string;
    constructor(list: T[], opts: FuzzyInputListOptions<T>) {
        this.fuzzyResultsContainer = document.createElement("div");
        this.fuzzyResultsContainer.className = "_fuzzy-results";
        document.body.appendChild(this.fuzzyResultsContainer);

        this.fuzzyOptions = opts.fuzzyOptions;
        this.getID = opts.getItemID;
        this.getItemText = opts.getItemText;
        this.fuzzyResultsContainer.addEventListener("mousedown", (e) => {
            if (
                this.fuzzyResultsContainer.classList.contains("no-results") ||
                !e.target ||
                !this.currentInput
            ) {
                return;
            }
            if (e.target === this.fuzzyResultsContainer) {
                const i = this.currentInput;
                setTimeout(() => {
                    i.focus();
                }, 0);
            }
            this.handleClickResult(e.target as HTMLDivElement);
        });
        this.updateFuzzyInputList(list);
    }

    updateFuzzyInputList(list: T[]) {
        this.list = list;
        this.fuzzySearch = createFuzzySearch(this.list, this.fuzzyOptions);
    }

    addWatcher(input: HTMLInputElement) {
        if (!input.id) {
            throw new Error("input must have an id");
        }
        this.watchingInputs[input.id] = {
            input,
            latestSelectedItemID: "",
            latestSelectedItemText: "",
        };
        input.addEventListener("focus", (e) => {
            this.currentInput = input;
            this.refreshResultsContainer(input);
            this.viewResultsContainer(input);
        });
        input.addEventListener("input", (e) => {
            this.refreshResultsContainer(input);
        });
        input.addEventListener("keydown", (e) => {
            if (e.key === "Enter") {
                input.blur();
            }
        });
        input.addEventListener("blur", (e) => {
            this.hideResultsContainer();
        });
    }

    private viewResultsContainer(input: HTMLInputElement) {
        this.currentInput = input;
        const rect = input.getBoundingClientRect();

        this.fuzzyResultsContainer.style.top = `${rect.top + window.scrollY + input.offsetHeight}px`;
        this.fuzzyResultsContainer.style.left = `${rect.left + window.scrollX}px`;
        this.fuzzyResultsContainer.style.width = `${input.offsetWidth}px`;
        this.fuzzyResultsContainer.classList.add("visible");
    }

    private refreshResultsContainer(input: HTMLInputElement) {
        const query = input.value;
        const watched = this.watchingInputs[input.id];
        if (watched.latestSelectedItemText !== query) {
            watched.latestSelectedItemID = "";
            watched.latestSelectedItemText = "";
        }
        if (query === "") {
            this.fuzzyResultsContainer.classList.toggle(
                "no-results",
                !this.list.length,
            );
            this.fuzzyResultsContainer.innerHTML = "";
            for (const item of this.list) {
                const div = document.createElement("div");
                div.className = "fuzzy-result";
                div.dataset.id = this.getID(item);
                div.innerText = this.getItemText(item);
                this.fuzzyResultsContainer.appendChild(div);
            }
            return;
        }
        const results = this.fuzzySearch(query);
        this.fuzzyResultsContainer.classList.toggle(
            "no-results",
            !results.length,
        );
        if (!results.length) {
            this.fuzzyResultsContainer.innerHTML = "";
            const span = document.createElement("span");
            span.className = "no-results";
            span.innerText = "No results found for " + query;
            this.fuzzyResultsContainer.appendChild(span);
            return;
        }
        this.fuzzyResultsContainer.innerHTML = "";

        for (const result of results) {
            const div = document.createElement("div");
            div.className = "fuzzy-result";
            div.dataset.id = this.getID(result.item);
            div.innerText = this.getItemText(result.item);
            this.fuzzyResultsContainer.appendChild(div);
        }

        if (results.length === 1 && results[0].score === 0) {
            watched.latestSelectedItemID = this.getID(results[0].item);
            watched.latestSelectedItemText = this.getItemText(results[0].item);
            this.currentInput?.setAttribute("invalid", "");
        }
    }

    private hideResultsContainer() {
        if (this.currentInput) {
            if (
                this.watchingInputs[this.currentInput.id]
                    .latestSelectedItemID ||
                this.currentInput.value === ""
            ) {
                this.currentInput.removeAttribute("invalid");
            } else {
                this.currentInput.setAttribute("invalid", "");
            }
        }
        this.currentInput = null;
        this.fuzzyResultsContainer.classList.remove("visible");
    }

    private handleClickResult(result: HTMLDivElement) {
        if (!this.currentInput || !result.dataset.id) {
            return;
        }
        const input = this.currentInput;
        input.value = result.innerText;
        this.watchingInputs[input.id].latestSelectedItemID = result.dataset.id;
        this.watchingInputs[input.id].latestSelectedItemText = result.innerText;
        input.dispatchEvent(
            new CustomEvent("fuzzyResultChose", {
                bubbles: true,
                cancelable: true,
                composed: true,
                detail: {
                    id: result.dataset.id,
                    text: result.innerText,
                },
            }),
        );
        this.hideResultsContainer();
    }

    getSelectedID(inputID: string) {
        return this.watchingInputs[inputID].latestSelectedItemID;
    }
}

export { FuzzyInputList, FuzzyInputListOptions };

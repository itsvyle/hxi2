export interface IUILoader {
    show(customDelay?: number): Promise<void>;
    hide(): Promise<void>;
}

//#region UI Dialog
export type UIDialogMouseCallback = (event: MouseEvent) => void;

export interface UIDialogButton {
    text: string;
    style?: string;
    bgColor?: string;
    className?: string;
    onclick?: (event: MouseEvent) => void;
    focus?: boolean;
}

export interface UIDialogDisplayOptions {
    title: string;
    message: string;
    buttons?: UIDialogButton[];
    checkboxes?: {
        id: string;
        text: string;
        checked: boolean;
    }[];
    allowCloseButton?: boolean;
    autoClose?: boolean;
    isError?: boolean;
}

export interface IUIDialog {
    display(options: UIDialogDisplayOptions): void;
    close(): void;
    error(
        message: string,
        fatal?: boolean,
        continueWith?: UIDialogMouseCallback,
    ): void;
    confirm(
        message: string,
        continueWith: UIDialogMouseCallback,
        cancelWith?: UIDialogMouseCallback,
    ): void;
    confirmWithCheckbox(
        message: string,
        checkbox_text: string,
        checkbox_checked: boolean,
        continueWith?: (e: MouseEvent, checkboxStatus: boolean) => void,
        cancelWith?: (e: MouseEvent, checkboxStatus: boolean) => void,
    ): void;

    success(message?: string, continueWith?: UIDialogMouseCallback): void;
}

import "./login.scss";
import Dialog from "../../global-frontend-dependencies/ui_dialog";

function getAndDeleteCookie(name): string | null {
    const cookieString = `; ${document.cookie}`;
    const parts = cookieString.split(`; ${name}=`);
    if (parts.length < 2) return null;

    const value = parts.pop().split(";").shift();

    // Delete the cookie (best-effort: path=/ covers most cases)
    document.cookie = `${name}=; Max-Age=0; path=/`;
    // @ts-expect-error
    document.cookie = `${name}=; Max-Age=0; path=/; domain=.${window.domain}`;

    return value;
}

window.addEventListener("DOMContentLoaded", () => {
    let authError = getAndDeleteCookie("authError");
    if (!authError) {
        authError = localStorage.getItem("authError");
        localStorage.removeItem("authError");
        if (authError) {
            try {
                authError = atob(authError);
            } catch (e) {
                console.error("Failed to decode auth error:", e);
                authError = "An unknown authentication error occurred.";
            }
        }
    } else {
        localStorage.removeItem("authError");
    }
    if (authError) {
        console.log("Auth error:", authError);
        if (authError.startsWith('"')) authError = authError.slice(1);
        if (authError.endsWith('"')) authError = authError.slice(0, -1);
        Dialog.error(authError, false);
    }
});

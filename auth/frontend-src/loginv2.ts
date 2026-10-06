import "./loginv2.scss";
import Dialog from "../../global-frontend-dependencies/ui_dialog";
import {
    create,
    toJson,
    toJsonString,
    fromJsonString,
} from "@bufbuild/protobuf";
import {
    AuthService,
    ListUsersResponseSchema,
} from "../../generated-proto/js/auth/v2/auth_pb";
import {
    GetCSRFTokenRequestSchema,
    GetCSRFTokenResponseSchema,
} from "../../generated-proto/js/auth/v2/jwt_pb";
import {
    createConnectTransport,
    createGrpcWebTransport,
} from "@connectrpc/connect-web";
import { ConnectError, createClient } from "@connectrpc/connect";
import { interceptors } from "./grpc-devtools";
import { fillWindowUserData } from "../../global-frontend-dependencies/authUtils";
fillWindowUserData();

const transport = createConnectTransport({
    baseUrl: "/",
    interceptors: [...interceptors],
});
const grpcClient = createClient(AuthService, transport);
// @ts-ignore
window.grpcClient = grpcClient;

// should handle getting a CSRFToken; on the page load, I'd like to start loading one; when i call "get" on this class, if I'm currently fetching one, wait for it to be done... otherwise potentially retry getting one
class CSRFToken {
    HeaderName = "x-csrf-token";
    value: string | null = null;
    initTask: Promise<void> | null = null;
    async init() {
        let res = await grpcClient
            .getCSRFToken(
                {},
                {
                    onHeader: (header) => {
                        this.value = header.get(this.HeaderName) ?? "";
                    },
                },
            )
            .catch((err) => {
                this.value = null;
                console.error("Error fetching CSRF token:", err);
            });
    }

    async get(): Promise<string | null> {
        if (this.value) {
            return this.value;
        }
        if (!this.initTask) {
            this.initTask = this.init();
        }
        await this.initTask;
        return this.value;
    }
}

let csrfToken = new CSRFToken();
document.addEventListener("DOMContentLoaded", () => csrfToken.init());

let loginOptionDivIDs = ["loginOptionOTP", "loginOptionUsername"];
function showLoginOptionDiv(divID: string) {
    loginOptionDivIDs.forEach((id) => {
        let div = document.getElementById(id);
        if (div) {
            div.classList.toggle("visible", id === divID);
        }
    });
    if (divID === "") {
        //@ts-ignore
        document.getElementById("loginOptionChoiceContainer").style.display =
            "";
    } else {
        //@ts-ignore
        document.getElementById("loginOptionChoiceContainer").style.display =
            "none";
    }
}
document.addEventListener("DOMContentLoaded", () => {
    let discordOption = document.getElementById("discordOptionButton");
    let oneTimeCodeOption = document.getElementById("oneTimeCodeOptionButton");
    let usernameOption = document.getElementById("usernameOptionButton");
    if (discordOption) {
        discordOption.addEventListener("click", () => {
            showLoginOptionDiv("loginOptionDiscord");
        });
    }
    if (oneTimeCodeOption) {
        oneTimeCodeOption.addEventListener("click", () => {
            showLoginOptionDiv("loginOptionOTP");
        });
    }
    if (usernameOption) {
        usernameOption.addEventListener("click", () => {
            showLoginOptionDiv("loginOptionUsername");
        });
    }

    document.querySelectorAll(".back-to-login-options").forEach((el) => {
        el.addEventListener("click", () => {
            showLoginOptionDiv("");
        });
    });
});

function initLoginForm() {
    const loginForm = document.getElementById("loginForm") as HTMLFormElement;
    const submitBtn = loginForm.querySelector(
        "button[type=submit]",
    ) as HTMLButtonElement;

    loginForm.addEventListener("submit", async (e) => {
        e.preventDefault();

        submitBtn.disabled = true;
        submitBtn.innerText = "Authenticating...";

        let csrf = await csrfToken.get();
        if (!csrf) {
            Dialog.error(
                "Failed to get CSRF token. Please try again later.",
                false,
            );
            submitBtn.disabled = false;
            submitBtn.innerText = "Se Connecter";
            return;
        }

        const username = (
            document.getElementById("username") as HTMLInputElement
        ).value;
        const password = (
            document.getElementById("password") as HTMLInputElement
        ).value;

        try {
            let res = await grpcClient.passwordLogin(
                {
                    username: username,
                    password: password,
                },
                {
                    headers: {
                        [csrfToken.HeaderName]: csrf,
                    },
                },
            );
            submitBtn.innerText = "Redirecting...";
            (document.getElementById("password") as HTMLInputElement).value =
                "";
            window.location.href = res.redirectTo || "/";
        } catch (err: ConnectError | any) {
            console.error("Error during login:", err);
            Dialog.error(
                err && err.message ? err.message : "An unknown error occurred",
                false,
            );
            submitBtn.disabled = false;
            submitBtn.innerText = "Se Connecter";
            return;
        }
    });
}
document.addEventListener("DOMContentLoaded", initLoginForm);

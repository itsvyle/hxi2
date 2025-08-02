import { checkAuthentication } from "../../global-frontend-dependencies/authUtils";

export interface MainUser {
    user_id: number;
    display_name: string;
    surnom: string;
    origine: string;
    voeu: string;
    couleur: string;
    c_or_ocaml: string;
    fun_fact: string;
    conseil: string;
    algebre_or_analyse: string;
    pronouns: string;
    discord_username: string;
    hide: boolean; // Indicates if the user is hidden
    edit_restrictions: number;
    updated_at: string;
}

export function listUsers(): Promise<Record<number, MainUser>> {
    return fetch(`/api/list_users`, { cache: "no-cache" })
        .catch((error) => {
            throw error;
        })
        .then(checkAuthentication)
        .then((res) => res.json());
}

export function getMyself(): Promise<MainUser> {
    return fetch(`/api/me`, { cache: "no-cache" })
        .catch((error) => {
            throw error;
        })
        .then(checkAuthentication)
        .then((res) => res.json());
}

export function updateMyself(data: MainUser): Promise<MainUser> {
    return fetch(`/api/me`, {
        method: "PUT",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(data),
    })
        .catch((error) => {
            throw error;
        })
        .then(checkAuthentication)
        .then((res) => res.json());
}

import loadingManager from "../../global-frontend-dependencies/ui_loader";
import {
    SmallData,
    checkAuthentication,
} from "../../global-frontend-dependencies/authUtils";

export interface CustomWindow extends Window {
    isDev: boolean;
    userData: SmallData | null;
    domain: string;
}
declare const window: CustomWindow;

export interface ActionResponse {
    success: boolean;
    error?: string;
}

export interface OtherUser {
    id: number;
    username: string;
    firstName: string;
    lastName: {
        Valid: boolean;
        String: string;
    };
    discordID: string;
    promotion: number;
    permissions: number;
}
export function otherUserDisplayName(user: OtherUser): string {
    return `${user.firstName}${user.lastName.Valid ? " " + user.lastName.String : ""} [${user.promotion}]`;
}

export function listUsers(): Promise<OtherUser[]> {
    return fetch(`/api/list_users`, { cache: "no-cache" })
        .catch((error) => {
            throw error;
        })
        .then(checkAuthentication)
        .then((res) => res.json());
}

export function listUsersMap(): Promise<Map<number, OtherUser>> {
    return listUsers().then((r: OtherUser[]) => {
        let m: Map<number, OtherUser> = new Map();
        r.forEach((u) => {
            m.set(u.id, u);
        });
        return m;
    });
}

export interface Relation {
    ID: number;
    parrainID: number;
    filleulID: number;
}

export function listRelations(): Promise<Map<number, Relation>> {
    return fetch(`/api/list_relations`, {
        cache: "no-cache",
    })
        .catch((error) => {
            throw error;
        })
        .then(checkAuthentication)
        .then((res) => res.json())
        .then((r: Relation[]) => {
            let m: Map<number, Relation> = new Map();
            r.forEach((u) => {
                m.set(u.ID, u);
            });
            return m;
        });
}

export function getUserTree(userID: number): Promise<string> {
    return fetch(`/api/list_relations?userID=${encodeURIComponent(userID)}`, {})
        .catch((error) => {
            throw error;
        })
        .then(checkAuthentication)
        .then((res) => res.text());
}

export function deleteRelation(id: number): Promise<ActionResponse> {
    let data = new FormData();
    data.append("id", String(id));

    return fetch(`/api/relation`, {
        method: "DELETE",
        body: data,
    })
        .catch((error) => {
            throw error;
        })
        .then(checkAuthentication)
        .then((res) => res.json());
}

export function createRelation(
    parrainID: number,
    filleulID: number,
): Promise<Relation | ActionResponse> {
    let data = new URLSearchParams();
    data.append("parrainID", String(parrainID));
    data.append("filleulID", String(filleulID));

    return fetch(`/api/relation`, {
        method: "POST",
        headers: {
            "Content-Type": "application/x-www-form-urlencoded",
        },
        body: data,
    })
        .catch((error) => {
            throw error;
        })
        .then(checkAuthentication)
        .then((res) => res.json());
}

interface GlobalTreeRelation extends Relation {
    coordX: number;
    coordY: number;
}

export interface GlobalTreeResponse {
    mermaidConfig: {
        layout: string;
        elk: {
            mergeEdges: boolean;
            nodePlacementStrategy: string;
        };
        startOnLoad: boolean;
        theme: string;
        themeVariables: Record<string, string>;
    };
    svg: string;
    svgHash: string;
    svgHeight: number;
    svgWidth: number;
    elements: Record<
        number,
        {
            x: number;
            y: number;
        }
    >;
}

export function getGlobalTree(): Promise<GlobalTreeResponse> {
    return fetch(`/api/global_tree`, { cache: "no-cache" })
        .catch((error) => {
            throw error;
        })
        .then(checkAuthentication)
        .then((res) => res.json());
}

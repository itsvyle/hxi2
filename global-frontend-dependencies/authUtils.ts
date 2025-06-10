import { Perms } from "./perms";

export interface SmallData {
    userID: number;
    username: string;
    firstName: string;
    lastName: string;
    permissions: number;
    promotion: number;
}

export function fillWindowUserData(): boolean {
    //@ts-expect-error
    if (window.userData) {
        return true;
    }
    const data = getLocalUserData();
    if (!data) {
        // console.error("No user data found");
        return false;
    }
    //@ts-expect-error
    window.userData = data;
    return true;
}

export function getLocalUserData(): SmallData | null {
    const cookie = document.cookie
        .split(";")
        .find((c) => c.trim().startsWith("HXI2_SMALL_DATA="));
    if (!cookie) {
        // console.error("HXI2_SMALL_DATA cookie not found");
        return null;
    }
    const cookieValue = cookie.split("=")[1];
    const decoded = b64DecodeUnicode(cookieValue);
    const data: SmallData = JSON.parse(decoded);
    return data;
}

export async function checkAuthentication(response: Response) {
    if (response.status === 401) {
        // window.location.href = "/authentication/login";
        throw "Not authenticated";
    } else if (response.status !== 200 && response.status !== 201) {
        // also catch other errors here; probably not very good design, but it's better than to add a new closure somewhere
        let t;
        try {
            t = await response.text();
        } catch (e) {
            throw `Error: ${response.status} ${response.statusText}`;
            return;
        }
        throw `Error: ${response.status} ${response.statusText}: ${t}`;
    }
    return response;
}

function b64DecodeUnicode(str) {
    // Going backwards: from bytestream, to percent-encoding, to original string.
    return decodeURIComponent(
        atob(str)
            .split("")
            .map(function (c) {
                return "%" + ("00" + c.charCodeAt(0).toString(16)).slice(-2);
            })
            .join(""),
    );
}

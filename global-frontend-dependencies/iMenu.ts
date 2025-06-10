export interface MenuItem {
    label: string;
    path: string;
    id?: string;
    requirePerms: number;
}

export interface MenuSection extends MenuItem {}

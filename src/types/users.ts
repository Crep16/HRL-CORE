export type Grant = { section: string; action: string };
export type Account = { id: number; name: string; role: string; accessLevel: string; active: boolean; permissions: Grant[] };
export type UserDraft = { id: number | null; name: string; role: string; accessLevel: string; active: boolean; permissions: Grant[]; password: string; confirmation: string; masterPassword: string };

import { request } from "./transport.ts";

export const windowApi = {
  synchronizeProfile: () => request<void>("sync_normalized_window_profile"),
};

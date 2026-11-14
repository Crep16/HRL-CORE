import type {
  LoginPayload, SessionResponse, SetupPayload, SetupStatus,
  TemporaryAccessPayload, TemporaryAccessResponse,
} from "../../types/ipc.ts";
import { request } from "./transport.ts";

export const authApi = {
  listLoginUsers: () => request<string[]>("list_login_users"),
  refresh: () => request<SessionResponse>("refresh_normalized_session"),
  checkSetupStatus: () => request<SetupStatus>("check_setup_status"),
  initializeSetup: (payload: SetupPayload) => request<SessionResponse>("initialize_setup", { payload }),
  loginUser: (payload: LoginPayload) => request<SessionResponse>("login_user", { payload }),
  logoutUser: () => request<void>("logout_user"),
  requestTemporaryAccess: (payload: TemporaryAccessPayload) =>
    request<TemporaryAccessResponse>("request_temporary_access", { payload }),
  finishTemporaryAccess: () => request<void>("finish_temporary_access"),
};

import type { RecoveryChallengeResponse, RecoveryResponse, VerifyRecoveryPayload } from "../../types/ipc.ts";
import { request } from "./transport.ts";

export const recoveryApi = {
  generateRecoveryChallenge: () => request<RecoveryChallengeResponse>("generate_recovery_challenge"),
  // Este comando Rust recibe cuatro argumentos planos, no un objeto { payload }.
  verifyRecoverySignature: (payload: VerifyRecoveryPayload) =>
    request<RecoveryResponse>("verify_recovery_signature", { ...payload }),
};

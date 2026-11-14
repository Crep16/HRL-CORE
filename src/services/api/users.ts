import { request } from './transport.ts';
import type { Account, UserDraft } from '../../types/users.ts';
export const usersApi = {
  list: () => request<Account[]>('list_normalized_users'),
  save: (payload: UserDraft) => request<void>('save_normalized_user', { payload }),
};

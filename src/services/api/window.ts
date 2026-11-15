import { request } from "./transport.ts";
import {isTauri} from '@tauri-apps/api/core';
import {getCurrentWindow} from '@tauri-apps/api/window';

export const windowApi = {
  native:isTauri,
  listenClose:(handler:()=>Promise<void>)=>getCurrentWindow().onCloseRequested(event=>{event.preventDefault();void handler();}),
  finishClose:()=>getCurrentWindow().destroy(),
  synchronizeProfile: () => request<void>("sync_normalized_window_profile"),
};

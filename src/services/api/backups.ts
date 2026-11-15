import { request } from './transport.ts';
export type BackupStatus = {folder:string;automatic:boolean;warning?:string|null;files:{name:string;bytes:number}[]};
export const backupApi={
  status:(masterPassword:string)=>request<BackupStatus>('backup_status',{payload:{masterPassword}}),
  configure:(masterPassword:string,automatic:boolean)=>request<void>('configure_backup',{payload:{masterPassword,automatic}}),
  create:(masterPassword:string)=>request<string>('create_backup',{payload:{masterPassword}}),
  openFolder:(masterPassword:string)=>request<void>('open_backup_folder',{payload:{masterPassword}}),
  automatic:()=>request<string|null>('automatic_backup'),
  restore:(currentMasterPassword:string,backupMasterPassword:string)=>request<boolean>('restore_backup',{payload:{currentMasterPassword,backupMasterPassword}}),
  restart:()=>request<void>('restart_after_restore'),
};

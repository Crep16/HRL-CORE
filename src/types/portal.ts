export interface PortalConfig { projectId:string; apiKey:string; publisherUid:string; portalUrl:string; paused:boolean }
export interface PortalSettings { config:PortalConfig; connected:boolean; error:string|null }
export type PublicationAction='publish'|'withdraw'|'rotate'|'reconcile';
export interface PortalOrderStatus {
  code:string|null; revision:number; enabled:boolean;
  state:'unpublished'|'pending'|'sending'|'confirmed'|'superseded'|'error'|'conflict';
  error:string|null; confirmedAt:string|null; configured:boolean; paused:boolean; url?:string;
}
export function initialPortalConfig():PortalConfig {
  return {projectId:'hrl-web',apiKey:'AIzaSyDZt78u5N36iMBAHzOAAj5YtQDX34Ucm9g',publisherUid:'Ziwbd42Rt9cLZbGsM91LAKAvotO2',portalUrl:'',paused:true};
}

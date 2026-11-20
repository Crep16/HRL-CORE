import {request} from './transport.ts';
import type {PortalConfig,PortalSettings,PortalOrderStatus,PublicationAction} from '../../types/portal.ts';
export const portalApi={
  settings:()=>request<PortalSettings>('portal_settings'),
  configure:(config:PortalConfig)=>request<void>('configure_portal',{config}),
  connect:(email:string,password:string)=>request<void>('connect_portal',{input:{email,password}}),
  pause:()=>request<void>('pause_portal'),
  resume:()=>request<void>('resume_portal'),
  disconnect:()=>request<void>('disconnect_portal'),
  status:(orderId:number)=>request<PortalOrderStatus>('portal_order_status',{orderId}),
  action:(orderId:number,action:PublicationAction)=>request<PortalOrderStatus>('portal_order_action',{orderId,action}),
};

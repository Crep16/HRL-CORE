export type Photo={id:string;orderId:string;originalName:string;mime:string;sizeBytes:number;source:'camera'|'file';note:string;authorName:string;createdAt:string;deleted:boolean;revision:number};
export type PhotoUpload={id:string;orderId:string;originalName:string;mime:string;source:'camera'|'file';note:string;bytes:number[]};
export type PhotoUpdate={orderId:string;id:string;note:string;deleted:boolean;expectedRevision:number};

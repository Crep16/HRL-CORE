export type LogbookChange = { field: string; oldValue: unknown; newValue: unknown };
export type LogbookEntry = {
  id: number; orderId: string; authorId: string; authorName: string;
  entity?: string; entityId?: string | null;
  operation: string; field: string | null; oldValue: string | null; newValue: string | null;
  details: LogbookChange[] | null; occurredAt: string | null; eventDate: string | null; currency: string;
  reason?:string|null; result?:string;
};
export type LogbookFilters = { query: string; operation: string; authorId: string; page: number;orderId?:string;fromUtc?:string;toUtc?:string };
export type LogbookPage = {
  entries: LogbookEntry[]; total: number; page: number; pageSize: number;
  authors: { id: string; name: string }[];
};

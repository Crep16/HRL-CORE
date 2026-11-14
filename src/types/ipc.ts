// Contratos IPC de src-tauri/src/domain/contracts.rs y security.rs (camelCase de Serde).
/** Entero seguro de JavaScript. Los ids Rust son i64; nunca se redondean ni se convierten a texto. */
export type EntityId = number;
/** Centavos enteros: Number.isSafeInteger debe cumplirse; no representan unidades decimales. */
export type MinorUnits = number;
export type AccessLevel = "complete" | "limited" | "custom";
export type Role = "administrator" | "technician";
export type AccessScope = "clients" | "repairs" | "company" | "logbook" | "users";

export type AppErrorCode =
  | "STORAGE_ERROR" | "MIGRATION_ERROR" | "LEGACY_DATABASE" | "VALIDATION_ERROR"
  | "UNAUTHENTICATED" | "FORBIDDEN" | "INVALID_CREDENTIALS" | "RATE_LIMITED"
  | "CONFLICT" | "NOT_FOUND" | "CRYPTO_ERROR" | "RECOVERY_UNAVAILABLE"
  | "RECOVERY_REJECTED" | "RESULT_UNCERTAIN" | "WINDOW_ERROR";

export interface AppError { code: AppErrorCode; message: string }
// Errores del puente/frontend; se distinguen de los códigos devueltos por Rust.
export interface ClientError {
  code: "IPC_ERROR" | "INVALID_INTEGER" | "BUSY" | "SESSION_CHANGED";
  message: string;
}
export type StoreError = AppError | ClientError;

export interface User {
  id: EntityId;
  name: string;
  loginName: string;
  role: Role;
  accessLevel: AccessLevel;
  active: boolean;
  createdAt: string;
}
export interface SetupStatus { setupComplete: boolean }
export interface LoginPayload { username: string; password: string }
export interface SetupPayload {
  name: string;
  username: string;
  password: string;
  confirmation: string;
  masterPassword: string;
  masterConfirmation: string;
}
export interface SessionResponse { user: User; permissions: string[] }
export interface TemporaryAccessPayload {
  username: string;
  password: string;
  scope: AccessScope;
  orderId: EntityId | null;
  operation: string;
  reason: string;
}
export interface TemporaryAccessResponse {
  id: EntityId;
  recipientId: EntityId;
  approverId: EntityId;
  scope: AccessScope;
  orderId: EntityId | null;
  operations: string[];
}
export interface RecoveryChallengeResponse {
  /** Firmar exactamente estos bytes UTF-8, sin agregar saltos de línea. */
  challenge: string;
  adminId: EntityId;
  adminUsername: string;
  expiresInSeconds: number;
}
export interface VerifyRecoveryPayload {
  challenge: string;
  /** Firma Ed25519 de 64 bytes codificada como 128 caracteres hexadecimales. */
  signature: string;
  newMasterPassword: string;
  newAdminPassword: string;
}
export interface RecoveryResponse { adminId: EntityId; requiresLogin: boolean }
export interface ClientInput {
  id: EntityId | null;
  fullName: string;
  phone: string;
  email: string | null;
}
export interface EquipmentInput {
  id: EntityId | null;
  item: string;
  brand: string;
  model: string;
  serial: string;
}
export interface EquipmentHistory {
  previousWorkshop: boolean;
  previousWorkshopDetail: string;
  sufferedImpact: boolean;
  impactDetail: string;
  liquidSpill: boolean;
  liquidDetail: string;
}
export interface InitialPaymentInput {
  /** Conservar el mismo id y contenido al reintentar una confirmación incierta. */
  requestId: string;
  amountMinor: MinorUnits;
  method: string;
}
export interface CreateOrderPayload {
  client: ClientInput;
  equipment: EquipmentInput;
  history: EquipmentHistory;
  folioPrefix: string;
  sequenceStart: number;
  status: string;
  currency: string;
  receivedAt: string;
  estimatedDeliveryAt: string | null;
  reportedFailure: string;
  plannedWork: string;
  receptionNotes: string;
  priceMinor: MinorUnits;
  devicePassword: string | null;
  initialPayment: InitialPaymentInput | null;
}
export interface GetOrderPayload {
  includeDeleted?: boolean;
  orderId: EntityId;
  scope: AccessScope;
  includeDevicePassword?: boolean;
}
export type OrderSearchField = "all" | "order" | "client" | "serial";
export interface ListOrdersPayload {
  trashed?: boolean; query: string; offset: number; limit: number; field?: OrderSearchField; oldestFirst?: boolean }
export interface OrderListItem {
  id: EntityId;
  folio: string;
  clientName: string;
  serial: string;
  item: string;
  status: string;
  receivedAt: string;
  currency: string;
  priceMinor: MinorUnits;
  netMinor: MinorUnits;
  pendingMinor: MinorUnits;
  creditMinor: MinorUnits;
  delivered: boolean;
}
export interface OrderListResponse { items: OrderListItem[]; total: number }
export interface FindOrderPayload { reference: string }
export interface OrderHistoryPayload { orderId: EntityId; offset: number; limit: number }
export interface OrderHistoryChange { field: string; oldValue: string | null; newValue: string | null }
export interface OrderHistoryEvent {
  id: EntityId; authorId: EntityId; authorName: string; operation: string; entity: string;
  entityId: string | null; reason: string | null; result: string; createdAt: string;
  changes: OrderHistoryChange[];
}
export interface OrderHistoryResponse { items: OrderHistoryEvent[]; total: number }
export interface ClientOutput { id: EntityId; fullName: string; phone: string; email: string | null }
export interface EquipmentOutput {
  id: EntityId;
  item: string;
  brand: string;
  model: string;
  serial: string;
}
export interface OrderResponse {
  id: EntityId;
  folio: string;
  clientId: EntityId;
  equipmentId: EntityId;
  receivedBy: EntityId;
  status: string;
  currency: string;
  receivedAt: string;
  estimatedDeliveryAt: string | null;
  reportedFailure: string;
  plannedWork: string;
  receptionNotes: string;
  priceMinor: MinorUnits;
  netMinor: MinorUnits;
  pendingMinor: MinorUnits;
  creditMinor: MinorUnits;
  hasDevicePassword: boolean;
  client: ClientOutput;
  equipment: EquipmentOutput;
  history: EquipmentHistory;
  revision: number;
  delivered: boolean;
  deliveredAt: string | null;
  /** Serde omite None. El secreto solo se obtiene por consulta autorizada explícita. */
  devicePassword?: string;
}

export type EditOrderPayload={orderId:number;revision:number;value:CreateOrderPayload;delivered:boolean;changePassword:boolean};
export type TrashOrderPayload={orderId:number;revision:number;deleted:boolean};

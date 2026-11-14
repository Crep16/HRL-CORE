use super::EntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentKind {
    Image,
    Video,
    Document,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceSource {
    Camera,
    File,
}

#[derive(Debug, Clone)]
pub struct Evidence {
    pub id: EntityId,
    pub file_code: String,
    pub order_id: EntityId,
    pub author_id: EntityId,
    pub original_name: String,
    pub relative_path: String,
    pub kind: AttachmentKind,
    pub mime: String,
    pub size_bytes: i64,
    pub source: EvidenceSource,
    pub description: String,
    pub initial_description: String,
    pub removed: bool,
    pub revision: i64,
    pub created_at: String,
}

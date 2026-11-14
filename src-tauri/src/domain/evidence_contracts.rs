use serde::{Deserialize, Serialize};
const MAX_BYTES: usize = 20 * 1024 * 1024;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Evidence {
    pub id: String,
    pub order_id: String,
    pub original_name: String,
    pub mime: String,
    pub size_bytes: i64,
    pub source: String,
    pub note: String,
    pub author_name: String,
    pub created_at: String,
    pub deleted: bool,
    pub revision: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NewEvidence {
    pub id: String,
    pub order_id: i64,
    pub original_name: String,
    pub mime: String,
    pub source: String,
    pub note: String,
    pub bytes: Vec<u8>,
}
pub fn valid_id(id: &str) -> Result<(), String> {
    if id.len() != 32
        || !id
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err("El identificador de la evidencia no es válido.".into());
    }
    Ok(())
}
pub fn validate(input: &NewEvidence) -> Result<(), String> {
    valid_id(&input.id)?;
    if input.original_name.trim().is_empty()
        || input.original_name.len() > 500
        || input.original_name.chars().any(char::is_control)
        || input.note.chars().count() > 500
        || !["file", "camera"].contains(&input.source.as_str())
    {
        return Err("Revisa el nombre, descripción y origen de la foto.".into());
    }
    if input.bytes.is_empty() || input.bytes.len() > MAX_BYTES {
        return Err("La foto debe ocupar entre 1 byte y 20 MB.".into());
    }
    let b = &input.bytes;
    let valid = match input.mime.as_str() {
        "image/jpeg" => b.starts_with(&[0xff, 0xd8, 0xff]) && b.ends_with(&[0xff, 0xd9]),
        "image/png" => b.starts_with(b"\x89PNG\r\n\x1a\n"),
        "image/webp" => b.len() >= 12 && &b[..4] == b"RIFF" && &b[8..12] == b"WEBP",
        _ => false,
    };
    if !valid {
        return Err("El archivo no corresponde a una foto JPG, PNG o WebP.".into());
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateEvidence {
    pub order_id: i64,
    pub id: String,
    pub note: String,
    pub deleted: bool,
    pub expected_revision: i64,
}

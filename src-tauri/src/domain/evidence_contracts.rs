use serde::{Deserialize, Serialize};
const MAX_BYTES: usize = 20 * 1024 * 1024;
pub fn evidence_format(mime: &str) -> Option<(&'static str, &'static str)> {
    match mime {
        "image/jpeg" => Some(("image", "jpg")),
        "image/png" => Some(("image", "png")),
        "image/webp" => Some(("image", "webp")),
        "application/pdf" => Some(("document", "pdf")),
        "video/mp4" => Some(("video", "mp4")),
        _ => None,
    }
}
fn valid_mp4(bytes: &[u8]) -> bool {
    let mut offset = 0;
    let (mut file_type, mut movie, mut data) = (false, false, false);
    while offset + 8 <= bytes.len() {
        let mut size = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        let kind = &bytes[offset + 4..offset + 8];
        let header = if size == 1 {
            if offset + 16 > bytes.len() {
                return false;
            }
            let wide = u64::from_be_bytes(bytes[offset + 8..offset + 16].try_into().unwrap());
            let Ok(value) = usize::try_from(wide) else {
                return false;
            };
            size = value;
            16
        } else {
            if size == 0 {
                size = bytes.len() - offset;
            }
            8
        };
        if size < header || size > bytes.len() - offset {
            return false;
        }
        if kind == b"ftyp" {
            if size < header + 8 {
                return false;
            }
            file_type = [
                &b"isom"[..],
                &b"iso2"[..],
                &b"mp41"[..],
                &b"mp42"[..],
                &b"avc1"[..],
                &b"M4V "[..],
            ]
            .contains(&&bytes[offset + header..offset + header + 4]);
        }
        movie |= kind == b"moov";
        data |= kind == b"mdat";
        offset += size;
    }
    offset == bytes.len() && file_type && movie && data
}
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
        return Err("Revisa el nombre, descripción y origen de la evidencia.".into());
    }
    if input.bytes.is_empty() || input.bytes.len() > MAX_BYTES {
        return Err("La evidencia debe ocupar entre 1 byte y 20 MB.".into());
    }
    let b = &input.bytes;
    let valid = match input.mime.as_str() {
        "image/jpeg" => b.starts_with(&[0xff, 0xd8, 0xff]) && b.ends_with(&[0xff, 0xd9]),
        "image/png" => b.starts_with(b"\x89PNG\r\n\x1a\n"),
        "image/webp" => b.len() >= 12 && &b[..4] == b"RIFF" && &b[8..12] == b"WEBP",
        "application/pdf" => {
            (b.starts_with(b"%PDF-1.") || b.starts_with(b"%PDF-2."))
                && b[b.len().saturating_sub(1024)..]
                    .windows(5)
                    .any(|s| s == b"%%EOF")
        }
        "video/mp4" => valid_mp4(b),
        _ => false,
    };
    if !valid || input.source == "camera" && !input.mime.starts_with("image/") {
        return Err(
            "El archivo no corresponde a JPG, PNG, WebP, PDF o MP4. La cámara solo admite fotos."
                .into(),
        );
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn attachments_validate_content_size_and_camera_origin() {
        let mut input = NewEvidence {
            id: "11112222333344445555666677778888".into(),
            order_id: 1,
            original_name: "informe.pdf".into(),
            mime: "application/pdf".into(),
            source: "file".into(),
            note: "".into(),
            bytes: b"%PDF-1.7\ncontenido\n%%EOF\n".to_vec(),
        };
        assert!(validate(&input).is_ok());
        input.source = "camera".into();
        assert!(validate(&input).is_err());
        input.source = "file".into();
        input.bytes = b"archivo-renombrado.pdf".to_vec();
        assert!(validate(&input).is_err());
        input.bytes = b"%PDF-1.7 sin final".to_vec();
        assert!(validate(&input).is_err());
        input.bytes = vec![0; MAX_BYTES + 1];
        assert!(validate(&input).is_err());
        input.mime = "video/mp4".into();
        input.bytes =
            b"\x00\x00\x00\x10ftypisom\x00\x00\x00\x00\x00\x00\x00\x08moov\x00\x00\x00\x09mdat0"
                .to_vec();
        assert!(validate(&input).is_ok());
        let valid = input.bytes.clone();
        input.bytes[3] = 255;
        assert!(validate(&input).is_err());
        input.bytes = valid[..16].to_vec();
        assert!(validate(&input).is_err());
        input.bytes = valid;
        input.mime = "application/pdf".into();
        assert!(validate(&input).is_err());
    }
}

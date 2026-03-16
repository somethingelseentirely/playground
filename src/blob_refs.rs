use triblespace::core::blob::schemas::UnknownBlob;
use triblespace::core::value::Value;
use triblespace::core::value::schemas::hash::{Blake3, Handle, Hash};

const FILES_SCHEME_PREFIX: &str = "files:";
const LEGACY_BLOB_SCHEME_PREFIX: &str = "blob:blake3:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobRef {
    pub alt: String,
    pub digest_hex: String,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptChunk {
    Text(String),
    Blob(BlobRef),
}

pub fn split_blob_refs(input: &str) -> Vec<PromptChunk> {
    let mut chunks = Vec::new();
    let mut cursor = 0usize;

    while let Some(start_rel) = input[cursor..].find("![") {
        let start = cursor + start_rel;
        if start > cursor {
            chunks.push(PromptChunk::Text(input[cursor..start].to_owned()));
        }

        let alt_start = start + 2;
        let Some(alt_sep_rel) = input[alt_start..].find("](") else {
            chunks.push(PromptChunk::Text(input[start..].to_owned()));
            cursor = input.len();
            break;
        };
        let alt_sep = alt_start + alt_sep_rel;
        let url_start = alt_sep + 2;
        let Some(url_end_rel) = input[url_start..].find(')') else {
            chunks.push(PromptChunk::Text(input[start..].to_owned()));
            cursor = input.len();
            break;
        };
        let url_end = url_start + url_end_rel;

        let alt = &input[alt_start..alt_sep];
        let url = &input[url_start..url_end];
        let raw = &input[start..=url_end];
        if let Some(blob_ref) = parse_blob_ref(alt, url, raw) {
            chunks.push(PromptChunk::Blob(blob_ref));
        } else {
            chunks.push(PromptChunk::Text(raw.to_owned()));
        }
        cursor = url_end + 1;
    }

    if cursor < input.len() {
        chunks.push(PromptChunk::Text(input[cursor..].to_owned()));
    }

    if chunks.is_empty() {
        chunks.push(PromptChunk::Text(String::new()));
    }
    merge_adjacent_text_chunks(chunks)
}

pub fn unknown_blob_handle_from_hex(hex: &str) -> Option<Value<Handle<Blake3, UnknownBlob>>> {
    let hash = Hash::<Blake3>::from_hex(hex).ok()?;
    Some(hash.into())
}

fn parse_blob_ref(alt: &str, url: &str, raw: &str) -> Option<BlobRef> {
    // Try files:<hash> first, then legacy blob:blake3:<hash>
    let digest_hex = if let Some(rest) = url.strip_prefix(FILES_SCHEME_PREFIX) {
        rest
    } else if let Some(rest) = url.strip_prefix(LEGACY_BLOB_SCHEME_PREFIX) {
        // Legacy format may have ?query params — strip them.
        rest.split_once('?').map_or(rest, |(digest, _)| digest)
    } else {
        return None;
    };

    if digest_hex.len() != 64 || !digest_hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }

    Some(BlobRef {
        alt: alt.to_owned(),
        digest_hex: digest_hex.to_ascii_uppercase(),
        raw: raw.to_owned(),
    })
}

fn merge_adjacent_text_chunks(chunks: Vec<PromptChunk>) -> Vec<PromptChunk> {
    let mut merged = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        match chunk {
            PromptChunk::Text(text) => match merged.last_mut() {
                Some(PromptChunk::Text(existing)) => existing.push_str(text.as_str()),
                _ => merged.push(PromptChunk::Text(text)),
            },
            other => merged.push(other),
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::{PromptChunk, split_blob_refs};

    #[test]
    fn parses_files_marker() {
        let input = "hello ![cat](files:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA) world";
        let chunks = split_blob_refs(input);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], PromptChunk::Text("hello ".to_string()));
        let PromptChunk::Blob(blob) = &chunks[1] else {
            panic!("expected blob");
        };
        assert_eq!(
            blob.digest_hex,
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        );
        assert_eq!(chunks[2], PromptChunk::Text(" world".to_string()));
    }

    #[test]
    fn parses_legacy_blob_marker_with_query() {
        let input = "hello ![cat](blob:blake3:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA?mime=image%2Fpng&name=cat.png) world";
        let chunks = split_blob_refs(input);
        assert_eq!(chunks.len(), 3);
        let PromptChunk::Blob(blob) = &chunks[1] else {
            panic!("expected blob");
        };
        assert_eq!(
            blob.digest_hex,
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        );
    }

    #[test]
    fn ignores_non_blob_markdown_images() {
        let input = "![x](https://example.com/x.png)";
        let chunks = split_blob_refs(input);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], PromptChunk::Text(input.to_string()));
    }
}

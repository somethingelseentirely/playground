use triblespace::core::metadata;
use triblespace::macros::id_hex;
pub use triblespace::prelude::blobschemas::FileBytes;
use triblespace::prelude::blobschemas::LongString;
use triblespace::prelude::valueschemas::{
    Blake3, GenId, Handle, NsTAIInterval, ShortString, U256BE,
};
use triblespace::prelude::*;

/// A unified archive projection for externally sourced conversations.
///
/// This schema is used by archive importers (ChatGPT, Codex, Copilot, Gemini, ...)
/// to store a common message/author/attachment graph, while keeping the raw
/// source artifacts separately (e.g. JSON trees, HTML, etc).
pub mod archive {
    use super::*;

    attributes! {
        "5F10520477A04E5FB322C85CC78C6762" as pub kind: GenId;

        "0D9195A7B1B20DE312A08ECE39168079" as pub reply_to: GenId;
        "838CC157FFDD37C6AC7CC5A472E43ADB" as pub author: GenId;
        "E63EE961ABDB1D1BEC0789FDAFFB9501" as pub author_name: Handle<Blake3, LongString>;
        "2D15150501ACCD9DFD96CB4BF19D1883" as pub author_role: Handle<Blake3, LongString>;
        "4FE6A8A43658BC2F61FEDF5CFB29EEFC" as pub author_model: Handle<Blake3, LongString>;
        "1F127324384335D12ECFE0CB84840925" as pub author_provider: Handle<Blake3, LongString>;
        "ACF09FF3D62B73983A222313FF0C52D2" as pub content: Handle<Blake3, LongString>;
        "0DA5DD275AA34F86B0297CC35F1B7395" as pub created_at: NsTAIInterval;

        "D8A469EAC2518D1A85692E0BEBF20D6C" as pub content_type: ShortString;
        "8334E282F24A4C7779C8899191B29E00" as pub attachment: GenId;

        "C9132D7400892F65B637BCBE92E230FB" as pub attachment_source_id: Handle<Blake3, LongString>;
        "A8F6CF04A9B2391A26F04BC84B77217D" as pub attachment_source_pointer: Handle<Blake3, LongString>;
        "9ADD88D3FFD9E4F91E0DC08126D9180A" as pub attachment_name: Handle<Blake3, LongString>;
        "EEFDB32D37B7B2834D99ACCF159B6507" as pub attachment_mime: ShortString;
        "D233E7BE0E973B09BD51E768E528ACA5" as pub attachment_size_bytes: U256BE;
        "5937E1072AF2F8E493321811B483C57B" as pub attachment_width_px: U256BE;
        "B252F4F77929E54FF8472027B7603EE9" as pub attachment_height_px: U256BE;
        "B0D18159D6035C576AE6B5D871AB4D63" as pub attachment_data: Handle<Blake3, FileBytes>;
    }

    /// Tag for message payloads.
    #[allow(non_upper_case_globals)]
    pub const kind_message: Id = id_hex!("1A0841C92BBDA0A26EA9A8252D6ECD9B");
    /// Tag for author entities.
    #[allow(non_upper_case_globals)]
    pub const kind_author: Id = id_hex!("4E4512EFB0BF0CD42265BD107AE7F082");
    /// Tag for attachment entities.
    #[allow(non_upper_case_globals)]
    pub const kind_attachment: Id = id_hex!("B465C85DD800633F58FE211B920AF2D9");

    #[allow(dead_code)]
    pub fn describe<B>(blobs: &mut B) -> std::result::Result<TribleSet, B::PutError>
    where
        B: BlobStore<Blake3>,
    {
        let mut tribles = TribleSet::new();

        tribles += entity! { ExclusiveId::force_ref(&kind_message) @
            metadata::shortname: "kind_message",
            metadata::name: blobs.put::<LongString, _>(
                "Message payload kind.".to_string(),
            )?,
        };

        tribles += entity! { ExclusiveId::force_ref(&kind_author) @
            metadata::shortname: "kind_author",
            metadata::name: blobs.put::<LongString, _>(
                "Author entity kind.".to_string(),
            )?,
        };

        tribles += entity! { ExclusiveId::force_ref(&kind_attachment) @
            metadata::shortname: "kind_attachment",
            metadata::name: blobs.put::<LongString, _>(
                "Attachment entity kind.".to_string(),
            )?,
        };

        Ok(tribles)
    }
}

#[allow(dead_code)]
pub fn build_archive_metadata<B>(blobs: &mut B) -> std::result::Result<TribleSet, B::PutError>
where
    B: BlobStore<Blake3>,
{
    let mut metadata = archive::describe(blobs)?;

    metadata.union(<GenId as metadata::ConstMetadata>::describe(blobs)?);
    metadata.union(<ShortString as metadata::ConstMetadata>::describe(blobs)?);
    metadata.union(<U256BE as metadata::ConstMetadata>::describe(blobs)?);
    metadata.union(<NsTAIInterval as metadata::ConstMetadata>::describe(blobs)?);
    metadata.union(<Handle<Blake3, LongString> as metadata::ConstMetadata>::describe(blobs)?);
    metadata.union(<Handle<Blake3, FileBytes> as metadata::ConstMetadata>::describe(blobs)?);
    metadata.union(<FileBytes as metadata::ConstMetadata>::describe(blobs)?);

    macro_rules! add_attribute {
        ($attribute:expr, $name:expr) => {
            metadata.union(describe_attribute(blobs, &$attribute, $name)?);
        };
    }

    add_attribute!(archive::kind, "kind");
    add_attribute!(archive::reply_to, "reply_to");
    add_attribute!(archive::author, "author");
    add_attribute!(archive::author_name, "author_name");
    add_attribute!(archive::author_role, "author_role");
    add_attribute!(archive::author_model, "author_model");
    add_attribute!(archive::author_provider, "author_provider");
    add_attribute!(archive::content, "content");
    add_attribute!(archive::created_at, "created_at");

    add_attribute!(archive::content_type, "content_type");
    add_attribute!(archive::attachment, "attachment");
    add_attribute!(archive::attachment_source_id, "attachment_source_id");
    add_attribute!(
        archive::attachment_source_pointer,
        "attachment_source_pointer"
    );
    add_attribute!(archive::attachment_name, "attachment_name");
    add_attribute!(archive::attachment_mime, "attachment_mime");
    add_attribute!(archive::attachment_size_bytes, "attachment_size_bytes");
    add_attribute!(archive::attachment_width_px, "attachment_width_px");
    add_attribute!(archive::attachment_height_px, "attachment_height_px");
    add_attribute!(archive::attachment_data, "attachment_data");

    Ok(metadata)
}

#[allow(dead_code)]
fn describe_attribute<B, S>(
    blobs: &mut B,
    attribute: &Attribute<S>,
    name: &str,
) -> std::result::Result<TribleSet, B::PutError>
where
    B: BlobStore<Blake3>,
    S: ValueSchema,
{
    let mut tribles = metadata::Metadata::describe(attribute, blobs)?;
    let handle = blobs.put::<LongString, _>(name.to_owned())?;
    let attribute_id = metadata::Metadata::id(attribute);
    tribles += entity! { ExclusiveId::force_ref(&attribute_id) @
        metadata::shortname: name,
        metadata::name: handle,
    };
    Ok(tribles)
}

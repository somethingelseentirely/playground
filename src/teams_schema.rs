use triblespace::core::metadata;
use triblespace::macros::id_hex;
use triblespace::prelude::blobschemas::LongString;
use triblespace::prelude::valueschemas::{Blake3, GenId, Handle, NsTAIInterval};
use triblespace::prelude::*;

pub mod teams {
    use super::*;

    attributes! {
        "1E525B603A0060D9FA132B3D4EE9538A" as pub chat: GenId;
        "B6089037C04529F55D2A2D1A668DBE95" as pub chat_id: Handle<Blake3, LongString>;
        "02D2C105E35BD5DD6CF7A1F1B74BA686" as pub message_id: Handle<Blake3, LongString>;
        "1DE123824D5BDA58F92CD002FCFB2BFF" as pub message_raw: Handle<Blake3, LongString>;
        "5820C49A7A8B4ADBCA4637E3AE2499EB" as pub user_id: Handle<Blake3, LongString>;
        "57AABA4FBA3A5EC6EF28DC80CD6E0919" as pub delta_link: Handle<Blake3, LongString>;
        "438A29922F91F873A69C3856AA7A553F" as pub access_token: Handle<Blake3, LongString>;
        "60C85DD37D09D3D27BC6BFA0E8040EA9" as pub refresh_token: Handle<Blake3, LongString>;
        "706CC590BF4684CA8FA00E4123C43124" as pub expires_at: valueschemas::NsTAIInterval;
        "0F7784BBDA2EE5B9009DE688472D6F24" as pub token_type: Handle<Blake3, LongString>;
        "139B46989D7F56C7DFE6259FD74479AC" as pub scope: Handle<Blake3, LongString>;
        "34ACCCECE281E1A0E191EEEBE7E47A23" as pub tenant: Handle<Blake3, LongString>;
        "8C6CA6A45DCA9F78420BC216A83F4C22" as pub client_id: Handle<Blake3, LongString>;
        "0E734F66EBBA45ED022D1EE539B11EBE" as pub client_secret: Handle<Blake3, LongString>;
    }

    /// Root id for describing the Teams protocol.
    #[allow(non_upper_case_globals)]
    #[allow(dead_code)]
    pub const teams_metadata: Id = id_hex!("CFE203B942D2534CC1212F1866804228");

    /// Tag for Teams chat entities.
    #[allow(non_upper_case_globals)]
    pub const kind_chat: Id = id_hex!("5BA4D47ED4358A77E29E372B972CA4F9");
    /// Tag for Teams cursor entities.
    #[allow(non_upper_case_globals)]
    pub const kind_cursor: Id = id_hex!("18B65C92AC77B1C1E2B3A4D6182A7EE7");
    /// Tag for Teams token cache entities.
    #[allow(non_upper_case_globals)]
    pub const kind_token: Id = id_hex!("7B6DBE9FD29182D97F1699437CF6627C");
    /// Tag for Teams log entries.
    #[allow(non_upper_case_globals)]
    pub const kind_log: Id = id_hex!("CAC47F309F894B23847E9A293F15C9B2");
    /// Tag for Teams app configuration entities.
    #[allow(non_upper_case_globals)]
    pub const kind_config: Id = id_hex!("0D7F4BBE36BD0D6FF4E6C651110D6E8B");

    /// Tag for Teams protocol metadata.
    #[allow(non_upper_case_globals)]
    pub const tag_protocol: Id = id_hex!("B28EB02C0AED51F14291FB989FC03C96");
    /// Tag for kind constants in the Teams protocol.
    #[allow(non_upper_case_globals)]
    pub const tag_kind: Id = id_hex!("DCB78D06E1E77B17B7097829DA395468");
    /// Tag for attribute constants in the Teams protocol.
    #[allow(non_upper_case_globals)]
    pub const tag_attribute: Id = id_hex!("B61C10637614B1D63485C6518DD70C56");
    /// Tag for tag constants in the Teams protocol.
    #[allow(non_upper_case_globals)]
    pub const tag_tag: Id = id_hex!("A0BBF05C642C2474D1574A67154D4F63");

    #[allow(dead_code)]
    pub fn describe<B>(blobs: &mut B) -> std::result::Result<TribleSet, B::PutError>
    where
        B: BlobStore<Blake3>,
    {
        let mut tribles = TribleSet::new();

        tribles += entity! { ExclusiveId::force_ref(&teams_metadata) @
            metadata::shortname: "teams_metadata",
            metadata::name: blobs.put::<LongString, _>(
                "Root id for describing the Teams bridge protocol.".to_string(),
            )?,
            metadata::tag: tag_protocol,
        };

        tribles += entity! { ExclusiveId::force_ref(&tag_protocol) @
            metadata::shortname: "tag_protocol",
            metadata::name: blobs.put::<LongString, _>(
                "Tag for Teams protocol metadata.".to_string(),
            )?,
            metadata::tag: tag_tag,
        };

        tribles += entity! { ExclusiveId::force_ref(&tag_kind) @
            metadata::shortname: "tag_kind",
            metadata::name: blobs.put::<LongString, _>(
                "Tag for Teams protocol kind constants.".to_string(),
            )?,
            metadata::tag: tag_tag,
        };

        tribles += entity! { ExclusiveId::force_ref(&tag_attribute) @
            metadata::shortname: "tag_attribute",
            metadata::name: blobs.put::<LongString, _>(
                "Tag for Teams protocol attributes.".to_string(),
            )?,
            metadata::tag: tag_tag,
        };

        tribles += entity! { ExclusiveId::force_ref(&tag_tag) @
            metadata::shortname: "tag_tag",
            metadata::name: blobs.put::<LongString, _>(
                "Tag for Teams protocol tag constants.".to_string(),
            )?,
            metadata::tag: tag_tag,
        };

        tribles += entity! { ExclusiveId::force_ref(&kind_chat) @
            metadata::shortname: "kind_chat",
            metadata::name: blobs.put::<LongString, _>(
                "Teams chat entity kind.".to_string(),
            )?,
            metadata::tag: tag_kind,
        };

        tribles += entity! { ExclusiveId::force_ref(&kind_cursor) @
            metadata::shortname: "kind_cursor",
            metadata::name: blobs.put::<LongString, _>(
                "Teams delta cursor kind.".to_string(),
            )?,
            metadata::tag: tag_kind,
        };

        tribles += entity! { ExclusiveId::force_ref(&kind_token) @
            metadata::shortname: "kind_token",
            metadata::name: blobs.put::<LongString, _>(
                "Teams token cache kind.".to_string(),
            )?,
            metadata::tag: tag_kind,
        };

        tribles += entity! { ExclusiveId::force_ref(&kind_log) @
            metadata::shortname: "kind_log",
            metadata::name: blobs.put::<LongString, _>(
                "Teams log entry kind.".to_string(),
            )?,
            metadata::tag: tag_kind,
        };

        tribles += entity! { ExclusiveId::force_ref(&kind_config) @
            metadata::shortname: "kind_config",
            metadata::name: blobs.put::<LongString, _>(
                "Teams app configuration kind.".to_string(),
            )?,
            metadata::tag: tag_kind,
        };

        Ok(tribles)
    }
}

#[allow(dead_code)]
pub fn build_teams_metadata<B>(blobs: &mut B) -> std::result::Result<TribleSet, B::PutError>
where
    B: BlobStore<Blake3>,
{
    let mut metadata = teams::describe(blobs)?;

    metadata.union(<GenId as metadata::ConstMetadata>::describe(blobs)?);
    metadata.union(<NsTAIInterval as metadata::ConstMetadata>::describe(blobs)?);
    metadata.union(<Handle<Blake3, LongString> as metadata::ConstMetadata>::describe(blobs)?);

    macro_rules! add_attribute {
        ($attribute:expr, $name:expr) => {
            metadata.union(describe_attribute(blobs, &$attribute, $name)?);
        };
    }

    add_attribute!(teams::chat, "teams_chat");
    add_attribute!(teams::chat_id, "teams_chat_id");
    add_attribute!(teams::message_id, "teams_message_id");
    add_attribute!(teams::message_raw, "teams_message_raw");
    add_attribute!(teams::user_id, "teams_user_id");
    add_attribute!(teams::delta_link, "teams_delta_link");
    add_attribute!(teams::access_token, "teams_access_token");
    add_attribute!(teams::refresh_token, "teams_refresh_token");
    add_attribute!(teams::expires_at, "teams_expires_at");
    add_attribute!(teams::token_type, "teams_token_type");
    add_attribute!(teams::scope, "teams_scope");
    add_attribute!(teams::tenant, "teams_tenant");
    add_attribute!(teams::client_id, "teams_client_id");
    add_attribute!(teams::client_secret, "teams_client_secret");

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
        metadata::tag: teams::tag_attribute,
    };
    Ok(tribles)
}

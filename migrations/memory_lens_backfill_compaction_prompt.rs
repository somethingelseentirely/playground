#!/usr/bin/env -S rust-script
//! ```cargo
//! [dependencies]
//! anyhow = "1.0"
//! clap = { version = "4.5.4", features = ["derive"] }
//! ed25519-dalek = "2.1.1"
//! hifitime = "4.2.3"
//! rand_core = "0.6.4"
//! triblespace = "0.16.0"
//! ```

use anyhow::{Result, anyhow, bail};
use clap::Parser;
use ed25519_dalek::SigningKey;
use hifitime::Epoch;
use rand_core::OsRng;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use triblespace::core::metadata;
use triblespace::core::repo::{Repository, Workspace};
use triblespace::macros::{attributes, find, id_hex, pattern};
use triblespace::prelude::*;

const DEFAULT_MEMORY_LENS_FACTUAL_PROMPT: &str = include_str!("../prompts/memory_lens_factual.md");
const DEFAULT_MEMORY_LENS_TECHNICAL_PROMPT: &str =
    include_str!("../prompts/memory_lens_technical.md");
const DEFAULT_MEMORY_LENS_EMOTIONAL_PROMPT: &str =
    include_str!("../prompts/memory_lens_emotional.md");
const DEFAULT_MEMORY_LENS_FACTUAL_COMPACTION_PROMPT: &str =
    include_str!("../prompts/memory_lens_factual_compaction.md");
const DEFAULT_MEMORY_LENS_TECHNICAL_COMPACTION_PROMPT: &str =
    include_str!("../prompts/memory_lens_technical_compaction.md");
const DEFAULT_MEMORY_LENS_EMOTIONAL_COMPACTION_PROMPT: &str =
    include_str!("../prompts/memory_lens_emotional_compaction.md");
const DEFAULT_MEMORY_LENS_FACTUAL_MAX_OUTPUT_TOKENS: u64 = 192;
const DEFAULT_MEMORY_LENS_TECHNICAL_MAX_OUTPUT_TOKENS: u64 = 224;
const DEFAULT_MEMORY_LENS_EMOTIONAL_MAX_OUTPUT_TOKENS: u64 = 96;
const DEFAULT_CONFIG_BRANCH_ID: Id = id_hex!("4790808CF044F979FC7C2E47FCCB4A64");

type TextHandle = Value<valueschemas::Handle<valueschemas::Blake3, blobschemas::LongString>>;

mod config_schema {
    use super::*;
    attributes! {
        "79F990573A9DCC91EF08A5F8CBA7AA25" as kind: valueschemas::GenId;
        "DDF83FEC915816ACAE7F3FEBB57E5137" as updated_at: valueschemas::NsTAIInterval;
        "24CF9D532E03C44CF719546DDE7E0493" as memory_lens_id: valueschemas::GenId;
        "1F0A596CD677F732CD5C506F74C61F6B" as memory_lens_prompt: valueschemas::Handle<valueschemas::Blake3, blobschemas::LongString>;
        "1067F34FE4517B058A74BC2118868DA4" as memory_lens_compaction_prompt: valueschemas::Handle<valueschemas::Blake3, blobschemas::LongString>;
        "84F32838DC66B0FB6F774150854521F8" as memory_lens_max_output_tokens: valueschemas::U256BE;
    }

    #[allow(non_upper_case_globals)]
    pub const kind_memory_lens: Id = id_hex!("D982F64C48F263A312D6E342D09554B0");
}

#[derive(Parser)]
#[command(
    name = "memory_lens_backfill_compaction_prompt",
    about = "Backfill missing memory lens compaction prompts (and other required fields)"
)]
struct Cli {
    /// Path to the pile file to use
    #[arg(long, default_value = "self.pile")]
    pile: PathBuf,
    /// Config branch id (hex). Defaults to canonical config branch id.
    #[arg(long)]
    config_branch_id: Option<String>,
    /// Print what would change without writing
    #[arg(long)]
    dry_run: bool,
}

#[derive(Clone, Debug)]
struct MemoryLensDefaults {
    name: String,
    prompt: String,
    compaction_prompt: String,
    max_output_tokens: u64,
}

fn now_epoch() -> Epoch {
    Epoch::now().unwrap_or_else(|_| Epoch::from_gregorian_utc(1970, 1, 1, 0, 0, 0, 0))
}

fn epoch_interval(epoch: Epoch) -> Value<valueschemas::NsTAIInterval> {
    (epoch, epoch).to_value()
}

fn interval_key(interval: Value<valueschemas::NsTAIInterval>) -> i128 {
    let (lower, _): (Epoch, Epoch) = interval.from_value();
    lower.to_tai_duration().total_nanoseconds()
}

fn parse_optional_hex_id(raw: Option<&str>, label: &str) -> Result<Option<Id>> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("{label} is empty");
    }
    let Some(id) = Id::from_hex(trimmed) else {
        bail!("invalid {label} '{trimmed}'");
    };
    Ok(Some(id))
}

fn open_repo(path: &Path) -> Result<Repository<Pile<valueschemas::Blake3>>> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|e| anyhow!("create pile dir {}: {e}", parent.display()))?;
    }
    let mut pile = Pile::<valueschemas::Blake3>::open(path)
        .map_err(|e| anyhow!("open pile {}: {e:?}", path.display()))?;
    if let Err(err) = pile.restore() {
        let _ = pile.close();
        return Err(anyhow!("restore pile {}: {err:?}", path.display()));
    }

    let signing_key = SigningKey::generate(&mut OsRng);
    Ok(Repository::new(pile, signing_key))
}

fn with_repo<T>(
    pile: &Path,
    f: impl FnOnce(&mut Repository<Pile<valueschemas::Blake3>>) -> Result<T>,
) -> Result<T> {
    let mut repo = open_repo(pile)?;
    let result = f(&mut repo);
    let close_res = repo.close().map_err(|e| anyhow!("close pile: {e:?}"));
    if let Err(err) = close_res {
        if result.is_ok() {
            return Err(err);
        }
        eprintln!("warning: failed to close pile cleanly: {err:#}");
    }
    result
}

fn load_string_attr(
    ws: &mut Workspace<Pile<valueschemas::Blake3>>,
    catalog: &TribleSet,
    entity_id: Id,
    attribute: &Attribute<valueschemas::Handle<valueschemas::Blake3, blobschemas::LongString>>,
) -> Result<Option<String>> {
    let handle = find!(
        (entity: Id, value: TextHandle),
        pattern!(catalog, [{ ?entity @ attribute: ?value }])
    )
    .into_iter()
    .find_map(|(entity, value)| (entity == entity_id).then_some(value));

    let Some(handle) = handle else {
        return Ok(None);
    };

    let view: View<str> = ws
        .get(handle)
        .map_err(|e| anyhow!("load longstring for entity {entity_id:x}: {e:?}"))?;
    Ok(Some(view.to_string()))
}

fn load_u64_attr(
    catalog: &TribleSet,
    entity_id: Id,
    attribute: &Attribute<valueschemas::U256BE>,
) -> Option<u64> {
    find!(
        (entity: Id, value: Value<valueschemas::U256BE>),
        pattern!(catalog, [{ ?entity @ attribute: ?value }])
    )
    .into_iter()
    .find_map(|(entity, value)| {
        if entity != entity_id {
            return None;
        }
        if value.raw[..24].iter().any(|byte| *byte != 0) {
            return None;
        }
        let bytes: [u8; 8] = value.raw[24..32].try_into().ok()?;
        Some(u64::from_be_bytes(bytes))
    })
}

fn latest_memory_lens_entries(catalog: &TribleSet) -> HashMap<Id, (Id, i128)> {
    let mut latest: HashMap<Id, (Id, i128)> = HashMap::new();
    for (entry_id, lens_id, updated_at) in find!(
        (
            entry_id: Id,
            lens_id: Value<valueschemas::GenId>,
            updated_at: Value<valueschemas::NsTAIInterval>
        ),
        pattern!(catalog, [{
            ?entry_id @
            config_schema::kind: &config_schema::kind_memory_lens,
            config_schema::updated_at: ?updated_at,
            config_schema::memory_lens_id: ?lens_id,
        }])
    ) {
        let lens_id = Id::from_value(&lens_id);
        let key = interval_key(updated_at);
        latest
            .entry(lens_id)
            .and_modify(|slot| {
                if key > slot.1 {
                    *slot = (entry_id, key);
                }
            })
            .or_insert((entry_id, key));
    }
    latest
}

fn defaults_for_lens(name: &str) -> MemoryLensDefaults {
    match name.to_ascii_lowercase().as_str() {
        "technical" => MemoryLensDefaults {
            name: "technical".to_string(),
            prompt: DEFAULT_MEMORY_LENS_TECHNICAL_PROMPT.to_string(),
            compaction_prompt: DEFAULT_MEMORY_LENS_TECHNICAL_COMPACTION_PROMPT.to_string(),
            max_output_tokens: DEFAULT_MEMORY_LENS_TECHNICAL_MAX_OUTPUT_TOKENS,
        },
        "emotional" => MemoryLensDefaults {
            name: "emotional".to_string(),
            prompt: DEFAULT_MEMORY_LENS_EMOTIONAL_PROMPT.to_string(),
            compaction_prompt: DEFAULT_MEMORY_LENS_EMOTIONAL_COMPACTION_PROMPT.to_string(),
            max_output_tokens: DEFAULT_MEMORY_LENS_EMOTIONAL_MAX_OUTPUT_TOKENS,
        },
        _ => MemoryLensDefaults {
            name: "factual".to_string(),
            prompt: DEFAULT_MEMORY_LENS_FACTUAL_PROMPT.to_string(),
            compaction_prompt: DEFAULT_MEMORY_LENS_FACTUAL_COMPACTION_PROMPT.to_string(),
            max_output_tokens: DEFAULT_MEMORY_LENS_FACTUAL_MAX_OUTPUT_TOKENS,
        },
    }
}

fn run(cli: Cli) -> Result<()> {
    let config_branch_id = parse_optional_hex_id(
        cli.config_branch_id.as_deref(),
        "config_branch_id",
    )?
    .unwrap_or(DEFAULT_CONFIG_BRANCH_ID);

    with_repo(&cli.pile, |repo| {
        let Some(_) = repo
            .storage_mut()
            .head(config_branch_id)
            .map_err(|e| anyhow!("config branch head: {e:?}"))?
        else {
            println!("No config branch found at {config_branch_id:x}; nothing to repair.");
            return Ok(());
        };

        let mut ws = repo
            .pull(config_branch_id)
            .map_err(|e| anyhow!("pull config workspace: {e:?}"))?;
        let catalog = ws
            .checkout(..)
            .map_err(|e| anyhow!("checkout config workspace: {e:?}"))?;

        let latest = latest_memory_lens_entries(&catalog);
        if latest.is_empty() {
            println!("No memory lens entries found; nothing to repair.");
            return Ok(());
        }

        let now = epoch_interval(now_epoch());
        let mut change = TribleSet::new();
        let mut repaired: Vec<String> = Vec::new();

        for (lens_id, (entry_id, _)) in latest {
            let fallback_name = format!("lens-{lens_id:x}");
            let name = load_string_attr(&mut ws, &catalog, entry_id, &metadata::name)?
                .unwrap_or(fallback_name);
            let defaults = defaults_for_lens(name.as_str());
            let canonical_name = if name.trim().is_empty() {
                defaults.name
            } else {
                name
            };

            let prompt = load_string_attr(&mut ws, &catalog, entry_id, &config_schema::memory_lens_prompt)?
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| defaults.prompt.clone());
            let compaction_prompt = load_string_attr(
                &mut ws,
                &catalog,
                entry_id,
                &config_schema::memory_lens_compaction_prompt,
            )?
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| defaults.compaction_prompt.clone());
            let max_output_tokens = load_u64_attr(
                &catalog,
                entry_id,
                &config_schema::memory_lens_max_output_tokens,
            )
            .filter(|value| *value > 0)
            .unwrap_or(defaults.max_output_tokens);

            let needs_repair = load_string_attr(
                &mut ws,
                &catalog,
                entry_id,
                &config_schema::memory_lens_compaction_prompt,
            )?
            .is_none()
                || load_u64_attr(
                    &catalog,
                    entry_id,
                    &config_schema::memory_lens_max_output_tokens,
                )
                .is_none()
                || load_string_attr(&mut ws, &catalog, entry_id, &config_schema::memory_lens_prompt)?
                    .is_none();

            if !needs_repair {
                continue;
            }

            repaired.push(format!("{canonical_name} ({lens_id:x})"));
            let entry_id = ufoid();
            let name_handle = ws.put(canonical_name.clone());
            let prompt_handle = ws.put(prompt);
            let compaction_handle = ws.put(compaction_prompt);
            let max_tokens: Value<valueschemas::U256BE> = max_output_tokens.to_value();
            change += entity! { &entry_id @
                config_schema::kind: &config_schema::kind_memory_lens,
                config_schema::updated_at: now,
                config_schema::memory_lens_id: lens_id,
                metadata::name: name_handle,
                config_schema::memory_lens_prompt: prompt_handle,
                config_schema::memory_lens_compaction_prompt: compaction_handle,
                config_schema::memory_lens_max_output_tokens: max_tokens,
            };
        }

        if repaired.is_empty() {
            println!("Memory lenses already complete; nothing to repair.");
            return Ok(());
        }

        repaired.sort();
        if cli.dry_run {
            println!("Would repair {} memory lens entry/entries:", repaired.len());
            for item in repaired {
                println!("- {item}");
            }
            return Ok(());
        }

        ws.commit(change, None, Some("backfill memory lens required fields"));
        repo.push(&mut ws)
            .map_err(|e| anyhow!("push memory lens repairs: {e:?}"))?;

        println!("Repaired {} memory lens entry/entries:", repaired.len());
        for item in repaired {
            println!("- {item}");
        }
        Ok(())
    })
}

fn main() -> Result<()> {
    run(Cli::parse())
}

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
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use triblespace::core::metadata;
use triblespace::core::repo::branch as branch_proto;
use triblespace::core::repo::{PushResult, Repository, Workspace};
use triblespace::macros::{attributes, find, id_hex, pattern};
use triblespace::prelude::*;

const DEFAULT_BRANCH: &str = "relations";
const CONFIG_BRANCH_ID: Id = id_hex!("4790808CF044F979FC7C2E47FCCB4A64");
const CONFIG_KIND_ID: Id = id_hex!("A8DCBFD625F386AA7CDFD62A81183E82");
const KIND_PERSON_ID: Id = id_hex!("D8ADDE47121F4E7868017463EC860726");

type TextHandle = Value<valueschemas::Handle<valueschemas::Blake3, blobschemas::LongString>>;

mod relations {
    use super::*;
    attributes! {
        "8F162B593D390E1424394DBF6883A72C" as alias: valueschemas::ShortString;
        "299E28A10114DC8C3B1661CD90CB8DF6" as label_norm: valueschemas::ShortString;
        "3E8812F6D22B2C93E2BCF0CE3C8C1979" as alias_norm: valueschemas::ShortString;
    }
}

mod config_schema {
    use super::*;
    attributes! {
        "79F990573A9DCC91EF08A5F8CBA7AA25" as kind: valueschemas::GenId;
        "DDF83FEC915816ACAE7F3FEBB57E5137" as updated_at: valueschemas::NsTAIInterval;
        "D35F4F02E29825FBC790E324EFCD1B34" as relations_branch_id: valueschemas::GenId;
    }
}

#[derive(Parser)]
#[command(
    name = "relations_backfill_norm",
    about = "Backfill relations label_norm/alias_norm fields on older piles"
)]
struct Cli {
    /// Path to the pile file to use
    #[arg(long, default_value = "self.pile")]
    pile: PathBuf,
    /// Branch name for relations data
    #[arg(long, default_value = DEFAULT_BRANCH)]
    branch: String,
    /// Branch id for relations data (hex). Overrides config.
    #[arg(long)]
    branch_id: Option<String>,
    /// Print what would change without writing
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Clone, Default)]
struct ConfigBranches {
    relations_branch_id: Option<Id>,
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

fn normalize_lookup_key(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        bail!("label is empty");
    }
    Ok(trimmed.to_ascii_lowercase())
}

fn read_text(ws: &mut Workspace<Pile<valueschemas::Blake3>>, handle: TextHandle) -> Result<String> {
    let view: View<str> = ws
        .get(handle)
        .map_err(|e| anyhow!("load longstring: {e:?}"))?;
    Ok(view.to_string())
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

fn find_branch_by_name(
    pile: &mut Pile<valueschemas::Blake3>,
    branch_name: &str,
) -> Result<Option<Id>> {
    let name_handle = branch_name
        .to_owned()
        .to_blob()
        .get_handle::<valueschemas::Blake3>();
    let reader = pile.reader().map_err(|e| anyhow!("pile reader: {e:?}"))?;
    let iter = pile
        .branches()
        .map_err(|e| anyhow!("list branches: {e:?}"))?;
    for branch in iter {
        let branch_id = branch.map_err(|e| anyhow!("branch id: {e:?}"))?;
        let Some(head) = pile
            .head(branch_id)
            .map_err(|e| anyhow!("branch head: {e:?}"))?
        else {
            continue;
        };
        let metadata_set: TribleSet = reader
            .get(head)
            .map_err(|e| anyhow!("branch metadata: {e:?}"))?;
        let mut names = find!(
            (handle: TextHandle),
            pattern!(&metadata_set, [{ metadata::name: ?handle }])
        )
        .into_iter();
        let Some(name) = names.next().map(|(handle,)| handle) else {
            continue;
        };
        if names.next().is_some() {
            continue;
        }
        if name == name_handle {
            return Ok(Some(branch_id));
        }
    }
    Ok(None)
}

fn load_config_branches(
    repo: &mut Repository<Pile<valueschemas::Blake3>>,
) -> Result<ConfigBranches> {
    let Some(_) = repo
        .storage_mut()
        .head(CONFIG_BRANCH_ID)
        .map_err(|e| anyhow!("config branch head: {e:?}"))?
    else {
        return Ok(ConfigBranches::default());
    };
    let mut ws = repo
        .pull(CONFIG_BRANCH_ID)
        .map_err(|e| anyhow!("pull config workspace: {e:?}"))?;
    let space = ws
        .checkout(..)
        .map_err(|e| anyhow!("checkout config workspace: {e:?}"))?;

    let mut latest: Option<(Id, i128)> = None;
    for (config_id, updated_at) in find!(
        (config_id: Id, updated_at: Value<valueschemas::NsTAIInterval>),
        pattern!(&space, [{
            ?config_id @
            config_schema::kind: &CONFIG_KIND_ID,
            config_schema::updated_at: ?updated_at,
        }])
    ) {
        let key = interval_key(updated_at);
        if latest.is_none_or(|(_, current)| key > current) {
            latest = Some((config_id, key));
        }
    }
    let Some((config_id, _)) = latest else {
        return Ok(ConfigBranches::default());
    };

    let relations_branch_id = find!(
        (entity: Id, value: Value<valueschemas::GenId>),
        pattern!(&space, [{ ?entity @ config_schema::relations_branch_id: ?value }])
    )
    .into_iter()
    .find_map(|(entity, value)| (entity == config_id).then_some(value.from_value()));

    Ok(ConfigBranches {
        relations_branch_id,
    })
}

fn resolve_branch_id(
    repo: &mut Repository<Pile<valueschemas::Blake3>>,
    explicit_id: Option<Id>,
    configured_id: Option<Id>,
    branch_name: &str,
) -> Result<Id> {
    if let Some(id) = explicit_id {
        return Ok(id);
    }
    if let Some(id) = configured_id {
        return Ok(id);
    }
    if let Some(id) = find_branch_by_name(repo.storage_mut(), branch_name)? {
        return Ok(id);
    }
    bail!(
        "missing relations branch id in config and branch '{}' not found",
        branch_name
    )
}

fn ensure_branch_with_id(
    repo: &mut Repository<Pile<valueschemas::Blake3>>,
    branch_id: Id,
    branch_name: &str,
) -> Result<()> {
    if repo
        .storage_mut()
        .head(branch_id)
        .map_err(|e| anyhow!("branch head {branch_name}: {e:?}"))?
        .is_some()
    {
        return Ok(());
    }

    let name_blob = branch_name.to_owned().to_blob();
    let name_handle = name_blob.get_handle::<valueschemas::Blake3>();
    repo.storage_mut()
        .put(name_blob)
        .map_err(|e| anyhow!("store branch name {branch_name}: {e:?}"))?;
    let metadata = branch_proto::branch_unsigned(branch_id, name_handle, None);
    let metadata_handle = repo
        .storage_mut()
        .put(metadata.to_blob())
        .map_err(|e| anyhow!("store branch metadata {branch_name}: {e:?}"))?;
    let result = repo
        .storage_mut()
        .update(branch_id, None, Some(metadata_handle))
        .map_err(|e| anyhow!("create branch {branch_name} ({branch_id:x}): {e:?}"))?;
    match result {
        PushResult::Success() | PushResult::Conflict(_) => Ok(()),
    }
}

fn run(cli: Cli) -> Result<()> {
    with_repo(&cli.pile, |repo| {
        let cfg = load_config_branches(repo)?;
        let explicit_branch_id = parse_optional_hex_id(cli.branch_id.as_deref(), "branch id")?;
        let branch_id = resolve_branch_id(
            repo,
            explicit_branch_id,
            cfg.relations_branch_id,
            &cli.branch,
        )?;
        ensure_branch_with_id(repo, branch_id, &cli.branch)?;

        let mut ws = repo
            .pull(branch_id)
            .map_err(|e| anyhow!("pull workspace: {e:?}"))?;
        let space = ws.checkout(..).map_err(|e| anyhow!("checkout: {e:?}"))?;

        let mut label_norms_by_person: HashMap<Id, HashSet<String>> = HashMap::new();
        for (person_id, handle) in find!(
            (person_id: Id, handle: TextHandle),
            pattern!(&space, [{
                ?person_id @
                metadata::tag: &KIND_PERSON_ID,
                metadata::name: ?handle,
            }])
        ) {
            let raw = read_text(&mut ws, handle)?;
            let key = normalize_lookup_key(&raw)?;
            label_norms_by_person
                .entry(person_id)
                .or_default()
                .insert(key);
        }

        let mut alias_norms_by_person: HashMap<Id, HashSet<String>> = HashMap::new();
        for (person_id, alias) in find!(
            (person_id: Id, alias: String),
            pattern!(&space, [{
                ?person_id @
                metadata::tag: &KIND_PERSON_ID,
                relations::alias: ?alias,
            }])
        ) {
            let key = alias.trim().to_ascii_lowercase();
            if key.is_empty() {
                continue;
            }
            alias_norms_by_person
                .entry(person_id)
                .or_default()
                .insert(key);
        }

        let mut person_ids = HashSet::new();
        person_ids.extend(label_norms_by_person.keys().copied());
        person_ids.extend(alias_norms_by_person.keys().copied());
        let mut person_ids: Vec<Id> = person_ids.into_iter().collect();
        person_ids.sort();

        let mut change = TribleSet::new();
        for person_id in &person_ids {
            let mut label_norms: Vec<String> = label_norms_by_person
                .remove(person_id)
                .unwrap_or_default()
                .into_iter()
                .collect();
            let mut alias_norms: Vec<String> = alias_norms_by_person
                .remove(person_id)
                .unwrap_or_default()
                .into_iter()
                .collect();
            label_norms.sort();
            alias_norms.sort();
            if label_norms.is_empty() && alias_norms.is_empty() {
                continue;
            }
            change += entity! { ExclusiveId::force_ref(person_id) @
                relations::label_norm*: label_norms.iter().map(String::as_str),
                relations::alias_norm*: alias_norms.iter().map(String::as_str),
            };
        }

        let delta = change.difference(&space);
        if delta.is_empty() {
            println!("Normalized lookup keys already up to date.");
            return Ok(());
        }

        if cli.dry_run {
            println!(
                "Would backfill normalized lookup keys for {} person(s).",
                person_ids.len()
            );
            return Ok(());
        }

        ws.commit(delta, None, Some("relations normalize lookup keys"));
        repo.push(&mut ws)
            .map_err(|e| anyhow!("push normalized lookup keys: {e:?}"))?;
        println!(
            "Backfilled normalized lookup keys for {} person(s).",
            person_ids.len()
        );
        Ok(())
    })
}

fn main() -> Result<()> {
    run(Cli::parse())
}

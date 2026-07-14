//! AD.5 — fixture tests for the transactional original-path installer. No network:
//! every fixture zip is built in a tempdir git repo at test time and served via a
//! local-path `source`. Ports all 8 AD.2 fixture scenarios to the new semantics and
//! adds the ratified-semantic coverage (version replace+prune, unknown-file refusal +
//! --adopt, journal roll-forward AND re-fetch recovery, retry engagement, profiles,
//! gen-ignore/ci-guard, lock).

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use xtask::fetch_assets::{
    ci_guard, gen_ignore, gen_keeplist, generated_ignore_content, is_safe_relative, load_manifest,
    retry_on_access_denied, run_fetch, verify_assets_cmd, FetchOptions, Manifest, PackEntry,
    PackStatus, Selection, GENERATED_IGNORE, PACKLIST_DIR, SCHEMA_VERSION, STATE_DIR,
};

// ───────────────────────── fixture plumbing ─────────────────────────

fn git(root: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .expect("git on PATH");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Build an AD.3-family zip: members at original repo-relative paths, plus the two
/// synthetic files (`_PACK_MANIFEST.txt` listing the members, `_ATTRIBUTION.txt`).
fn build_pack_zip(path: &Path, members: &[(&str, &[u8])]) {
    let file = File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let opts: zip::write::SimpleFileOptions = Default::default();
    for (name, contents) in members {
        zip.start_file(*name, opts).unwrap();
        zip.write_all(contents).unwrap();
    }
    let mut listed: Vec<&str> = members.iter().map(|(n, _)| *n).collect();
    listed.sort();
    zip.start_file("_PACK_MANIFEST.txt", opts).unwrap();
    zip.write_all((listed.join("\n") + "\n").as_bytes())
        .unwrap();
    zip.start_file("_ATTRIBUTION.txt", opts).unwrap();
    zip.write_all(b"AstraWeave Asset Pack: fixture\n").unwrap();
    zip.finish().unwrap();
}

fn sha256_hex(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(fs::read(path).unwrap()))
}

struct Fixture {
    _tmp: TempDir,
    root: PathBuf,
    manifest_path: PathBuf,
    zip_path: PathBuf,
}

impl Fixture {
    fn state(&self, rel: &str) -> PathBuf {
        self.root.join(STATE_DIR).join(rel)
    }
    fn opts(&self) -> FetchOptions {
        FetchOptions {
            selection: Selection::All,
            ..Default::default()
        }
    }
    fn rewrite_manifest(&self, m: &Manifest) {
        fs::write(&self.manifest_path, toml::to_string_pretty(m).unwrap()).unwrap();
    }
    /// Rewrite a pack's committed packlist (the file-exact ownership ledger).
    fn write_packlist(&self, pack: &str, members: &[&str]) {
        let dir = self.root.join(PACKLIST_DIR);
        fs::create_dir_all(&dir).unwrap();
        let mut sorted: Vec<&str> = members.to_vec();
        sorted.sort();
        fs::write(dir.join(format!("{pack}.txt")), sorted.join("\n") + "\n").unwrap();
    }
}

/// Write a packlist under an arbitrary repo root (for multi-pack fixtures).
fn write_packlist_at(root: &Path, pack: &str, members: &[&str]) {
    let dir = root.join(PACKLIST_DIR);
    fs::create_dir_all(&dir).unwrap();
    let mut sorted: Vec<&str> = members.to_vec();
    sorted.sort();
    fs::write(dir.join(format!("{pack}.txt")), sorted.join("\n") + "\n").unwrap();
}

const MEMBERS: &[(&str, &[u8])] = &[
    ("assets/x/hello.txt", b"hello packs"),
    ("assets/x/sub/data.bin", &[0u8, 1, 2, 3, 255]),
];

/// Standard fixture: a git repo, generated assets/.gitignore committed, one pack
/// (roots = ["assets/x/"]) with a local-path source.
fn fixture(zip_members: &[(&str, &[u8])], sha256_override: Option<&str>) -> Fixture {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    let zip_path = root.join("fixture_pack.zip");
    build_pack_zip(&zip_path, zip_members);
    let sha = sha256_override
        .map(str::to_string)
        .unwrap_or_else(|| sha256_hex(&zip_path));
    let manifest = Manifest {
        schema_version: SCHEMA_VERSION,
        packs: vec![PackEntry {
            name: "fixture".into(),
            source: zip_path.to_string_lossy().into_owned(),
            sha256: sha,
            size_bytes: fs::metadata(&zip_path).unwrap().len(),
            roots: vec!["assets/x/".into()],
        }],
        profiles: BTreeMap::from([
            ("starter".into(), vec!["fixture".into()]),
            ("all".into(), vec!["*".into()]),
        ]),
    };
    fs::create_dir_all(root.join("assets")).unwrap();
    let manifest_path = root.join("assets/packs.manifest.toml");
    fs::write(&manifest_path, toml::to_string_pretty(&manifest).unwrap()).unwrap();
    fs::write(
        root.join(GENERATED_IGNORE),
        generated_ignore_content(&manifest),
    )
    .unwrap();
    // Committed packlist = the zip's member list (the ownership ledger of record).
    let member_names: Vec<&str> = zip_members.iter().map(|(n, _)| *n).collect();
    write_packlist_at(&root, "fixture", &member_names);

    git(&root, &["init", "-q"]);
    git(&root, &["config", "user.name", "fixture"]);
    git(&root, &["config", "user.email", "fixture@test"]);
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "fixture init"]);

    Fixture {
        root,
        manifest_path,
        zip_path,
        _tmp: tmp,
    }
}

// ───────────────────────── ported AD.2 scenarios ─────────────────────────

#[test]
fn manifest_parse_round_trip() {
    let f = fixture(MEMBERS, None);
    let parsed = load_manifest(&f.manifest_path).unwrap();
    assert_eq!(parsed.schema_version, SCHEMA_VERSION);
    assert_eq!(parsed.packs.len(), 1);
    assert_eq!(parsed.packs[0].roots, vec!["assets/x/"]);
    assert!(parsed.profiles.contains_key("starter"));
    let reserialized = toml::to_string_pretty(&parsed).unwrap();
    let reparsed: Manifest = toml::from_str(&reserialized).unwrap();
    assert_eq!(parsed, reparsed);
}

#[test]
fn happy_path_install_original_paths_stamp_ledger() {
    let f = fixture(MEMBERS, None);
    let results = run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    assert_eq!(results, vec![("fixture".into(), PackStatus::Installed)]);
    // restored to ORIGINAL paths
    assert_eq!(
        fs::read(f.root.join("assets/x/hello.txt")).unwrap(),
        b"hello packs"
    );
    assert!(f.root.join("assets/x/sub/data.bin").exists());
    // state written
    assert!(f.state("stamps/fixture.toml").exists());
    assert!(f.state("ledgers/fixture.toml").exists());
    assert!(f.state("meta/fixture/_PACK_MANIFEST.txt").exists());
    assert!(f.state("meta/fixture/_ATTRIBUTION.txt").exists());
    // transaction cleaned
    assert!(!f.state("journal/fixture.toml").exists());
    assert!(!f.state("staging/fixture").exists());
}

#[test]
fn checksum_mismatch_rejected_loudly() {
    let bogus = "0".repeat(64);
    let f = fixture(MEMBERS, Some(&bogus));
    let results = run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    match &results[0].1 {
        PackStatus::Failed(msg) => {
            assert!(msg.contains("sha256 mismatch"), "got: {msg}");
            assert!(msg.contains(&bogus), "must name expected hash; got: {msg}");
            assert!(
                msg.contains(&sha256_hex(&f.zip_path)),
                "must name actual; got: {msg}"
            );
        }
        other => panic!("expected Failed, got {other:?}"),
    }
    assert!(!f.root.join("assets/x/hello.txt").exists());
    assert!(!f.state("stamps/fixture.toml").exists());
}

#[test]
fn stamped_noop_leaves_everything_alone() {
    let f = fixture(MEMBERS, None);
    run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    // Unknown file under the root (NOT at a pack destination) + local edit marker.
    let marker = f.root.join("assets/x/MARKER.local");
    fs::write(&marker, b"survives no-op").unwrap();

    let results = run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    assert_eq!(results, vec![("fixture".into(), PackStatus::UpToDate)]);
    assert!(marker.exists(), "no-op must not touch the tree");
}

#[test]
fn path_traversal_and_out_of_root_members_rejected() {
    // (a) traversal member
    let f = fixture(&[("../evil.txt", b"escape")], None);
    let results = run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    match &results[0].1 {
        PackStatus::Failed(msg) => assert!(msg.contains("path traversal"), "got: {msg}"),
        other => panic!("expected Failed, got {other:?}"),
    }
    assert!(!f.root.parent().unwrap().join("evil.txt").exists());

    // (b) member outside the declared roots (covers both out-of-assets and
    // out-of-root shapes — ownership is packlist+roots exact)
    let f2 = fixture(&[("outside/file.txt", b"nope")], None);
    let results = run_fetch(&f2.manifest_path, &f2.root, &f2.opts()).unwrap();
    match &results[0].1 {
        PackStatus::Failed(msg) => assert!(msg.contains("declared roots"), "got: {msg}"),
        other => panic!("expected Failed, got {other:?}"),
    }
    assert!(!f2.root.join("outside/file.txt").exists());

    // (c) member inside assets/ but outside this pack's roots
    let f3 = fixture(&[("assets/y/file.txt", b"nope")], None);
    let results = run_fetch(&f3.manifest_path, &f3.root, &f3.opts()).unwrap();
    match &results[0].1 {
        PackStatus::Failed(msg) => assert!(msg.contains("declared roots"), "got: {msg}"),
        other => panic!("expected Failed, got {other:?}"),
    }
    assert!(!f3.root.join("assets/y/file.txt").exists());

    // (d) the predicate itself, incl. Windows-specific shapes
    for bad in ["../x", "a/../../x", "/abs", "C:/x", "C:\\x", "..", ""] {
        assert!(!is_safe_relative(bad), "must reject {bad:?}");
    }
    for good in ["a", "a/b.txt", "./a/b", "a/./b"] {
        assert!(is_safe_relative(good), "must accept {good:?}");
    }
}

#[test]
fn force_restores_pristine_without_touching_unknown_files() {
    let f = fixture(MEMBERS, None);
    run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    // Locally corrupt a pack file; plant an unknown file under the root.
    fs::write(f.root.join("assets/x/hello.txt"), b"locally corrupted").unwrap();
    let marker = f.root.join("assets/x/MARKER.local");
    fs::write(&marker, b"not pack-managed").unwrap();

    let opts = FetchOptions {
        selection: Selection::All,
        force: true,
        ..Default::default()
    };
    let results = run_fetch(&f.manifest_path, &f.root, &opts).unwrap();
    assert_eq!(results, vec![("fixture".into(), PackStatus::Installed)]);
    assert_eq!(
        fs::read(f.root.join("assets/x/hello.txt")).unwrap(),
        b"hello packs",
        "--force must restore pack files to pristine"
    );
    assert!(
        marker.exists(),
        "--force must NOT delete unknown files that are not at pack destinations"
    );
}

#[test]
fn pack_filter_selects_and_rejects_unknown() {
    let f = fixture(MEMBERS, None);
    let opts = FetchOptions {
        selection: Selection::Pack("fixture".into()),
        ..Default::default()
    };
    let results = run_fetch(&f.manifest_path, &f.root, &opts).unwrap();
    assert_eq!(results.len(), 1);

    let missing = FetchOptions {
        selection: Selection::Pack("nope".into()),
        ..Default::default()
    };
    let err = run_fetch(&f.manifest_path, &f.root, &missing).unwrap_err();
    assert!(err.to_string().contains("no pack named 'nope'"));
    assert!(err.to_string().contains("fixture"), "should list available");
}

#[test]
fn corrupted_stamp_triggers_reinstall() {
    let f = fixture(MEMBERS, None);
    run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    let stamp_path = f.state("stamps/fixture.toml");
    let corrupted = fs::read_to_string(&stamp_path)
        .unwrap()
        .replace(&sha256_hex(&f.zip_path), &"f".repeat(64));
    fs::write(&stamp_path, corrupted).unwrap();

    let results = run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    assert_eq!(results, vec![("fixture".into(), PackStatus::Installed)]);
}

// ───────────────────────── new ratified semantics ─────────────────────────

#[test]
fn version_change_replaces_and_prunes() {
    let f = fixture(MEMBERS, None);
    run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    assert!(f.root.join("assets/x/sub/data.bin").exists());

    // New pack version: hello.txt changed, sub/data.bin dropped, fresh.txt added.
    let v2_members: &[(&str, &[u8])] = &[
        ("assets/x/hello.txt", b"hello v2"),
        ("assets/x/fresh.txt", b"new in v2"),
    ];
    build_pack_zip(&f.zip_path, v2_members);
    let mut m = load_manifest(&f.manifest_path).unwrap();
    m.packs[0].sha256 = sha256_hex(&f.zip_path);
    m.packs[0].size_bytes = fs::metadata(&f.zip_path).unwrap().len();
    f.rewrite_manifest(&m);
    // A re-cut pack ships with a regenerated committed packlist.
    f.write_packlist("fixture", &["assets/x/hello.txt", "assets/x/fresh.txt"]);

    let results = run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    assert_eq!(results, vec![("fixture".into(), PackStatus::Installed)]);
    assert_eq!(
        fs::read(f.root.join("assets/x/hello.txt")).unwrap(),
        b"hello v2"
    );
    assert_eq!(
        fs::read(f.root.join("assets/x/fresh.txt")).unwrap(),
        b"new in v2"
    );
    assert!(
        !f.root.join("assets/x/sub/data.bin").exists(),
        "obsolete file must be pruned"
    );
    assert!(
        !f.root.join("assets/x/sub").exists(),
        "emptied directories must be cleaned up"
    );
}

#[test]
fn unknown_preexisting_file_refused_then_adopted() {
    let f = fixture(MEMBERS, None);
    // A file at an EXACT pack destination, not owned by any installed version.
    fs::create_dir_all(f.root.join("assets/x")).unwrap();
    fs::write(f.root.join("assets/x/hello.txt"), b"user's own file").unwrap();

    let results = run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    match &results[0].1 {
        PackStatus::Failed(msg) => {
            assert!(msg.contains("unknown preexisting"), "got: {msg}");
            assert!(
                msg.contains("--adopt"),
                "must name the override; got: {msg}"
            );
        }
        other => panic!("expected Failed, got {other:?}"),
    }
    assert_eq!(
        fs::read(f.root.join("assets/x/hello.txt")).unwrap(),
        b"user's own file",
        "refusal must not overwrite"
    );

    let opts = FetchOptions {
        selection: Selection::All,
        adopt: true,
        ..Default::default()
    };
    let results = run_fetch(&f.manifest_path, &f.root, &opts).unwrap();
    assert_eq!(results, vec![("fixture".into(), PackStatus::Installed)]);
    assert_eq!(
        fs::read(f.root.join("assets/x/hello.txt")).unwrap(),
        b"hello packs"
    );
}

#[test]
fn tracked_member_refused_at_preflight() {
    let f = fixture(MEMBERS, None);
    // A pack MEMBER path is git-tracked (the pre-rewrite tree shape, or an
    // accidental re-add). Member-exact refusal — a tracked NON-member cohabitant
    // (License.txt-style) must NOT trip it.
    fs::create_dir_all(f.root.join("assets/x")).unwrap();
    fs::write(f.root.join("assets/x/hello.txt"), b"committed content").unwrap();
    git(&f.root, &["add", "-f", "assets/x/hello.txt"]);
    git(&f.root, &["commit", "-q", "-m", "track a member path"]);

    let results = run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    match &results[0].1 {
        PackStatus::Failed(msg) => assert!(msg.contains("git-tracked"), "got: {msg}"),
        other => panic!("expected Failed, got {other:?}"),
    }
    assert_eq!(
        fs::read(f.root.join("assets/x/hello.txt")).unwrap(),
        b"committed content",
        "no work may have happened"
    );
    assert!(!f.state("stamps/fixture.toml").exists());
}

#[test]
fn tracked_cohabitant_does_not_block_install() {
    let f = fixture(MEMBERS, None);
    // A tracked NON-member under the shared root (the License.txt pattern).
    fs::create_dir_all(f.root.join("assets/x")).unwrap();
    fs::write(f.root.join("assets/x/License.txt"), b"license evidence").unwrap();
    git(&f.root, &["add", "-f", "assets/x/License.txt"]);
    git(
        &f.root,
        &["commit", "-q", "-m", "keep license evidence tracked"],
    );

    let results = run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    assert_eq!(results, vec![("fixture".into(), PackStatus::Installed)]);
    assert!(f.root.join("assets/x/License.txt").exists());
    assert!(f.root.join("assets/x/hello.txt").exists());
}

#[test]
fn missing_ignore_rules_refused_at_preflight() {
    let f = fixture(MEMBERS, None);
    // Remove the generated ignore (and commit, so check-ignore truly fails).
    fs::remove_file(f.root.join(GENERATED_IGNORE)).unwrap();
    git(&f.root, &["add", "-A"]);
    git(&f.root, &["commit", "-q", "-m", "drop ignore"]);

    let err = run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap_err();
    assert!(err.to_string().contains("not covered"), "got: {err:#}");
}

#[test]
fn journal_roll_forward_completes_interrupted_install() {
    let f = fixture(MEMBERS, None);
    // Construct the crash state between steps 6→7: staging fully populated,
    // journal present (phase staged), nothing at destinations, no stamp.
    let staging = f.state("staging/fixture");
    for (name, contents) in MEMBERS {
        let p = staging.join(name);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, contents).unwrap();
    }
    let sha = sha256_hex(&f.zip_path);
    let journal = format!(
        "name = \"fixture\"\nsha256 = \"{sha}\"\nphase = \"staged\"\nfiles = [\"assets/x/hello.txt\", \"assets/x/sub/data.bin\"]\n"
    );
    let jpath = f.state("journal/fixture.toml");
    fs::create_dir_all(jpath.parent().unwrap()).unwrap();
    fs::write(&jpath, journal).unwrap();

    let results = run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    // Recovery rolls forward and stamps the journaled version; the normal flow then
    // sees a matching stamp → UpToDate. No re-download happened (staging was intact).
    assert_eq!(results, vec![("fixture".into(), PackStatus::UpToDate)]);
    assert_eq!(
        fs::read(f.root.join("assets/x/hello.txt")).unwrap(),
        b"hello packs"
    );
    assert!(f.root.join("assets/x/sub/data.bin").exists());
    assert!(f.state("stamps/fixture.toml").exists());
    assert!(!jpath.exists(), "journal must be cleared");
    assert!(!staging.exists(), "staging must be cleaned");
}

#[test]
fn journal_refetch_when_staging_lost() {
    let f = fixture(MEMBERS, None);
    // Crash state between 7→9 with staging LOST: journal present, one file already
    // placed at its destination, no staging dir.
    fs::create_dir_all(f.root.join("assets/x")).unwrap();
    fs::write(
        f.root.join("assets/x/hello.txt"),
        b"partial from torn install",
    )
    .unwrap();
    let sha = sha256_hex(&f.zip_path);
    let journal = format!(
        "name = \"fixture\"\nsha256 = \"{sha}\"\nphase = \"renaming\"\nfiles = [\"assets/x/hello.txt\", \"assets/x/sub/data.bin\"]\n"
    );
    let jpath = f.state("journal/fixture.toml");
    fs::create_dir_all(jpath.parent().unwrap()).unwrap();
    fs::write(&jpath, journal).unwrap();

    let results = run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    // Recovery sweeps the partial file, then the normal flow re-fetches cleanly.
    assert_eq!(results, vec![("fixture".into(), PackStatus::Installed)]);
    assert_eq!(
        fs::read(f.root.join("assets/x/hello.txt")).unwrap(),
        b"hello packs",
        "re-fetch must restore pristine zip content"
    );
    assert!(!jpath.exists());
}

#[test]
fn retry_on_access_denied_engages_and_gives_up_correctly() {
    // Fails twice with os error 5, then succeeds → 3 attempts total.
    let mut attempts = 0;
    let result: std::io::Result<u32> = retry_on_access_denied(|| {
        attempts += 1;
        if attempts < 3 {
            Err(std::io::Error::from_raw_os_error(5))
        } else {
            Ok(42)
        }
    });
    assert_eq!(result.unwrap(), 42);
    assert_eq!(attempts, 3, "retry must engage on os error 5");

    // A different error is NOT retried.
    let mut attempts = 0;
    let result: std::io::Result<()> = retry_on_access_denied(|| {
        attempts += 1;
        Err(std::io::Error::from_raw_os_error(2)) // ENOENT
    });
    assert!(result.is_err());
    assert_eq!(attempts, 1, "non-access-denied errors must not retry");

    // Persistent os error 5 gives up after 5 attempts.
    let mut attempts = 0;
    let result: std::io::Result<()> = retry_on_access_denied(|| {
        attempts += 1;
        Err(std::io::Error::from_raw_os_error(5))
    });
    assert!(result.is_err());
    assert_eq!(attempts, 5, "must give up after the 5th attempt");
}

#[test]
fn profiles_starter_default_and_all() {
    // Two packs; starter contains only the first.
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    let zip_a = root.join("a.zip");
    let zip_b = root.join("b.zip");
    build_pack_zip(&zip_a, &[("assets/a/one.txt", b"a")]);
    build_pack_zip(&zip_b, &[("assets/b/two.txt", b"b")]);
    let manifest = Manifest {
        schema_version: SCHEMA_VERSION,
        packs: vec![
            PackEntry {
                name: "pack-a".into(),
                source: zip_a.to_string_lossy().into_owned(),
                sha256: sha256_hex(&zip_a),
                size_bytes: fs::metadata(&zip_a).unwrap().len(),
                roots: vec!["assets/a/".into()],
            },
            PackEntry {
                name: "pack-b".into(),
                source: zip_b.to_string_lossy().into_owned(),
                sha256: sha256_hex(&zip_b),
                size_bytes: fs::metadata(&zip_b).unwrap().len(),
                roots: vec!["assets/b/".into()],
            },
        ],
        profiles: BTreeMap::from([
            ("starter".into(), vec!["pack-a".into()]),
            ("all".into(), vec!["*".into()]),
        ]),
    };
    fs::create_dir_all(root.join("assets")).unwrap();
    let manifest_path = root.join("assets/packs.manifest.toml");
    fs::write(&manifest_path, toml::to_string_pretty(&manifest).unwrap()).unwrap();
    fs::write(
        root.join(GENERATED_IGNORE),
        generated_ignore_content(&manifest),
    )
    .unwrap();
    write_packlist_at(&root, "pack-a", &["assets/a/one.txt"]);
    write_packlist_at(&root, "pack-b", &["assets/b/two.txt"]);
    git(&root, &["init", "-q"]);
    git(&root, &["config", "user.name", "fixture"]);
    git(&root, &["config", "user.email", "fixture@test"]);
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "init"]);

    // Default selection = starter profile → only pack-a.
    let results = run_fetch(&manifest_path, &root, &FetchOptions::default()).unwrap();
    assert_eq!(results, vec![("pack-a".into(), PackStatus::Installed)]);
    assert!(root.join("assets/a/one.txt").exists());
    assert!(!root.join("assets/b/two.txt").exists());

    // `all` profile via wildcard → both (pack-a now up-to-date).
    let opts = FetchOptions {
        selection: Selection::Profile("all".into()),
        ..Default::default()
    };
    let results = run_fetch(&manifest_path, &root, &opts).unwrap();
    assert_eq!(
        results,
        vec![
            ("pack-a".into(), PackStatus::UpToDate),
            ("pack-b".into(), PackStatus::Installed),
        ]
    );
    assert!(root.join("assets/b/two.txt").exists());

    // Unknown profile → clear error.
    let bad = FetchOptions {
        selection: Selection::Profile("nope".into()),
        ..Default::default()
    };
    let err = run_fetch(&manifest_path, &root, &bad).unwrap_err();
    assert!(err.to_string().contains("no profile 'nope'"));
}

#[test]
fn inter_pack_member_collision_rejected() {
    // Shared root containers are LEGAL (the textures-environment splits); co-owning
    // the same FILE is not. The disjointness check is packlist-exact.
    let f = fixture(MEMBERS, None);
    let mut m = load_manifest(&f.manifest_path).unwrap();
    let mut second = m.packs[0].clone();
    second.name = "overlapper".into();
    second.roots = vec!["assets/x/sub/".into()]; // nested root: allowed
    m.packs.push(second);
    f.rewrite_manifest(&m);
    // overlapper's packlist claims a file that fixture's packlist also claims.
    f.write_packlist("overlapper", &["assets/x/sub/data.bin"]);

    let err = run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap_err();
    assert!(err.to_string().contains("member collision"), "got: {err:#}");
}

#[test]
fn lock_refuses_second_instance() {
    let f = fixture(MEMBERS, None);
    let lock = f.state("lock");
    fs::create_dir_all(lock.parent().unwrap()).unwrap();
    fs::write(&lock, b"pid 99999").unwrap();

    let err = run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap_err();
    assert!(
        err.to_string().contains("installation lock"),
        "got: {err:#}"
    );
}

#[test]
fn gen_ignore_generates_and_ci_guard_enforces() {
    let f = fixture(MEMBERS, None);
    let m = load_manifest(&f.manifest_path).unwrap();
    let content = generated_ignore_content(&m);
    assert!(content.contains("/.pack-state/"));
    assert!(
        content.contains("/x/"),
        "root assets/x/ → /x/ in assets/.gitignore"
    );

    // gen-ignore --check passes on the fixture (ignore written at fixture build).
    gen_ignore(&f.manifest_path, &f.root, true).unwrap();
    // ci-guard passes on the clean fixture.
    ci_guard(&f.manifest_path, &f.root).unwrap();

    // Drift → --check fails.
    fs::write(f.root.join(GENERATED_IGNORE), "# hand-edited\n").unwrap();
    assert!(gen_ignore(&f.manifest_path, &f.root, true).is_err());
    gen_ignore(&f.manifest_path, &f.root, false).unwrap(); // regenerate

    // A tracked pack MEMBER → ci-guard fails (the classic re-bloat: force-adding
    // restored pack content).
    fs::create_dir_all(f.root.join("assets/x")).unwrap();
    fs::write(f.root.join("assets/x/hello.txt"), b"re-added pack file").unwrap();
    git(&f.root, &["add", "-f", "assets/x/hello.txt"]);
    git(&f.root, &["commit", "-q", "-m", "re-add pack content"]);
    let err = ci_guard(&f.manifest_path, &f.root).unwrap_err();
    assert!(err.to_string().contains("pack member file"), "got: {err:#}");
    git(&f.root, &["rm", "-q", "--cached", "assets/x/hello.txt"]);
    git(&f.root, &["commit", "-q", "-m", "untrack again"]);
    fs::remove_file(f.root.join("assets/x/hello.txt")).unwrap();

    // A stray NEW tracked blob under a root (not a member, not keeplisted) → fails…
    fs::write(f.root.join("assets/x/sneaky.bin"), b"re-bloat").unwrap();
    git(&f.root, &["add", "-f", "assets/x/sneaky.bin"]);
    git(&f.root, &["commit", "-q", "-m", "sneak a blob in"]);
    let err = ci_guard(&f.manifest_path, &f.root).unwrap_err();
    assert!(
        err.to_string()
            .contains("neither pack members nor keeplist"),
        "got: {err:#}"
    );
    // …until deliberately keeplisted (gen-keeplist subset semantics).
    gen_keeplist(&f.manifest_path, &f.root).unwrap();
    ci_guard(&f.manifest_path, &f.root).unwrap();
}

#[test]
fn verify_assets_flags_missing_ledger_files() {
    let f = fixture(MEMBERS, None);
    run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    verify_assets_cmd(&f.manifest_path, &f.root).unwrap();

    fs::remove_file(f.root.join("assets/x/hello.txt")).unwrap();
    let err = verify_assets_cmd(&f.manifest_path, &f.root).unwrap_err();
    assert!(
        err.to_string().contains("verify-assets FAILED"),
        "got: {err:#}"
    );
}

#[test]
fn three_run_loop_fresh_noop_repair() {
    // The AD.2 protocol re-expressed under AD.5 semantics.
    let f = fixture(MEMBERS, None);
    // Run 1: fresh install.
    let r1 = run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    assert_eq!(r1, vec![("fixture".into(), PackStatus::Installed)]);
    // Run 2: stamped no-op.
    let r2 = run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    assert_eq!(r2, vec![("fixture".into(), PackStatus::UpToDate)]);
    // Run 3: corrupted stamp → repair (reinstall).
    let stamp_path = f.state("stamps/fixture.toml");
    let corrupted = fs::read_to_string(&stamp_path)
        .unwrap()
        .replace(&sha256_hex(&f.zip_path), &"e".repeat(64));
    fs::write(&stamp_path, corrupted).unwrap();
    let r3 = run_fetch(&f.manifest_path, &f.root, &f.opts()).unwrap();
    assert_eq!(r3, vec![("fixture".into(), PackStatus::Installed)]);
}

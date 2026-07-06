//! AD.2 Phase 3 — fixture-based tests for `cargo xtask fetch-assets`. No network:
//! every fixture zip is built in a tempdir at test time and served via a local-path
//! `source` (the same code path AD.3 will use to pre-verify packs before upload).

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use xtask::fetch_assets::{
    is_safe_relative, load_manifest, run_fetch, FetchOptions, Manifest, PackEntry, PackStatus,
};

/// Build a zip at `path` with the given (member name, contents) pairs.
fn build_zip(path: &Path, members: &[(&str, &[u8])]) {
    let file = File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let opts: zip::write::SimpleFileOptions = Default::default();
    for (name, contents) in members {
        zip.start_file(*name, opts).unwrap();
        zip.write_all(contents).unwrap();
    }
    zip.finish().unwrap();
}

fn sha256_hex(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(path).unwrap();
    hex::encode(Sha256::digest(&bytes))
}

/// Standard happy-path fixture: one zip, one manifest entry with a local source.
struct Fixture {
    _tmp: TempDir,
    manifest_path: PathBuf,
    packs_root: PathBuf,
    zip_path: PathBuf,
}

fn fixture(zip_members: &[(&str, &[u8])], sha256_override: Option<&str>) -> Fixture {
    let tmp = TempDir::new().unwrap();
    let zip_path = tmp.path().join("fixture_pack.zip");
    build_zip(&zip_path, zip_members);
    let sha = match sha256_override {
        Some(s) => s.to_string(),
        None => sha256_hex(&zip_path),
    };
    let manifest = Manifest {
        schema_version: 1,
        packs: vec![PackEntry {
            name: "fixture".into(),
            source: zip_path.to_string_lossy().into_owned(),
            sha256: sha,
            unpack_to: "fixture".into(),
            size_bytes: fs::metadata(&zip_path).unwrap().len(),
        }],
    };
    let manifest_path = tmp.path().join("packs.manifest.toml");
    fs::write(&manifest_path, toml::to_string_pretty(&manifest).unwrap()).unwrap();
    Fixture {
        packs_root: tmp.path().join("packs"),
        manifest_path,
        zip_path,
        _tmp: tmp,
    }
}

const MEMBERS: &[(&str, &[u8])] = &[
    ("hello.txt", b"hello packs"),
    ("sub/data.bin", &[0u8, 1, 2, 3, 255]),
];

#[test]
fn manifest_parse_round_trip() {
    let f = fixture(MEMBERS, None);
    let parsed = load_manifest(&f.manifest_path).unwrap();
    assert_eq!(parsed.schema_version, 1);
    assert_eq!(parsed.packs.len(), 1);
    assert_eq!(parsed.packs[0].name, "fixture");
    // round-trip: serialize -> reparse -> equal
    let reserialized = toml::to_string_pretty(&parsed).unwrap();
    let reparsed: Manifest = toml::from_str(&reserialized).unwrap();
    assert_eq!(parsed, reparsed);
}

#[test]
fn happy_path_fetch_unpack_stamp() {
    let f = fixture(MEMBERS, None);
    let results = run_fetch(&f.manifest_path, &f.packs_root, &FetchOptions::default()).unwrap();
    assert_eq!(results, vec![("fixture".into(), PackStatus::Fetched)]);
    assert_eq!(
        fs::read(f.packs_root.join("fixture/hello.txt")).unwrap(),
        b"hello packs"
    );
    assert!(f.packs_root.join("fixture/sub/data.bin").exists());
    assert!(f.packs_root.join(".stamps/fixture.toml").exists());
}

#[test]
fn checksum_mismatch_rejected_loudly() {
    let bogus = "0".repeat(64);
    let f = fixture(MEMBERS, Some(&bogus));
    let results = run_fetch(&f.manifest_path, &f.packs_root, &FetchOptions::default()).unwrap();
    match &results[0].1 {
        PackStatus::Failed(msg) => {
            assert!(msg.contains("sha256 mismatch"), "got: {msg}");
            assert!(msg.contains(&bogus), "must name expected hash; got: {msg}");
            assert!(
                msg.contains(&sha256_hex(&f.zip_path)),
                "must name actual hash; got: {msg}"
            );
        }
        other => panic!("expected Failed, got {other:?}"),
    }
    // never unpacked, never stamped, temp cleaned
    assert!(!f.packs_root.join("fixture").exists());
    assert!(!f.packs_root.join(".stamps/fixture.toml").exists());
    let leftovers: Vec<_> = match fs::read_dir(f.packs_root.join(".tmp")) {
        Ok(rd) => rd.map(|e| e.unwrap().file_name()).collect(),
        Err(_) => vec![],
    };
    assert!(leftovers.is_empty(), "temp not cleaned: {leftovers:?}");
}

#[test]
fn second_run_is_idempotent_no_reunpack() {
    let f = fixture(MEMBERS, None);
    run_fetch(&f.manifest_path, &f.packs_root, &FetchOptions::default()).unwrap();
    // Plant a marker inside the unpacked tree; a re-unpack would wipe it.
    let marker = f.packs_root.join("fixture/MARKER.local");
    fs::write(&marker, b"survives idempotent run").unwrap();

    let results = run_fetch(&f.manifest_path, &f.packs_root, &FetchOptions::default()).unwrap();
    assert_eq!(results, vec![("fixture".into(), PackStatus::UpToDate)]);
    assert!(marker.exists(), "second run must not re-unpack");
}

#[test]
fn path_traversal_members_and_unpack_to_rejected() {
    // (a) zip member escaping the destination
    let f = fixture(&[("../evil.txt", b"escape attempt")], None);
    let results = run_fetch(&f.manifest_path, &f.packs_root, &FetchOptions::default()).unwrap();
    match &results[0].1 {
        PackStatus::Failed(msg) => {
            assert!(msg.contains("path traversal"), "got: {msg}")
        }
        other => panic!("expected Failed, got {other:?}"),
    }
    assert!(!f.packs_root.join("fixture").exists());
    // nothing may have escaped above the packs root
    assert!(!f.packs_root.parent().unwrap().join("evil.txt").exists());
    assert!(!f.packs_root.join(".stamps/fixture.toml").exists());

    // (b) manifest unpack_to escaping the packs root
    let f2 = fixture(MEMBERS, None);
    let mut manifest = load_manifest(&f2.manifest_path).unwrap();
    manifest.packs[0].unpack_to = "../escaped".into();
    fs::write(
        &f2.manifest_path,
        toml::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let results = run_fetch(&f2.manifest_path, &f2.packs_root, &FetchOptions::default()).unwrap();
    match &results[0].1 {
        PackStatus::Failed(msg) => assert!(msg.contains("escapes"), "got: {msg}"),
        other => panic!("expected Failed, got {other:?}"),
    }
    assert!(!f2.packs_root.parent().unwrap().join("escaped").exists());

    // (c) the predicate itself, incl. Windows-specific shapes
    for bad in ["../x", "a/../../x", "/abs", "C:/x", "C:\\x", "..", ""] {
        assert!(!is_safe_relative(bad), "must reject {bad:?}");
    }
    for good in ["a", "a/b.txt", "./a/b", "a/./b"] {
        assert!(is_safe_relative(good), "must accept {good:?}");
    }
}

#[test]
fn force_refetches_despite_stamp() {
    let f = fixture(MEMBERS, None);
    run_fetch(&f.manifest_path, &f.packs_root, &FetchOptions::default()).unwrap();
    let marker = f.packs_root.join("fixture/MARKER.local");
    fs::write(&marker, b"will be replaced by --force").unwrap();

    let opts = FetchOptions {
        force: true,
        only_pack: None,
    };
    let results = run_fetch(&f.manifest_path, &f.packs_root, &opts).unwrap();
    assert_eq!(results, vec![("fixture".into(), PackStatus::Fetched)]);
    assert!(
        !marker.exists(),
        "--force must replace the unpacked tree from the archive"
    );
    assert!(f.packs_root.join("fixture/hello.txt").exists());
}

#[test]
fn pack_filter_selects_and_rejects_unknown() {
    let f = fixture(MEMBERS, None);
    let opts = FetchOptions {
        only_pack: Some("fixture".into()),
        force: false,
    };
    let results = run_fetch(&f.manifest_path, &f.packs_root, &opts).unwrap();
    assert_eq!(results.len(), 1);

    let missing = FetchOptions {
        only_pack: Some("nope".into()),
        force: false,
    };
    let err = run_fetch(&f.manifest_path, &f.packs_root, &missing).unwrap_err();
    assert!(err.to_string().contains("no pack named 'nope'"));
    assert!(err.to_string().contains("fixture"), "should list available");
}

#[test]
fn corrupted_stamp_hash_triggers_refetch() {
    let f = fixture(MEMBERS, None);
    run_fetch(&f.manifest_path, &f.packs_root, &FetchOptions::default()).unwrap();
    // Corrupt the stamp's recorded hash (the Phase 4 protocol's third run).
    let stamp_path = f.packs_root.join(".stamps/fixture.toml");
    let corrupted = fs::read_to_string(&stamp_path)
        .unwrap()
        .replace(&sha256_hex(&f.zip_path), &"f".repeat(64));
    fs::write(&stamp_path, corrupted).unwrap();

    let results = run_fetch(&f.manifest_path, &f.packs_root, &FetchOptions::default()).unwrap();
    assert_eq!(results, vec![("fixture".into(), PackStatus::Fetched)]);
}

//! The default suite is pure, and this is where that stops being a sentence.
//!
//! Pure means no display server, no elevated rights, no host font directory and
//! no network. Two things hold it. The scan below refuses a source that names a
//! host font directory or asks a library to load the host's fonts, and it runs in
//! the default suite on every machine. The environment itself holds the rest:
//! `.github/scripts/run-sealed.sh` runs this suite in a container with none of
//! those things present, and the probes at the bottom of this file are what shows
//! that the container really has none of them.
//!
//! The probes are `#[ignore]`d. They are written to pass where a display, a host
//! font directory, a socket and a writable filesystem exist, so a run in which
//! they FAIL is a run in a sealed environment. The gate runs them and requires
//! them to fail. Nothing else runs them.
//!
//! `docs/test-harness.md` argues the rule and `CONTRIBUTING.md` states it in one
//! sentence.

use std::fs;
use std::path::{Path, PathBuf};

/// The things a source in this workspace may not name.
///
/// Each is assembled from two pieces so that this file does not itself contain
/// any of the strings it looks for. Excluding this file by name would have worked
/// too and would have left one file in the tree where the rule does not apply.
fn needles() -> Vec<(&'static str, String)> {
    vec![
        (
            "a host font directory",
            format!("{}{}", "/usr/share/", "fonts"),
        ),
        (
            "a host font directory",
            format!("{}{}", "/usr/local/share/", "fonts"),
        ),
        (
            "a host font directory",
            format!("{}{}", "/System/Library/", "Fonts"),
        ),
        (
            "a host font directory",
            format!("{}{}", "C:\\Windows\\", "Fonts"),
        ),
        (
            "a call that loads the host's fonts",
            format!("{}{}", "load_system_", "fonts"),
        ),
    ]
}

/// Every `.rs` file under a directory, in a stable order.
fn rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(error) => panic!("cannot read {}: {error}", dir.display()),
        };
        for entry in entries {
            let entry = entry.expect("cannot read a directory entry");
            let path = entry.path();
            let kind = entry.file_type().expect("cannot read a file type");
            if kind.is_dir() {
                // Build output is derived from the sources beside it and is not
                // itself tracked, so scanning it would report the same finding
                // twice or a finding about a file nobody wrote.
                if entry.file_name() == "target" {
                    continue;
                }
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

/// What a source reaching for the host's fonts looks like, as lines a reader can
/// act on.
fn host_font_references(root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for path in rust_sources(root) {
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => panic!("cannot read {}: {error}", path.display()),
        };
        for (line_number, line) in text.lines().enumerate() {
            for (what, needle) in needles() {
                if line.contains(&needle) {
                    found.push(format!(
                        "{}:{} names {what}. A test's fonts come from this \
                         repository, never from whatever the host happens to \
                         have installed.",
                        path.display(),
                        line_number + 1
                    ));
                }
            }
        }
    }
    found.sort();
    found
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn no_source_reaches_for_the_host_s_fonts() {
    let found = host_font_references(&workspace_root().join("crates"));
    assert!(
        found.is_empty(),
        "a source in this workspace reaches outside it for fonts:\n{}",
        found
            .iter()
            .map(|line| format!("  {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// A scratch directory that removes itself.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "rechenblatt-environment-{}-{label}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("cannot create a scratch directory");
        Scratch(dir)
    }

    fn write(&self, name: &str, contents: &str) -> &Self {
        fs::write(self.0.join(name), contents).expect("cannot write a scratch file");
        self
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn the_font_scan_refuses_a_source_that_names_a_host_font_directory() {
    let scratch = Scratch::new("host-font-directory");
    scratch.write(
        "reaching.rs",
        &format!(
            "fn load() {{ let _ = std::fs::read_dir(\"{}{}\"); }}\n",
            "/usr/share/", "fonts"
        ),
    );
    let found = host_font_references(&scratch.0);
    assert_eq!(
        found.len(),
        1,
        "expected exactly one finding, got {found:?}"
    );
    assert!(found[0].contains("reaching.rs:1"), "{}", found[0]);
}

#[test]
fn the_font_scan_refuses_a_source_that_asks_a_library_for_them() {
    let scratch = Scratch::new("system-font-call");
    scratch.write(
        "reaching.rs",
        &format!(
            "fn load(db: &mut Db) {{ db.{}{}(); }}\n",
            "load_system_", "fonts"
        ),
    );
    assert_eq!(host_font_references(&scratch.0).len(), 1);
}

#[test]
fn the_font_scan_passes_a_source_that_reads_a_font_from_this_repository() {
    let scratch = Scratch::new("repository-font");
    scratch.write(
        "reading.rs",
        "fn load() { let _ = std::fs::read(\"tests/fixtures/one.ttf\"); }\n",
    );
    assert_eq!(host_font_references(&scratch.0), Vec::<String>::new());
}

#[test]
fn a_temporary_path_is_one_the_test_created() {
    let scratch = Scratch::new("temporary-path");
    scratch.write("note.txt", "written under a directory this test made");
    assert!(
        scratch.0.starts_with(std::env::temp_dir()),
        "{} is not under the temporary directory this run was given",
        scratch.0.display()
    );
    assert!(scratch.0.join("note.txt").is_file());
}

// The probes.
//
// Each one asserts that something exists which a sealed environment does not
// have, so each one FAILS there. That inversion is deliberate: it means the gate
// is reading the environment rather than reading a test that was written to agree
// with it. `.github/scripts/run-sealed.sh probes` runs them and requires every one
// to fail, and requires each failure to carry a cause rather than a timeout.
//
// What is NOT claimed is that every probe passes on an arbitrary machine. Three
// of them do wherever the thing they name is present, and probe/write-outside
// needs a machine where the calling user may write under /usr/local, which a
// contributor's own account usually may not. That does not weaken the gate: what
// the gate reads is the failure, and the marker plus the cause on it is what says
// the failure came from the thing being absent.
//
// Every message begins with its own marker, because the gate greps for those
// markers rather than for a count.

#[test]
#[ignore = "a probe of the environment; the headless gate runs it and requires it to fail"]
fn probe_a_display_server_is_reachable() {
    let display = std::env::var("DISPLAY").ok();
    let wayland = std::env::var("WAYLAND_DISPLAY").ok();
    assert!(
        display.is_some() || wayland.is_some(),
        "probe/display: neither DISPLAY nor WAYLAND_DISPLAY is set, so nothing \
         here could reach a display server"
    );
}

#[test]
#[ignore = "a probe of the environment; the headless gate runs it and requires it to fail"]
fn probe_the_host_has_fonts_installed() {
    // A font file rather than a directory. An image can carry the directory and
    // not one font in it, and a probe reading only whether the directory opens
    // would pass there and report the environment unsealed for a reason that has
    // nothing to do with fonts. What a renderer could actually take from the host
    // is a font file, so that is what is asked for.
    let dirs = [
        format!("{}{}", "/usr/share/", "fonts"),
        format!("{}{}", "/usr/local/share/", "fonts"),
    ];

    let mut found = Vec::new();
    let mut stack: Vec<PathBuf> = dirs.iter().map(PathBuf::from).collect();
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| matches!(ext, "ttf" | "otf" | "ttc" | "pfb" | "pfa"))
            {
                found.push(path);
            }
        }
    }

    assert!(
        !found.is_empty(),
        "probe/host-fonts: no font file is installed under {}, so nothing here \
         could load a font the host supplied",
        dirs.join(" or ")
    );
}

#[test]
#[ignore = "a probe of the environment; the headless gate runs it and requires it to fail"]
fn probe_a_network_socket_can_be_opened() {
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

    // An address rather than a name, so a failure is a routing failure and not a
    // resolver timeout, and a short timeout so it is neither.
    let address: SocketAddr = "1.1.1.1:443".parse().expect("a literal address");
    if let Err(error) = TcpStream::connect_timeout(&address, Duration::from_secs(3)) {
        panic!("probe/network: {address} could not be reached: {error}");
    }
}

#[test]
#[ignore = "a probe of the environment; the headless gate runs it and requires it to fail"]
fn probe_a_file_can_be_written_outside_the_scratch_directory() {
    let path = Path::new("/usr/local/rechenblatt-probe");
    let written = fs::write(path, b"probe");
    let _ = fs::remove_file(path);
    if let Err(error) = written {
        panic!(
            "probe/write-outside: {} could not be written: {error}",
            path.display()
        );
    }
}

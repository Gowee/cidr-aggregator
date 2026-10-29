//! Integration tests exercising the full pipeline from parse → aggregate → normalize → export.

use cidr_aggregator::aggregator::Aggregator;
use cidr_aggregator::{parse_cidrs, IpRange, Ipv4Range};

#[test]
fn full_pipeline_single_v4() {
    let (mut v4, v6, invalid) = parse_cidrs("10.0.0.0/24");
    assert!(v6.is_empty());
    assert!(invalid.is_empty());

    v4.aggregate();
    v4.normalize();
    assert_eq!(v4.export(), "10.0.0.0/24");
}

#[test]
fn full_pipeline_merge_adjacent() {
    let (mut v4, _, _) = parse_cidrs("10.0.0.0/24\n10.0.1.0/24");
    v4.aggregate();
    v4.normalize();
    assert_eq!(v4.export(), "10.0.0.0/23");
}

#[test]
fn full_pipeline_with_comments_and_invalid() {
    let input = "# block list\n10.0.0.0/8\n# internal\n192.168.0.0/16\nbad-line\n172.16.0.0/12";
    let (mut v4, _v6, invalid) = parse_cidrs(input);
    assert_eq!(invalid.len(), 1);
    assert_eq!(invalid[0], "bad-line");

    v4.aggregate();
    v4.normalize();
    let output = v4.export();
    assert!(output.contains("10.0.0.0/8"));
    assert!(output.contains("172.16.0.0/12"));
    assert!(output.contains("192.168.0.0/16"));
}

#[test]
fn full_pipeline_reverse() {
    let (mut v4, _, _) = parse_cidrs("10.0.0.0/8");
    v4.aggregate();
    v4.reverse();
    v4.normalize();
    // The complement of 10.0.0.0/8 should contain 0.0.0.0/5 (or similar),
    // but not contain 10.0.0.0/8
    let output = v4.export();
    assert!(!output.contains("10.0.0.0/8"));
    assert!(v4.count_address() > 0u32);
}

#[test]
fn full_pipeline_exclude_reserved() {
    let (mut v4, _, _) = parse_cidrs("10.0.0.0/8");
    v4.aggregate();
    v4.difference(Ipv4Range::reserved());
    // 10.0.0.0/8 is itself in the reserved list, so it should be removed
    assert!(v4.is_empty());
}

#[test]
fn full_pipeline_ipv6() {
    let (_, mut v6, _) = parse_cidrs("2001:db8::/32\n2001:db8:1::/48");
    v6.aggregate();
    v6.normalize();
    // The /48 is contained in /32
    assert_eq!(v6.export(), "2001:db8::/32");
}

#[test]
fn full_pipeline_mixed() {
    let (mut v4, mut v6, invalid) =
        parse_cidrs("10.0.0.0/24\n10.0.1.0/24\n2001:db8::/32\n2001:db8:1::/48");
    assert!(invalid.is_empty());

    v4.aggregate();
    v4.normalize();
    v6.aggregate();
    v6.normalize();

    assert_eq!(v4.export(), "10.0.0.0/23");
    assert_eq!(v6.export(), "2001:db8::/32");
}

#[test]
fn aggregate_idempotent() {
    let (mut v4, _, _) = parse_cidrs("10.0.2.0/24\n10.0.0.0/24\n10.0.1.0/24");
    v4.aggregate();
    let once = v4.clone();
    v4.aggregate();
    // Aggregate twice should produce the same result as aggregate once
    assert_eq!(once, v4);
}

#[test]
fn normalize_idempotent() {
    let (mut v4, _, _) = parse_cidrs("10.0.0.0/24\n10.0.1.0/24");
    v4.aggregate();
    v4.normalize();
    let once = v4.export();
    v4.normalize();
    let twice = v4.export();
    assert_eq!(once, twice);
}

#[test]
fn empty_input() {
    let (v4, v6, invalid) = parse_cidrs("");
    assert!(v4.is_empty());
    assert!(v6.is_empty());
    assert!(invalid.is_empty());
}

#[test]
fn bare_ip_treated_as_host() {
    let (mut v4, _, _) = parse_cidrs("10.0.0.1");
    assert_eq!(v4.len(), 1);
    v4.normalize();
    assert_eq!(v4.export(), "10.0.0.1/32");
}

#[test]
fn full_range() {
    let (mut v4, _, _) = parse_cidrs("0.0.0.0/0");
    v4.aggregate();
    v4.normalize();
    assert_eq!(v4.export(), "0.0.0.0/0");
    assert_eq!(v4.len(), 1);
}

#[test]
fn full_pipeline_with_exclude() {
    let (v4, _, _) = parse_cidrs("10.0.0.0/16");
    let (v4_exclude, _, _) = parse_cidrs("10.0.1.0/24");
    let res = cidr_aggregator::aggregator::process(v4, false, false, &v4_exclude);
    assert_eq!(res.len(), 8);
    assert_eq!(res.count_address(), (1u32 << 16) - 256);
    assert!(!res.contains(&Ipv4Range::from_cidr_pair_decimal((0x0a000100, 0x0a0001ff))));
}

#[cfg(feature = "cli")]
mod cli_tests {
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    fn bin_cmd() -> Command {
        Command::new(env!("CARGO_BIN_EXE_cidr-aggregator"))
    }

    struct TempFile(PathBuf);
    impl TempFile {
        fn new(name: &str, content: &str) -> Self {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "cidr_test_{}_{}_{}",
                std::process::id(),
                id,
                name
            ));
            std::fs::write(&path, content).unwrap();
            Self(path)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    #[test]
    fn cli_exclude_file() {
        let input_file = TempFile::new("input.txt", "# allow\n10.0.0.0/23\n");
        let exclude_file = TempFile::new("exclude.txt", "# block\n10.0.1.0/24\n");

        let output = bin_cmd()
            .arg("-f")
            .arg(input_file.path())
            .arg("-x")
            .arg(exclude_file.path())
            .output()
            .unwrap();

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout.trim(), "10.0.0.0/24");
    }

    #[test]
    fn cli_multiple_exclude_files() {
        let input_file = TempFile::new("input.txt", "10.0.0.0/23\n");
        let exclude_file1 = TempFile::new("ex1.txt", "10.0.1.0/24\n");
        let exclude_file2 = TempFile::new("ex2.txt", "10.0.0.0/25\n");

        let output = bin_cmd()
            .arg("-f")
            .arg(input_file.path())
            .arg("-x")
            .arg(exclude_file1.path())
            .arg("-x")
            .arg(exclude_file2.path())
            .output()
            .unwrap();

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout.trim(), "10.0.0.128/25");
    }

    #[test]
    fn cli_stdin_dash_and_stdout_dash() {
        let mut child = bin_cmd()
            .arg("-f")
            .arg("-")
            .arg("-o")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"10.0.0.0/24\n10.0.1.0/24\n")
            .unwrap();

        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout.trim(), "10.0.0.0/23");
    }

    #[test]
    fn cli_exclude_reserved_capital_x() {
        let mut child = bin_cmd()
            .arg("-X")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"10.0.0.0/24\n1.1.1.0/24\n")
            .unwrap();

        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout.trim(), "1.1.1.0/24");
    }

    #[test]
    fn cli_exclude_missing_file_fails() {
        let output = bin_cmd()
            .arg("-f")
            .arg("-")
            .arg("-x")
            .arg("this_file_does_not_exist_at_all.txt")
            .stdin(Stdio::piped())
            .output()
            .unwrap();

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Failed to read exclude file"));
    }

    #[test]
    fn cli_exclude_invalid_lines_fails_by_default_succeeds_with_ignore() {
        let input_file = TempFile::new("input.txt", "1.1.1.0/24\n");
        let exclude_file = TempFile::new("bad_exclude.txt", "invalid-entry\n");

        // Should fail without -i
        let output_fail = bin_cmd()
            .arg("-f")
            .arg(input_file.path())
            .arg("-x")
            .arg(exclude_file.path())
            .output()
            .unwrap();
        assert!(!output_fail.status.success());

        // Should succeed with -i
        let output_ok = bin_cmd()
            .arg("-f")
            .arg(input_file.path())
            .arg("-x")
            .arg(exclude_file.path())
            .arg("-i")
            .output()
            .unwrap();
        assert!(output_ok.status.success());
        let stdout = String::from_utf8(output_ok.stdout).unwrap();
        assert_eq!(stdout.trim(), "1.1.1.0/24");
    }

    #[test]
    fn cli_multiline_input_and_multiline_exclude() {
        let input_content = "\
# IPv4 ranges
10.0.0.0/22
192.168.0.0/23

# IPv6 ranges
2001:db8::/48
";
        let exclude_content1 = "\
# Exclude some IPv4 subnets
10.0.1.0/24
10.0.2.0/24

# Exclude an IPv6 subnet
2001:db8:0:1::/64
";
        let exclude_content2 = "\
# Exclude another IPv4 subnet from the second block
192.168.1.0/24
";

        let input_file = TempFile::new("input.txt", input_content);
        let ex1 = TempFile::new("ex1.txt", exclude_content1);
        let ex2 = TempFile::new("ex2.txt", exclude_content2);

        let output = bin_cmd()
            .arg("-f")
            .arg(input_file.path())
            .arg("-x")
            .arg(ex1.path())
            .arg("-x")
            .arg(ex2.path())
            .output()
            .unwrap();

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();

        // 10.0.0.0/22 minus 10.0.1.0/24 and 10.0.2.0/24 leaves 10.0.0.0/24 and 10.0.3.0/24
        assert!(stdout.contains("10.0.0.0/24"));
        assert!(stdout.contains("10.0.3.0/24"));
        assert!(!stdout.contains("10.0.1.0/24"));
        assert!(!stdout.contains("10.0.2.0/24"));

        // 192.168.0.0/23 minus 192.168.1.0/24 leaves 192.168.0.0/24
        assert!(stdout.contains("192.168.0.0/24"));
        assert!(!stdout.contains("192.168.1.0/24"));

        // 2001:db8::/48 minus 2001:db8:0:1::/64 has the /64 removed
        assert!(!stdout.contains("2001:db8:0:1::/64"));
    }
}

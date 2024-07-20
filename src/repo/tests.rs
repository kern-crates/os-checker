use super::Config;
use crate::layout::Package;
use expect_test::expect;

const YAML: &str = "
os-checker/os-checker:
  fmt: true
  clippy: cargo clippy -F a,b,c
  miri: |
    # this is a comment line
    cargo miri run # a comment
    cargo miri test -- a_test_fn
  semver-checks: false
  # non-existent key-value pair is ignored
  non-existent: pair

user/repo: 
  all: true # enable all tools for all packages, but ...
  lockbud: false # except lockbud for all packages
  packages: # packages are the union of all members across all workspaces
    crate1: 
      miri: false # except miri for crate1
    crate2:
      semver-checks: false # except semver-checks for crate2
";

#[test]
fn parse() {
    let parsed = Config::from_yaml(YAML).unwrap();
    let expected = expect![[r#"
        [
            Config {
                repo: "os-checker/os-checker",
                config: RepoConfig {
                    fmt: Perform(
                        true,
                    ),
                    clippy: Lines(
                        [
                            "cargo clippy -F a,b,c",
                        ],
                    ),
                    miri: Lines(
                        [
                            "cargo miri run",
                            "cargo miri test -- a_test_fn",
                        ],
                    ),
                    semver-checks: Perform(
                        false,
                    ),
                },
            },
            Config {
                repo: "user/repo",
                config: RepoConfig {
                    all: Perform(
                        true,
                    ),
                    lockbud: Perform(
                        false,
                    ),
                    packages: {
                        "crate1": RepoConfig {
                            miri: Perform(
                                false,
                            ),
                        },
                        "crate2": RepoConfig {
                            semver-checks: Perform(
                                false,
                            ),
                        },
                    },
                },
            },
        ]
    "#]];
    expected.assert_debug_eq(&parsed);

    let v: Vec<_> = parsed
        .iter()
        .map(|c| (&c.repo, c.config.checker_action()))
        .collect();
    let expected = expect![[r#"
        [
            (
                "os-checker/os-checker",
                [
                    (
                        Fmt,
                        Perform(
                            true,
                        ),
                    ),
                    (
                        Clippy,
                        Lines(
                            [
                                "cargo clippy -F a,b,c",
                            ],
                        ),
                    ),
                    (
                        Miri,
                        Lines(
                            [
                                "cargo miri run",
                                "cargo miri test -- a_test_fn",
                            ],
                        ),
                    ),
                    (
                        SemverChecks,
                        Perform(
                            false,
                        ),
                    ),
                ],
            ),
            (
                "user/repo",
                [
                    (
                        All,
                        Perform(
                            true,
                        ),
                    ),
                    (
                        Lockbud,
                        Perform(
                            false,
                        ),
                    ),
                    (
                        Miri,
                        Perform(
                            false,
                        ),
                    ),
                    (
                        SemverChecks,
                        Perform(
                            false,
                        ),
                    ),
                ],
            ),
        ]
    "#]];
    expected.assert_debug_eq(&v);
}

#[test]
fn pkg_checker_action() {
    let parsed = Config::from_yaml(YAML).unwrap();
    let v = parsed[0]
        .config
        .pkg_checker_action(&[Package::test_new("package1"), Package::test_new("package2")])
        .unwrap();
    expect![[r#"
        [
            (
                Package {
                    name: "package1",
                    cargo_toml: "./Cargo.toml",
                    workspace_root (file name): "unknown???",
                },
                Cmd(
                    [
                        "cargo",
                        "fmt",
                        "--check",
                        "--manifest-path",
                        "./Cargo.toml",
                    ],
                ),
            ),
            (
                Package {
                    name: "package2",
                    cargo_toml: "./Cargo.toml",
                    workspace_root (file name): "unknown???",
                },
                Cmd(
                    [
                        "cargo",
                        "fmt",
                        "--check",
                        "--manifest-path",
                        "./Cargo.toml",
                    ],
                ),
            ),
            (
                Package {
                    name: "package1",
                    cargo_toml: "./Cargo.toml",
                    workspace_root (file name): "unknown???",
                },
                Io(
                    Dir(
                        ".",
                    ),
                    Cmd(
                        [
                            "cargo",
                            "clippy",
                            "-F",
                            "a,b,c",
                        ],
                    ),
                ),
            ),
            (
                Package {
                    name: "package2",
                    cargo_toml: "./Cargo.toml",
                    workspace_root (file name): "unknown???",
                },
                Io(
                    Dir(
                        ".",
                    ),
                    Cmd(
                        [
                            "cargo",
                            "clippy",
                            "-F",
                            "a,b,c",
                        ],
                    ),
                ),
            ),
        ]
    "#]]
    .assert_debug_eq(&v);
}

#[test]
fn pkg_checker_action_only_fmt_clippy() {
    let yaml = r#"
user/repo:
  all: true # not supported yet
  packages:
    crate1:
      fmt: true
    crate2:
      clippy: RUSTFLAGS="-cfg abc" cargo clippy
    crate3:
      all: true # not supported yet
"#;
    let v = Config::from_yaml(yaml).unwrap()[0]
        .config
        .pkg_checker_action(&[
            Package::test_new("crate1"),
            Package::test_new("crate2"),
            Package::test_new("crate3"),
        ])
        .unwrap();
    expect![[r#"
        [
            (
                Package {
                    name: "crate1",
                    cargo_toml: "./Cargo.toml",
                    workspace_root (file name): "unknown???",
                },
                Cmd(
                    [
                        "cargo",
                        "fmt",
                        "--check",
                        "--manifest-path",
                        "./Cargo.toml",
                    ],
                ),
            ),
            (
                Package {
                    name: "crate2",
                    cargo_toml: "./Cargo.toml",
                    workspace_root (file name): "unknown???",
                },
                Io(
                    Dir(
                        ".",
                    ),
                    Io(
                        Env(
                            "RUSTFLAGS",
                            "-cfg abc",
                        ),
                        Cmd(
                            [
                                "cargo",
                                "clippy",
                            ],
                        ),
                    ),
                ),
            ),
        ]
    "#]]
    .assert_debug_eq(&v);
}

#[test]
fn bad_check() {
    let bad1 = "
user/repo: 
  clippy: cargo miri run
";
    let err = format!("{}", Config::from_yaml(bad1).unwrap_err());
    let expected = expect!["命令 `cargo miri run` 与检查工具 `clippy` 不匹配"];
    expected.assert_eq(&err);

    let bad2 = "
user/repo: 
  packages:
    crate1: 
      clippy: cargo miri run
";
    let err = format!("{}", Config::from_yaml(bad2).unwrap_err());
    // FIXME: 或许可以更好的错误报告，比如在哪个仓库哪个库的命令上不匹配
    let expected = expect!["命令 `cargo miri run` 与检查工具 `clippy` 不匹配"];
    expected.assert_eq(&err);
}

//! os-checker 会启发式搜索一些常见的脚本来猜测额外的目标架构信息，这些被查询脚本为：
//! .github 文件夹内的任何文件、递归查找 Makefile、makefile、py、sh、just 后缀的文件
//!
//! 此外，一个可能的改进是查找 Cargo.toml 中的条件编译中包含架构的信息（见 `layout::parse`）。
//!
//! 除了标准的 Makefile 文件外，还有一些其他文件名可能会与 Makefile 一起使用，这些文件通常用于定义特定目标或条件的规则：
//!
//! 1. **GNUmakefile**: 这是 GNU make 的标准文件名，用于区分其他 make 版本。
//! 2. **makefile**: 这是 BSD make 的标准文件名。
//! 3. **Makefile.am** 或 **Makefile.in**: 这些文件通常由 autotools 生成，用于自动配置 makefile。
//! 4. **Makefile.\***: 有时，项目可能会有多个 Makefile 文件，用于不同的平台或配置，例如 Makefile.linux 或 Makefile.debug。
//! 5. **.mk**: 这是 Makefile 的另一种扩展名，用于包含在其他 Makefile 中的 makefile 片段。
//!
//! 请注意，尽管有多种可能的文件名，但大多数 make 工具默认寻找的文件名是 "Makefile" 或 "makefile"。如果你使用不同的文件名，可能需要在调用 `make` 命令时指定文件名。

use super::targets::Targets;
use crate::{
    cli::is_not_layout,
    utils::{
        empty, install_toolchain, rustup_target_add, rustup_target_add_for_checkers,
        scan_scripts_for_target, walk_dir, PECULIAR_TARGETS,
    },
    Result, XString,
};
use cargo_metadata::{
    camino::{Utf8Path, Utf8PathBuf},
    Metadata,
};
use duct::cmd;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Targets obtained from `.cargo/config{,.toml}` and `Cargo.toml`'s metadata table.
pub struct WorkspaceTargetTriples {
    pub packages: Vec<PackageTargets>,
}

pub struct PackageTargets {
    pub pkg_name: XString,
    pub pkg_dir: Utf8PathBuf,
    // NOTE: this can be empty if no target is specified in .cargo/config.toml,
    // in which case the host target should be implied when the empty value is handled.
    pub targets: Targets,
    pub toolchain: Option<RustToolchain>,
}

impl WorkspaceTargetTriples {
    pub fn new(repo_root: &Utf8Path, ws: &Metadata) -> Self {
        // src: https://docs.rs/crate/riscv/0.8.0/source/Cargo.toml.orig
        // [package.metadata.docs.rs]
        // default-target = "riscv64imac-unknown-none-elf" # 可选的
        // targets = [ # 也是可选的
        //     "riscv32i-unknown-none-elf", "riscv32imc-unknown-none-elf", "riscv32imac-unknown-none-elf",
        //     "riscv64imac-unknown-none-elf", "riscv64gc-unknown-none-elf",
        // ]
        let ws_targets = ws_targets(ws).unwrap_or_default();
        WorkspaceTargetTriples {
            packages: ws
                .workspace_packages()
                .iter()
                .map(|pkg| {
                    let mut targets = Targets::new();
                    let pkg_dir = pkg.manifest_path.parent().unwrap();
                    let toolchain = RustToolchain::search(pkg_dir, repo_root).unwrap();
                    if let Some(t) = toolchain.as_ref().and_then(RustToolchain::targets) {
                        targets.merge(&t);
                    }

                    targets.merge(&search_cargo_config_toml(pkg_dir, repo_root).unwrap());

                    let pkg_targets = pkg_targets(pkg).unwrap_or_default();

                    targets.merge(&pkg_targets);
                    targets.merge(&ws_targets);

                    // filter out peculiar targets
                    targets.remove_peculiar_targets();

                    PackageTargets {
                        pkg_name: XString::from(&*pkg.name),
                        pkg_dir: pkg_dir.to_owned(),
                        targets,
                        toolchain,
                    }
                })
                .collect(),
        }
    }
}

fn ws_targets(ws: &Metadata) -> Option<Targets> {
    let ws_manifest_path = &ws.workspace_root.join("Cargo.toml");
    let mut ws_targets = Targets::default();
    let docsrs = ws.workspace_metadata.get("docs")?.get("rs")?;
    if let Some(value) = docsrs.get("default-target") {
        metadata_targets(value, |target| {
            ws_targets.cargo_toml_docsrs_in_workspace_default(target, ws_manifest_path)
        });
    }
    if let Some(value) = docsrs.get("targets") {
        metadata_targets(value, |target| {
            ws_targets.cargo_toml_docsrs_in_workspace(target, ws_manifest_path)
        });
    }
    Some(ws_targets)
}

fn pkg_targets(pkg: &cargo_metadata::Package) -> Option<Targets> {
    let manifest_path = &pkg.manifest_path;
    let mut targets = Targets::default();
    let docsrs = pkg.metadata.get("docs")?.get("rs")?;
    if let Some(value) = docsrs.get("default-target") {
        metadata_targets(value, |target| {
            targets.cargo_toml_docsrs_in_pkg_default(target, manifest_path)
        });
    }
    if let Some(value) = docsrs.get("targets") {
        metadata_targets(value, |target| {
            targets.cargo_toml_docsrs_in_pkg(target, manifest_path)
        });
    }
    Some(targets)
}

fn metadata_targets(value: &Value, mut f: impl FnMut(&str)) {
    match value {
        Value::String(target) => f(target),
        Value::Array(v) => {
            for target in v.iter().filter_map(Value::as_str) {
                f(target);
            }
        }
        _ => (),
    }
}

pub fn scripts_and_github_dir_in_repo(repo_root: &Utf8Path) -> Result<Targets> {
    let mut targets = Targets::new();
    scripts_in_dir(repo_root, |target, path| {
        targets.detected_by_repo_scripts(target, path);
    })?;

    let github_dir = Utf8Path::new(repo_root).join(".github");
    let github_files = walk_dir(&github_dir, 4, empty(), &[], Some);

    scan_scripts_for_target(&github_files, |target, path| {
        targets.detected_by_repo_github(target, path);
    })?;

    targets.remove_peculiar_targets();
    Ok(targets)
}

fn scripts_in_dir(dir: &Utf8Path, f: impl FnMut(&str, Utf8PathBuf)) -> Result<()> {
    let scripts = walk_dir(dir, 4, [".github"], &[], |file_path| {
        let file_stem = file_path.file_stem()?;

        if file_stem.starts_with("Makefile")
            || file_stem.starts_with("makefile")
            || file_stem == "GNUmakefile"
        {
            return Some(file_path);
        }
        if let "mk" | "sh" | "py" | "just" = file_path.extension()? {
            return Some(file_path);
        }
        None
    });
    scan_scripts_for_target(&scripts, f)
}

pub fn scripts_in_pkg_dir(pkg_dir: &Utf8Path, targets: &mut Targets) -> Result<()> {
    scripts_in_dir(pkg_dir, |target, path| {
        targets.detected_by_pkg_scripts(target, path);
    })
}

// *************** .cargo/config.toml ***************

// [build]
// target = ["x86_64-unknown-linux-gnu", "riscv64gc-unknown-none-elf"]
#[derive(Deserialize, Serialize, Debug)]
pub struct CargoConfigToml {
    build: BuildTarget,
}

impl CargoConfigToml {
    #[cfg(test)]
    pub fn test() -> Self {
        CargoConfigToml {
            build: BuildTarget {
                target: CargoConfigTomlTarget::Multiple(vec!["a".to_owned()]),
            },
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct BuildTarget {
    target: CargoConfigTomlTarget,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum CargoConfigTomlTarget {
    One(String),
    Multiple(Vec<String>),
}

impl CargoConfigTomlTarget {
    fn new(path: &Utf8Path) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        let config: CargoConfigToml = basic_toml::from_slice(&bytes)?;
        Ok(config.build.target)
    }

    pub fn search(child: &Utf8Path, root: &Utf8Path) -> Result<Option<(Self, Utf8PathBuf)>> {
        search_from_child_to_root(
            |path| {
                path.extend([".cargo", "config.toml"]);
                if let Ok(target) = CargoConfigTomlTarget::new(path) {
                    return Some(target);
                }
                path.set_file_name("config");
                if let Ok(target) = CargoConfigTomlTarget::new(path) {
                    return Some(target);
                }
                None
            },
            child,
            root,
        )
    }
}

fn search_from_child_to_root<T>(
    mut f: impl FnMut(&mut Utf8PathBuf) -> Option<T>,
    child: &Utf8Path,
    root: &Utf8Path,
) -> Result<Option<(T, Utf8PathBuf)>> {
    let child = child.canonicalize_utf8()?;
    let root = root.canonicalize_utf8()?;
    let mut path = Utf8PathBuf::new();
    for parent in child.ancestors() {
        path.extend(parent);
        if let Some(ret) = f(&mut path) {
            return Ok(Some((ret, path)));
        }
        if parent == root {
            break;
        }
        path.clear();
    }
    Ok(None)
}

/// 搜索从 package dir 开始往父级到 repo_root 的 .cargo/ 目录下的
/// config 和 config.toml 文件。
///
/// 注意：一旦找到一个更高优先级的配置文件中的 build.target，那么不再进行搜索
/// * 子级优于父级
/// * config.toml 文件优于 config 文件
fn search_cargo_config_toml(pkg_dir: &Utf8Path, repo_root: &Utf8Path) -> Result<Targets> {
    let mut targets = Targets::default();
    match CargoConfigTomlTarget::search(pkg_dir, repo_root)? {
        Some((CargoConfigTomlTarget::One(target), p)) => targets.cargo_config_toml(target, p),
        Some((CargoConfigTomlTarget::Multiple(v), p)) => {
            for target in v {
                targets.cargo_config_toml(target, p.clone());
            }
        }
        None => (),
    }
    Ok(targets)
}

#[derive(Deserialize, Debug)]
pub struct RustToolchainToml {
    pub toolchain: RustToolchain,
}

impl RustToolchainToml {
    fn new(path: &Utf8Path) -> Option<Self> {
        let bytes = std::fs::read(path).ok()?;
        basic_toml::from_slice(&bytes).ok()
    }
}

// [toolchain]
// channel = "nightly-2020-07-10"
// components = [ "rustfmt", "rustc-dev" ]
// targets = [ "wasm32-unknown-unknown", "thumbv2-none-eabi" ]
// profile = "minimal"
#[derive(Deserialize, Serialize, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct RustToolchain {
    pub channel: String,
    pub profile: Option<XString>,
    pub targets: Option<Vec<String>>,
    pub components: Option<Vec<String>>,
    /// 默认为空路径
    #[serde(skip_deserializing)]
    pub toml_path: Utf8PathBuf,
    /// 如果仓库的工具链没写 clippy，那么该字段为 true，表示 os-checker 会安装它
    #[serde(default)]
    pub need_install_clippy: bool,
    /// 特殊的编译目标，比如 avr-unknown-gnu-atmega328、x86_64-fuchsia
    /// 记录见 https://github.com/os-checker/os-checker/issues/77
    #[serde(default)]
    pub peculiar_targets: Option<Vec<String>>,
}

impl RustToolchain {
    /// 将该工具链信息存到全局数组，返回唯一的索引
    pub fn store(self) -> usize {
        crate::output::push_toolchain(self)
    }

    // /// 从数据库的 channel 字符串构建非完整的工具链信息。
    // /// 将来应该记录 RustToolchain，但目前先跳过它。
    // pub fn from_channel_string(channel: &str) -> Self {
    //     let (profile, targets, components, toml_path, need_install_clippy, peculiar_targets) =
    //         Default::default();
    //     RustToolchain {
    //         channel: channel.to_owned(),
    //         profile,
    //         targets,
    //         components,
    //         toml_path,
    //         need_install_clippy,
    //         peculiar_targets,
    //     }
    // }

    pub fn targets(&self) -> Option<Targets> {
        let v = self.targets.as_deref()?;
        let mut targets = Targets::new();
        for target in v {
            targets.rust_toolchain_toml(target, &self.toml_path);
        }
        Some(targets)
    }

    pub fn search(pkg_dir: &Utf8Path, repo_root: &Utf8Path) -> Result<Option<Self>> {
        let Some((toolchain, toml_path)) = search_from_child_to_root(
            |path| {
                path.push("rust-toolchain.toml");
                if let Some(target) = RustToolchainToml::new(path) {
                    return Some(target);
                }
                path.set_file_name("rust-toolchain");
                if let Some(target) = RustToolchainToml::new(path) {
                    return Some(target);
                }
                None
            },
            pkg_dir,
            repo_root,
        )?
        else {
            return Ok(None);
        };
        let mut toolchain = toolchain.toolchain;
        toolchain.toml_path = toml_path;
        if is_not_layout() {
            // 不再解析工具链时安装这些，因为每个 pkg 检查工作空间下的
            // 配置时，会重复安装。
            // toolchain.install_toolchain_and_components()?;
            toolchain.check_peculiar_targets();
        }
        Ok(Some(toolchain))
    }

    pub fn append_targets(&mut self, targets: &[String]) {
        if targets.is_empty() {
            return;
        }
        let targets = targets
            .iter()
            .filter(|t| !PECULIAR_TARGETS.contains(&t.as_str()))
            .map(String::clone);
        match &mut self.targets {
            Some(v) => {
                v.extend(targets);
                v.sort_unstable();
                v.dedup();
            }
            None => self.targets = Some(targets.collect()),
        }
    }

    /// 仅安装 targets。
    ///
    /// NOTE: 由于会添加新的 targets，因此该函数是为此设置的。
    /// 尤其是，主机和检查工具所需的工具链必须提前安装这些 targets。
    pub fn install_targets(&self) -> Result<()> {
        if let Some(targets) = self.targets.as_deref() {
            let targets: Vec<_> = targets.iter().map(|s| s.as_str()).collect();
            let repo_dir = self.toml_path.parent().unwrap();
            rustup_target_add(&targets, repo_dir)?;
            rustup_target_add_for_checkers(&targets)?;
        }
        Ok(())
    }

    /// 检查自定义工具链是否包含必要的组件（比如 clippy），如果未安装，则本地安装它。
    /// 注意：我们已经强制让 fmt 使用主机的 nightly 工具链，因此不检查它。
    pub fn install_toolchain_and_components(&mut self) -> Result<()> {
        let has_clippy = self
            .components
            .as_deref()
            .map(|v| v.iter().any(|c| c.contains("clippy")))
            .unwrap_or(false);
        if !has_clippy {
            self.need_install_clippy = true;

            let repo_dir = self.toml_path.parent().unwrap();
            let stdout = install_toolchain(repo_dir)?;
            println!(
                "{}\ntargets = {:#?}",
                String::from_utf8(stdout)?,
                self.targets
            );

            let output = cmd!(
                "rustup",
                "component",
                "add",
                "clippy",
                "--toolchain",
                &self.channel
            )
            .run()?;

            ensure!(
                output.status.success(),
                "RustToolchain = {self:#?}\n无法给仓库设置的工具链安装 clippy：\nstderr={}",
                String::from_utf8_lossy(&output.stderr)
            );

            info!("仓库设置的工具链不含 clippy，os-checker 自动安装它；RustToolchain = {self:#?}");

            match self.components.as_mut() {
                Some(v) => v.push("clippy".to_owned()),
                None => self.components = Some(vec!["clippy".to_owned()]),
            }
        }
        Ok(())
    }

    /// 将特殊的编译目标从 targets 移到 peculiar_targets。
    /// 该函数还对 targets 进行字母表排序，保证这两个列表内的顺序。
    fn check_peculiar_targets(&mut self) {
        if let Some(v) = &mut self.targets {
            v.sort_unstable();
            v.dedup();
            let mut idx = 0;
            loop {
                if idx == v.len() {
                    break;
                }
                let target = &*v[idx];
                if PECULIAR_TARGETS.contains(&target) {
                    info!(
                        target,
                        "检查到不寻常的编译目标；os-checker 暂时不在该目标上运行检查"
                    );
                    let target = v.remove(idx);
                    if let Some(p) = &mut self.peculiar_targets {
                        p.push(target);
                    } else {
                        self.peculiar_targets = Some(vec![target]);
                    }
                } else {
                    idx += 1;
                }
            }
        }
    }

    /// 虽然主机工具链很可能安装了 rustfmt，但检查一遍也是好的。
    /// 此函数用于主机工具链检查，而不是仓库工具链。
    pub fn install_rustfmt(&self) -> Result<()> {
        let output = cmd!("rustup", "component", "add", "rustfmt").run()?;
        ensure!(
            output.status.success(),
            "RustToolchain = {self:#?}\n无法给仓库设置的工具链安装 clippy：\nstderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }
}

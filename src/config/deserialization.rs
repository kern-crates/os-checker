use super::checker::CheckerTool;
use crate::{
    config::{checker::TOOLS, Resolve},
    layout::{Packages, Pkg},
    Result,
};
use cargo_metadata::camino::Utf8Path;
use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[cfg(test)]
mod tests;

mod config_options;
use config_options::{Cmds, Setup, Targets};

#[derive(Debug, Serialize, Deserialize, Default, JsonSchema)]
pub struct RepoConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup: Option<Setup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub targets: Option<Targets>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Cmds::is_empty")]
    pub cmds: Cmds,
    #[serde(default)]
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub packages: IndexMap<String, RepoConfig>,
}

impl RepoConfig {
    /// 每个 package 及其对应的检查命令
    pub fn resolve(&self, repo: &str, pkgs: &Packages) -> Result<Vec<Resolve>> {
        // validate pkg names in packages
        self.validate_pkgs(repo, pkgs)?;

        let mut cmds = Cmds::new_with_all_checkers_enabled();

        // validate checkers in cmds
        self.validate_checker(repo, &cmds)?;

        let mut v = Vec::<Resolve>::with_capacity(pkgs.len() * TOOLS);

        let targets_for_all_pkgs = self.targets.as_ref().map(|val| val.as_slice());
        for (pkg_name, info) in &**pkgs {
            // set cmds from repo
            cmds.merge(&self.cmds);

            // pick targets configurated from pkg or repo
            let targets = if let Some(pkg_config) = self.packages.get(pkg_name.as_str()) {
                // set cmds from pkg
                cmds.merge(&pkg_config.cmds);

                let targets_for_pkg = pkg_config.targets.as_ref().map(|val| val.as_slice());
                targets_for_pkg.or(targets_for_all_pkgs)
            } else {
                targets_for_all_pkgs
            };

            // if targets is empty, pick candidates detected from repo
            let pkgs = info.pkgs(pkg_name, targets);

            resolve_for_single_pkg(&cmds, &pkgs, &mut v)?;

            // default to enable all checkers for next package
            cmds.enable_all_checkers();
        }

        v.sort_unstable_by(|a, b| (&a.pkg_name, a.checker).cmp(&(&b.pkg_name, b.checker)));
        Ok(v)
    }

    fn validate_pkgs(&self, repo: &str, pkgs: &Packages) -> Result<()> {
        for pkg_name in self.packages.keys() {
            ensure!(
                pkgs.contains_key(&**pkg_name),
                "The package `{pkg_name}` is not in the repo `{repo}`."
            );
        }
        Ok(())
    }

    /// cmds is from new_with_all_checkers_enabled
    ///
    /// 这个其实可以做到解析 JSON 那个步骤，但为了更好的错误报告，在这附加 repo 或者 pkg 信息
    fn validate_checker(&self, repo: &str, cmds: &Cmds) -> Result<()> {
        // validate repo's checkers in cmds
        for cmd in self.cmds.keys() {
            ensure!(
                cmds.contains_key(cmd),
                "Checker `{}` is not supported in cmds of repo `{repo}`",
                cmd.name()
            );
        }
        // validate pkg's checkers in cmds
        for (pkg_name, pkg_config) in &self.packages {
            pkg_config.validate_checker_in_pkg(repo, pkg_name, cmds)?;
        }
        Ok(())
    }

    // self is a pkg config
    fn validate_checker_in_pkg(&self, repo: &str, pkg: &str, cmds: &Cmds) -> Result<()> {
        for cmd in self.cmds.keys() {
            ensure!(
                cmds.contains_key(cmd),
                "Checker `{}` is not supported in cmds of repo `{repo}`'s pkg `{pkg}`",
                cmd.name()
            );
        }
        Ok(())
    }

    /// 检查自定义命令是否与 checker 匹配
    pub fn validate_checker_name(&self, repo: &str) -> Result<()> {
        for (checker, cmd) in &*self.cmds {
            let name = checker.name();
            // NOTE: 如果采用 make 脚本运行检查，则可以写 `make clippy`。
            if let Err(failed_cmd) = cmd.validate_checker_name(name) {
                bail!("For repo `{repo}`, `{failed_cmd}` doesn't contain the corresponding checker name `{name}`");
            }
        }
        // valide pkg's checkers in cmds
        for (pkg_name, pkg_config) in &self.packages {
            pkg_config.validate_checker_name_in_pkg(repo, pkg_name)?;
        }
        Ok(())
    }

    // self is a pkg config
    pub fn validate_checker_name_in_pkg(&self, repo: &str, pkg: &str) -> Result<()> {
        for (checker, cmd) in &*self.cmds {
            let name = checker.name();
            if let Err(failed_cmd) = cmd.validate_checker_name(name) {
                bail!(
                    "For pkg `{pkg}` in repo `{repo}`, `{failed_cmd}` \
                     doesn't contain the corresponding checker name `{name}`"
                );
            }
        }
        Ok(())
    }

    // TODO: setup environment for repo
    // pub fn setup(&self) {}
}

/// TODO: 其他工具待完成
fn resolve_for_single_pkg(cmds: &Cmds, pkgs: &[Pkg], v: &mut Vec<Resolve>) -> Result<()> {
    use either::{Left, Right};
    use CheckerTool::*;

    // apply cmds
    for (checker, cmd) in &**cmds {
        match (*checker, cmd.cmd()) {
            (Fmt, Left(true)) => Resolve::fmt(pkgs, v),
            (Clippy, Left(true)) => Resolve::clippy(pkgs, v),
            (Lockbud, Left(true)) => Resolve::lockbud(pkgs, v),
            (c, Right(s)) => Resolve::custom(pkgs, s, c, v)?,
            _ => (),
        }
    }

    Ok(())
}

/// Generate JSON schema
pub fn gen_schema(path: &Utf8Path) -> Result<()> {
    use schemars::generate::SchemaSettings;
    use std::io::Write;

    let settings = SchemaSettings::draft07().with(|s| {
        s.option_nullable = true;
        s.option_add_null_type = false;
    });
    let generator = settings.into_generator();
    let schema = generator.into_root_schema_for::<IndexMap<String, RepoConfig>>();
    let json = serde_json::to_string_pretty(&schema)?;
    std::fs::File::create(path)?.write_all(json.as_bytes())?;
    Ok(())
}

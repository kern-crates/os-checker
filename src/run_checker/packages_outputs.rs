use super::{utils::DbRepo, Output, Resolve};
use crate::{config::TOOLS, db::CacheValue};
use color_eyre::owo_colors::OwoColorize;
use indexmap::IndexMap;
use regex::Regex;
use std::sync::LazyLock;
use time::OffsetDateTime;

pub type PackageName = String;

#[derive(Debug)]
pub struct Outputs {
    /// 对于 Cargo 检查类型会导致多个 Output，因为每个输出与 cmd 相关；
    /// 对于其他检查类型，只有一个 Output。
    inner: Vec<CacheValue>,
}

impl Outputs {
    fn new() -> Self {
        Outputs {
            inner: Vec::with_capacity(TOOLS),
        }
    }

    pub fn count(&self) -> usize {
        self.inner.iter().map(|out| out.count()).sum()
    }

    pub fn as_slice(&self) -> &[CacheValue] {
        &self.inner
    }

    pub fn push(&mut self, cache: CacheValue) {
        self.inner.push(cache);
    }
}

#[derive(Debug)]
pub struct PackagesOutputs {
    // key 为 pkg_name, value 为 outputs
    map: IndexMap<PackageName, Outputs>,
}

impl From<Vec<(&str, CacheValue)>> for PackagesOutputs {
    fn from(v: Vec<(&str, CacheValue)>) -> Self {
        let mut map = IndexMap::<PackageName, Outputs>::with_capacity(v.len());
        for (pkg_name, cache) in v {
            if let Some(outputs) = map.get_mut(pkg_name) {
                outputs.push(cache);
            } else {
                map.insert(pkg_name.to_owned(), Outputs { inner: vec![cache] });
            }
        }
        let mut outputs = PackagesOutputs { map };
        outputs.sort_by_name_and_checkers();
        outputs
    }
}

impl PackagesOutputs {
    pub fn new() -> Self {
        PackagesOutputs {
            map: IndexMap::with_capacity(4),
        }
    }

    pub fn count(&self) -> usize {
        // 这里的计数应该包括 CheckerTool::Cargo
        self.values().map(Outputs::count).sum()
    }

    /// This should be called after all outputs of all packages finish.
    pub fn sort_by_name_and_checkers(&mut self) {
        self.sort_unstable_keys();
        for outputs in self.values_mut() {
            outputs.inner.sort_unstable_by_key(|o| o.checker());
        }
    }

    /// 获取缓存的检查结果。
    /// `true` 表示成功获取；`false` 表示无缓存。
    pub fn fetch_cache(&mut self, resolve: &Resolve, db_repo: Option<DbRepo>) -> bool {
        let Some(db_repo) = db_repo else { return false };

        let _span = error_span!("fetch_cache", %resolve.pkg_name, resolve.cmd).entered();

        let key = &db_repo.key(resolve);
        match db_repo.read_cache(key) {
            Ok(Some(cache)) => {
                let pkg_name = resolve.pkg_name.as_str();
                if let Some(v) = self.get_mut(pkg_name) {
                    v.push(cache);
                } else {
                    let pkg_name = pkg_name.to_owned();
                    let outputs = Outputs { inner: vec![cache] };
                    self.insert(pkg_name, outputs);
                }
                db_repo.set_info_cache(key);

                let resolve_cargo = resolve.new_cargo();
                let key_cargo = &db_repo.key(&resolve_cargo);
                match db_repo.read_cache(key_cargo) {
                    Ok(Some(cache_cargo)) => {
                        self.get_mut(pkg_name).unwrap().push(cache_cargo);
                        info!("成功获取缓存（含 Cargo）");
                        db_repo.set_info_cache(key_cargo);
                    }
                    Ok(None) => info!("成功获取缓存"),
                    Err(err) => error!(?err, "无法获取 Cargo 检查结果缓存"),
                };

                return true;
            }
            Ok(None) => warn!("无缓存"),
            Err(err) => error!(?err, "获取缓存失败"),
        }

        false
    }

    pub fn push_output_with_cargo(
        &mut self,
        output: Output,
        db_repo: Option<DbRepo>,
        now_utc: OffsetDateTime,
    ) {
        let pkg_name = output.resolve.pkg_name.as_str();
        if let Some(v) = self.get_mut(pkg_name) {
            if let Some(stderr_parsed) = cargo_stderr_stripped(&output, now_utc) {
                let output = output
                    .new_cargo_from_checker(stderr_parsed)
                    .to_cache(db_repo);
                v.push(output);
            }

            v.push(output.to_cache(db_repo));
        } else {
            let pkg_name = pkg_name.to_owned();
            let mut outputs = Outputs::new();

            if let Some(stderr_parsed) = cargo_stderr_stripped(&output, now_utc) {
                outputs.push(
                    output
                        .new_cargo_from_checker(stderr_parsed)
                        .to_cache(db_repo),
                );
            }

            outputs.push(output.to_cache(db_repo));
            self.insert(pkg_name, outputs);
        }
    }

    pub fn push_cargo_layout_parse_error(
        &mut self,
        key: String,
        output: Output,
        db_repo: Option<DbRepo>,
    ) {
        self.map.insert(
            key,
            Outputs {
                inner: vec![output.to_cache(db_repo)],
            },
        );
    }
}

/// Some means there is a cargo erroneous output to be created or updated.
fn cargo_stderr_stripped(output: &Output, now_utc: OffsetDateTime) -> Option<String> {
    let resolve = &output.resolve;
    let raw_stderr = output.raw.stderr.as_slice();
    let stderr = String::from_utf8_lossy(raw_stderr);

    debug!(
        %resolve.pkg_name, %resolve.pkg_dir,
        success = %(if output.raw.status.success() {
            "true".bright_green().to_string()
        } else {
            "false".bright_red().to_string()
        }),
        resolve.cmd = %resolve.cmd.bright_black().italic(),
        stderr=%(stderr.on_bright_black())
    );

    let stderr_stripped = strip_ansi_escapes::strip(raw_stderr);
    let stderr = String::from_utf8_lossy(&stderr_stripped);
    // stderr 包含额外的 error: 信息，那么将所有 stderr 内容 作为 cargo 的检查结果
    RE.is_match(&stderr)
        .then(|| extra_header(&stderr, resolve, now_utc))
}

// 在原始的 Cargo 输出的顶部增加必要的信息，方便浏览
fn extra_header(stderr: &str, resolve: &Resolve, now_utc: OffsetDateTime) -> String {
    let Resolve {
        pkg_name,
        pkg_dir,
        target,
        checker,
        cmd,
        ..
    } = resolve;
    let toolchain = resolve.toolchain();
    let now = now_utc.to_offset(time::macros::offset!(+8));
    let features = resolve.features_args.join(" ");
    format!(
        "// pkg_name={pkg_name}, checker={checker:?}\n\
         // toolchain={toolchain}, target={target}\n\
         // features={features}\n\
         // pkg_dir={pkg_dir}\n\
         // cmd={cmd}\n\
         // timestamp={now}\n\
         {stderr}"
    )
}

static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new("\nerror: ").unwrap());

impl std::ops::Deref for PackagesOutputs {
    type Target = IndexMap<PackageName, Outputs>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl std::ops::DerefMut for PackagesOutputs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

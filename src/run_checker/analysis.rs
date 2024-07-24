use super::*;
use ahash::{HashMap, HashMapExt};

pub struct Statistics {
    pkg: XString,
    /// 检查工具报告的不通过的数量（基于文件）
    counts: Count,
    /// 总计
    total: Total,
}

impl Statistics {
    pub fn new(outputs: &[Output]) -> Vec<Statistics> {
        outputs
            .iter()
            .chunk_by(|out| out.package_name.clone())
            .into_iter()
            .map(|(pkg, outputs)| {
                // iterate over outputs from each checker
                let mut counts = Count::default();
                let mut total = Total::default();
                for out in outputs {
                    total.duration_ms += out.duration_ms;

                    // FIXME: 由于路径的唯一性在这变得重要，需要提前归一化路径；两条思路：
                    // * package_name 暗含了库的根目录，因此需要把路径的根目录去掉
                    // * 如果能保证都是绝对路径，那么不需要处理路径
                    match &out.parsed {
                        OutputParsed::Fmt(v) => {
                            for file in v.iter() {
                                counts.push_unformatted(&file.name, file);
                            }
                        }
                        // FIXME: rustc 的文件路径包含了行列信息，记得去除
                        OutputParsed::Clippy(v) => counts.push_clippy(v),
                    }
                }
                Statistics { pkg, counts, total }
            })
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct Total {
    duration_ms: u64,
    counts_on_kind: Vec<(Kind, u32)>,
    counts_on_file: Vec<(Utf8PathBuf, u32)>,
}

#[derive(Debug, Default)]
pub struct Count {
    inner: HashMap<CountKey, u32>,
}

impl Count {
    fn push_unformatted(&mut self, file: &Utf8PathBuf, mes: &FmtMessage) {
        let count = mes
            .mismatches
            .iter()
            .map(|ele| ele.original_end_line + 1 - ele.original_begin_line)
            .sum::<u32>();
        let key_line = CountKey::unformatted_line(file);
        *self.inner.entry(key_line).or_insert(count) += count;

        let key_file = CountKey::unformatted_file(file);
        *self.inner.entry(key_file).or_insert(1) += 1;
    }

    fn push_clippy(&mut self, messages: &[Message]) {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct CountKey {
    file: Utf8PathBuf,
    kind: Kind,
}

impl CountKey {
    /// 表明一个文件中未格式化的地点数量
    fn unformatted_file(file: &Utf8PathBuf) -> Self {
        Self {
            file: file.clone(),
            kind: Kind::Unformatted(Unformatted::File),
        }
    }

    /// 表明一个文件中未格式化的总行数
    fn unformatted_line(file: &Utf8PathBuf) -> Self {
        Self {
            file: file.clone(),
            kind: Kind::Unformatted(Unformatted::Line),
        }
    }
}

/// The kind a checker reports.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Kind {
    /// fmt
    Unformatted(Unformatted),
    /// clippy
    Clippy(Rustc),
    /// miri
    UndefinedBehavior(Rustc),
    /// semver-checks
    SemverViolation,
    /// TODO
    Lockbud,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Unformatted {
    File,
    Line,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Rustc {
    Warn,
    Error,
}
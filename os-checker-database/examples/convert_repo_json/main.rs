//! This program rewrites ui/repos/user/repo/*.json (except basic.json) in these ways:
//! * parse lockbud outputs via Regex to know which file has what diagnostics
//! * same for AtomVChecker, but need slightly adjusting parsing
//! * same for RAPx, but need another parsing (haven't done yet)

use eyre::Result;
use indexmap::IndexSet;
use os_checker_types::{
    out_json::file_tree::{FileTreeRepo, RawReport},
    Kind,
};
use regex::Regex;
use std::{fs::File, sync::LazyLock};

#[macro_use]
extern crate eyre;

mod atomvchecker;
mod cli;
mod lockbud;

fn main() -> Result<()> {
    let cli: cli::Cli = argh::from_env();
    dbg!(&cli);
    let (paths, emit) = cli.json_files()?;

    for path in paths {
        println!("Handle {path:?}");
        let file = File::open(&path)?;
        let mut json: FileTreeRepo = serde_json::from_reader(file)?;

        for data in &mut json.data {
            let mut set = IndexSet::new();
            for report in &data.raw_reports {
                for (&kind, diagnoses) in &report.kinds {
                    match kind {
                        Kind::LockbudPossibly | Kind::LockbudProbably => {
                            rewrite_lockbud(kind, report, diagnoses, &mut set);
                        }
                        Kind::Atomvchecker => {
                            rewrite_atomvchecker(kind, report, diagnoses, &mut set)
                        }
                        _ => (),
                    }
                }
            }
            for diagosis in set {
                diagosis.update_raw_reports(&mut data.raw_reports);
            }
        }

        json.recount_and_sort();

        emit.emit(&path, json)?;
    }

    Ok(())
}

struct Re {
    lockbud: Regex,
    span: Regex,
}

impl Re {
    fn parse_file_paths(&self, hay: &str) -> Vec<String> {
        self.span
            .captures_iter(hay)
            .map(|cap| cap.get(1).unwrap().as_str().to_owned())
            .collect()
    }
}

static RE: LazyLock<Re> = LazyLock::new(|| Re {
    lockbud: Regex::new(r"(?s) \[\n.*?\n    \]\n?").unwrap(),
    span: Regex::new(r"(\S+\.rs):\d+:\d+: \d+:\d+").unwrap(),
});

fn rewrite_lockbud(
    kind: Kind,
    report: &RawReport,
    diagnoses: &[String],
    set: &mut IndexSet<Diagnosis>,
) {
    println!("  Lockbud has {} disanosis.", diagnoses.len());
    for diagnosis in diagnoses {
        for cap in RE.lockbud.captures_iter(diagnosis) {
            let matched = cap.get(0).unwrap().as_str();
            let v_map: Vec<indexmap::IndexMap<lockbud::BugKind, lockbud::Lockbud>> =
                serde_json::from_str(matched).unwrap_or_else(|err| {
                    // FIXME: https://github.com/os-checker/os-checker/issues/362
                    eprintln!("Unable to parse data:\nerr={err:?}\nmatched=\n{matched}");
                    Vec::new()
                });
            for map in &v_map {
                for (bug_kind, val) in map {
                    let file_paths = val.file_paths();
                    println!("{bug_kind:?}: {file_paths:?}");
                    for file in file_paths {
                        // dedup by diag: never emit two identical diags for the same file, kind, feature
                        set.insert(Diagnosis {
                            features: report.features.clone(),
                            kind,
                            file,
                            diag: serde_json::to_string_pretty(&val).unwrap(),
                        });
                    }
                }
            }
        }
    }
}

fn rewrite_atomvchecker(
    kind: Kind,
    report: &RawReport,
    diagnoses: &[String],
    set: &mut IndexSet<Diagnosis>,
) {
    println!("  AtomVChecker has {} disanosis.", diagnoses.len());
    for diagnosis in diagnoses {
        // share the same regex with lockbud
        for cap in RE.lockbud.captures_iter(diagnosis) {
            let matched = cap.get(0).unwrap().as_str();
            let v_out: Vec<atomvchecker::Report> =
                serde_json::from_str(matched).unwrap_or_else(|err| {
                    eprintln!("Unable to parse data:\nerr={err:?}\nmatched=\n{matched}");
                    Vec::new()
                });
            for out in &v_out {
                let bug_kind = out.kind_str();
                let file = out.file_path();
                println!("{bug_kind:?}: {file:?}");
                // dedup by diag: never emit two identical diags for the same file, kind, feature
                set.insert(Diagnosis {
                    features: report.features.clone(),
                    kind,
                    file,
                    diag: out.diag(),
                });
            }
        }
    }
}

/// A temporary datastructure similar to RawReport.
#[derive(Debug, PartialEq, Eq, Hash)]
struct Diagnosis {
    features: String,
    kind: Kind,
    file: String,
    diag: String,
}

impl Diagnosis {
    fn update_raw_reports(self, v: &mut Vec<RawReport>) {
        for report in &mut *v {
            if report.file == self.file && report.features == self.features {
                // the file exists in lockbud kind, then append the diag
                if let Some(kind) = report.kinds.get_mut(&self.kind) {
                    kind.push(self.diag);
                    report.count += 1;
                    return;
                }
            }
        }
        // create a diag
        v.push(RawReport {
            file: self.file.into(),
            features: self.features,
            count: 1,
            kinds: indexmap::indexmap! {
                self.kind => vec![self.diag]
            },
        });
    }
}

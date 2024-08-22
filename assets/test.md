### 主页树状表示例

虽然与现在使用的 JSON 格式不完全一致，但已经非常接近了

```jq
def extract_kind_count: . as $x | .data | map({key: {cmd_idx, kind}}) | group_by(.key) # 按诊断类型分组
| map({
    key: .[0].key,
    count: . | length,
  } | . + { cmd: $x.cmd[.key.cmd_idx] }
    | . + { pkg: $x.env.packages[.cmd.package_idx] }
    | {
      key: {
        user: .pkg.repo.user,
        repo: .pkg.repo.repo,
        package: .pkg.name,
      },
      kind: .key.kind,
      count,
    }
);

def group_by_package: . | group_by(.key) | map({
  key1: .[0].key,
  total_count: map(.count) | add,
  kinds: map({kind, count})
} # + (map({kind, count} | {(.kind): .count}) | add) 
  | . + { key2: { user: .key1.user, repo: .key1.repo } }
);

def group_by_repo: . | group_by(.key2) | map({
  children: map(del(.key2)),
  total_count: map(.total_count) | add
} + .[0].key2 # 从每个 children 数组元素中删除 key2，并把它展开到 children 数组外
  | . + {
    # 聚合所有 children （也就是 packages）的 kind 及其计数：
    # .children | map(.kinds) => 筛选 kinds，得到二维数组，最外层表示每个 package，最内层表示每个 package 的诊断
    # . | add => 把二维数组合并到一维，即所有诊断
    # . | group_by*(.kind) => 按照 .kind 聚合
    # (1) .map(...) => 对每个 kind 得到的数组进行操作，每个数组具有相同的 kind
    # (2) (.[0].kind) => 选取数组第一个元素的 kind 作为键（聚合数组的键总是相同的，并且至少由一个元素，因此 .[0].kind 总是有效的）
    # (3) map(.count) | add => 选取所有 count 并计总（在聚合数组中，已经保证了相同的 kind）
    kinds: .children | map(.kinds) | add | group_by(.kind) | map({kind: .[0].kind, count: map(.count) | add})
  }
);

# 所有计数按照降序排列；先按照总计数，如果相同，按照指定的字段的值来比较先后顺序
def sort_by_count: . | sort_by(
  -.total_count,
  -.sorting["Clippy(Error)"],
  -.sorting["Clippy(Warn)"],
  -.sorting["Unformatted"]
) | map(del(.sorting)); # 最后删除排序键

# 由于 sort_by 不允许对 null 值排序，所以给默认值；
# 必须放置在已有值之前，并通过 + 连接，因为 + 会让右边的键覆盖左边的键
def zero: {
  "Clippy(Error)": 0,
  "Clippy(Warn)": 0,
  Unformatted: 0,
};

# 由于 sort_by 只能指定字段排序，因此从数组转换到对象
def gen_sorting_keys: . | map(zero + {(.kind): .count}) | add;

# 重新排列字段，以及按照计数排序
def epilogue: . | map({
  user, repo, total_count, kinds, sorting: .kinds | gen_sorting_keys,
  children: .children | map({
    user: .key1.user,
    repo: .key1.repo,
    package: .key1.package,
    total_count,
    kinds,
    sorting: .kinds | gen_sorting_keys
  }) | sort_by_count
}) | sort_by_count;

. | extract_kind_count | group_by_package | group_by_repo | epilogue
```

<details>

<summary>jq 执行的 JSON 结果</summary>

```json
[
  {
    "user": "repos",
    "repo": "arceos",
    "total_count": 83,
    "kinds": [
      {
        "kind": "Clippy(Error)",
        "count": 26
      },
      {
        "kind": "Clippy(Warn)",
        "count": 22
      },
      {
        "kind": "Unformatted",
        "count": 35
      }
    ],
    "children": [
      {
        "user": "repos",
        "repo": "arceos",
        "package": "deptool",
        "total_count": 45,
        "kinds": [
          {
            "kind": "Unformatted",
            "count": 35
          },
          {
            "kind": "Clippy(Warn)",
            "count": 10
          }
        ]
      },
      {
        "user": "repos",
        "repo": "arceos",
        "package": "mingo",
        "total_count": 8,
        "kinds": [
          {
            "kind": "Clippy(Error)",
            "count": 7
          },
          {
            "kind": "Clippy(Warn)",
            "count": 1
          }
        ]
      },
      {
        "user": "repos",
        "repo": "arceos",
        "package": "scheduler",
        "total_count": 5,
        "kinds": [
          {
            "kind": "Clippy(Warn)",
            "count": 5
          }
        ]
      },
      {
        "user": "repos",
        "repo": "arceos",
        "package": "axdisplay",
        "total_count": 4,
        "kinds": [
          {
            "kind": "Clippy(Error)",
            "count": 4
          }
        ]
      },
      {
        "user": "repos",
        "repo": "arceos",
        "package": "axfs",
        "total_count": 4,
        "kinds": [
          {
            "kind": "Clippy(Error)",
            "count": 4
          }
        ]
      },
      {
        "user": "repos",
        "repo": "arceos",
        "package": "axnet",
        "total_count": 4,
        "kinds": [
          {
            "kind": "Clippy(Error)",
            "count": 4
          }
        ]
      },
      {
        "user": "repos",
        "repo": "arceos",
        "package": "arceos-bwbench",
        "total_count": 2,
        "kinds": [
          {
            "kind": "Clippy(Error)",
            "count": 2
          }
        ]
      },
      {
        "user": "repos",
        "repo": "arceos",
        "package": "axdriver",
        "total_count": 2,
        "kinds": [
          {
            "kind": "Clippy(Error)",
            "count": 2
          }
        ]
      },
      {
        "user": "repos",
        "repo": "arceos",
        "package": "allocator",
        "total_count": 2,
        "kinds": [
          {
            "kind": "Clippy(Warn)",
            "count": 2
          }
        ]
      },
      {
        "user": "repos",
        "repo": "arceos",
        "package": "linked_list",
        "total_count": 2,
        "kinds": [
          {
            "kind": "Clippy(Warn)",
            "count": 2
          }
        ]
      },
      {
        "user": "repos",
        "repo": "arceos",
        "package": "arceos-display",
        "total_count": 1,
        "kinds": [
          {
            "kind": "Clippy(Error)",
            "count": 1
          }
        ]
      },
      {
        "user": "repos",
        "repo": "arceos",
        "package": "axlibc",
        "total_count": 1,
        "kinds": [
          {
            "kind": "Clippy(Error)",
            "count": 1
          }
        ]
      },
      {
        "user": "repos",
        "repo": "arceos",
        "package": "bwbench-client",
        "total_count": 1,
        "kinds": [
          {
            "kind": "Clippy(Error)",
            "count": 1
          }
        ]
      },
      {
        "user": "repos",
        "repo": "arceos",
        "package": "arceos-priority",
        "total_count": 1,
        "kinds": [
          {
            "kind": "Clippy(Warn)",
            "count": 1
          }
        ]
      },
      {
        "user": "repos",
        "repo": "arceos",
        "package": "slab_allocator",
        "total_count": 1,
        "kinds": [
          {
            "kind": "Clippy(Warn)",
            "count": 1
          }
        ]
      }
    ]
  },
  {
    "user": "repos",
    "repo": "os-checker-test-suite",
    "total_count": 6,
    "kinds": [
      {
        "kind": "Clippy(Error)",
        "count": 1
      },
      {
        "kind": "Clippy(Warn)",
        "count": 1
      },
      {
        "kind": "Unformatted",
        "count": 4
      }
    ],
    "children": [
      {
        "user": "repos",
        "repo": "os-checker-test-suite",
        "package": "os-checker-test-suite",
        "total_count": 6,
        "kinds": [
          {
            "kind": "Unformatted",
            "count": 4
          },
          {
            "kind": "Clippy(Error)",
            "count": 1
          },
          {
            "kind": "Clippy(Warn)",
            "count": 1
          }
        ]
      }
    ]
  }
]
```

</details>
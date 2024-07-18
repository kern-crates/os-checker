use super::LayoutOwner;
use expect_test::expect;

#[test]
fn arceos_layout() {
    crate::logger_init();
    let excluded = &["tmp"];
    assert!(LayoutOwner::new("tmp", excluded).is_err());

    let arceos = LayoutOwner::new("./repos/arceos", excluded).unwrap();
    expect![[r#"
        Layout {
            repo_root: "./repos/arceos",
            cargo_tomls: [
                "./repos/arceos/Cargo.toml",
                "./repos/arceos/api/arceos_api/Cargo.toml",
                "./repos/arceos/api/arceos_posix_api/Cargo.toml",
                "./repos/arceos/api/axfeat/Cargo.toml",
                "./repos/arceos/apps/display/Cargo.toml",
                "./repos/arceos/apps/exception/Cargo.toml",
                "./repos/arceos/apps/fs/shell/Cargo.toml",
                "./repos/arceos/apps/helloworld/Cargo.toml",
                "./repos/arceos/apps/memtest/Cargo.toml",
                "./repos/arceos/apps/net/bwbench/Cargo.toml",
                "./repos/arceos/apps/net/echoserver/Cargo.toml",
                "./repos/arceos/apps/net/httpclient/Cargo.toml",
                "./repos/arceos/apps/net/httpserver/Cargo.toml",
                "./repos/arceos/apps/net/udpserver/Cargo.toml",
                "./repos/arceos/apps/task/parallel/Cargo.toml",
                "./repos/arceos/apps/task/priority/Cargo.toml",
                "./repos/arceos/apps/task/sleep/Cargo.toml",
                "./repos/arceos/apps/task/tls/Cargo.toml",
                "./repos/arceos/apps/task/yield/Cargo.toml",
                "./repos/arceos/crates/allocator/Cargo.toml",
                "./repos/arceos/crates/arm_gic/Cargo.toml",
                "./repos/arceos/crates/arm_pl011/Cargo.toml",
                "./repos/arceos/crates/axerrno/Cargo.toml",
                "./repos/arceos/crates/axfs_devfs/Cargo.toml",
                "./repos/arceos/crates/axfs_ramfs/Cargo.toml",
                "./repos/arceos/crates/axfs_vfs/Cargo.toml",
                "./repos/arceos/crates/axio/Cargo.toml",
                "./repos/arceos/crates/capability/Cargo.toml",
                "./repos/arceos/crates/crate_interface/Cargo.toml",
                "./repos/arceos/crates/driver_block/Cargo.toml",
                "./repos/arceos/crates/driver_common/Cargo.toml",
                "./repos/arceos/crates/driver_display/Cargo.toml",
                "./repos/arceos/crates/driver_net/Cargo.toml",
                "./repos/arceos/crates/driver_pci/Cargo.toml",
                "./repos/arceos/crates/driver_virtio/Cargo.toml",
                "./repos/arceos/crates/dw_apb_uart/Cargo.toml",
                "./repos/arceos/crates/flatten_objects/Cargo.toml",
                "./repos/arceos/crates/handler_table/Cargo.toml",
                "./repos/arceos/crates/kernel_guard/Cargo.toml",
                "./repos/arceos/crates/lazy_init/Cargo.toml",
                "./repos/arceos/crates/linked_list/Cargo.toml",
                "./repos/arceos/crates/memory_addr/Cargo.toml",
                "./repos/arceos/crates/page_table/Cargo.toml",
                "./repos/arceos/crates/page_table_entry/Cargo.toml",
                "./repos/arceos/crates/percpu/Cargo.toml",
                "./repos/arceos/crates/percpu_macros/Cargo.toml",
                "./repos/arceos/crates/ratio/Cargo.toml",
                "./repos/arceos/crates/scheduler/Cargo.toml",
                "./repos/arceos/crates/slab_allocator/Cargo.toml",
                "./repos/arceos/crates/spinlock/Cargo.toml",
                "./repos/arceos/crates/timer_list/Cargo.toml",
                "./repos/arceos/crates/tuple_for_each/Cargo.toml",
                "./repos/arceos/modules/axalloc/Cargo.toml",
                "./repos/arceos/modules/axconfig/Cargo.toml",
                "./repos/arceos/modules/axdisplay/Cargo.toml",
                "./repos/arceos/modules/axdriver/Cargo.toml",
                "./repos/arceos/modules/axfs/Cargo.toml",
                "./repos/arceos/modules/axhal/Cargo.toml",
                "./repos/arceos/modules/axlog/Cargo.toml",
                "./repos/arceos/modules/axnet/Cargo.toml",
                "./repos/arceos/modules/axruntime/Cargo.toml",
                "./repos/arceos/modules/axsync/Cargo.toml",
                "./repos/arceos/modules/axtask/Cargo.toml",
                "./repos/arceos/tools/bwbench_client/Cargo.toml",
                "./repos/arceos/tools/deptool/Cargo.toml",
                "./repos/arceos/tools/raspi4/chainloader/Cargo.toml",
                "./repos/arceos/ulib/axlibc/Cargo.toml",
                "./repos/arceos/ulib/axstd/Cargo.toml",
            ],
            workspaces: Workspaces {
                [0] root: "./",
                [0] root.members: [
                    "allocator",
                    "arceos-bwbench",
                    "arceos-display",
                    "arceos-echoserver",
                    "arceos-exception",
                    "arceos-helloworld",
                    "arceos-httpclient",
                    "arceos-httpserver",
                    "arceos-memtest",
                    "arceos-parallel",
                    "arceos-priority",
                    "arceos-shell",
                    "arceos-sleep",
                    "arceos-tls",
                    "arceos-udpserver",
                    "arceos-yield",
                    "arceos_api",
                    "arceos_posix_api",
                    "arm_gic",
                    "arm_pl011",
                    "axalloc",
                    "axconfig",
                    "axdisplay",
                    "axdriver",
                    "axerrno",
                    "axfeat",
                    "axfs",
                    "axfs_devfs",
                    "axfs_ramfs",
                    "axfs_vfs",
                    "axhal",
                    "axio",
                    "axlibc",
                    "axlog",
                    "axnet",
                    "axruntime",
                    "axstd",
                    "axsync",
                    "axtask",
                    "capability",
                    "crate_interface",
                    "driver_block",
                    "driver_common",
                    "driver_display",
                    "driver_net",
                    "driver_pci",
                    "driver_virtio",
                    "dw_apb_uart",
                    "flatten_objects",
                    "handler_table",
                    "kernel_guard",
                    "lazy_init",
                    "linked_list",
                    "memory_addr",
                    "page_table",
                    "page_table_entry",
                    "percpu",
                    "percpu_macros",
                    "ratio",
                    "scheduler",
                    "slab_allocator",
                    "spinlock",
                    "timer_list",
                    "tuple_for_each",
                ],
                [1] root: "./tools/bwbench_client",
                [1] root.members: [
                    "bwbench-client",
                ],
                [2] root: "./tools/deptool",
                [2] root.members: [
                    "deptool",
                ],
                [3] root: "./tools/raspi4/chainloader",
                [3] root.members: [
                    "mingo",
                ],
            },
        }
    "#]]
    .assert_debug_eq(&arceos);

    expect![[r#"
        [
            Package {
                name: "allocator",
                cargo_toml: "./crates/allocator/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arceos-bwbench",
                cargo_toml: "./apps/net/bwbench/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arceos-display",
                cargo_toml: "./apps/display/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arceos-echoserver",
                cargo_toml: "./apps/net/echoserver/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arceos-exception",
                cargo_toml: "./apps/exception/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arceos-helloworld",
                cargo_toml: "./apps/helloworld/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arceos-httpclient",
                cargo_toml: "./apps/net/httpclient/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arceos-httpserver",
                cargo_toml: "./apps/net/httpserver/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arceos-memtest",
                cargo_toml: "./apps/memtest/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arceos-parallel",
                cargo_toml: "./apps/task/parallel/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arceos-priority",
                cargo_toml: "./apps/task/priority/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arceos-shell",
                cargo_toml: "./apps/fs/shell/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arceos-sleep",
                cargo_toml: "./apps/task/sleep/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arceos-tls",
                cargo_toml: "./apps/task/tls/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arceos-udpserver",
                cargo_toml: "./apps/net/udpserver/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arceos-yield",
                cargo_toml: "./apps/task/yield/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arceos_api",
                cargo_toml: "./api/arceos_api/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arceos_posix_api",
                cargo_toml: "./api/arceos_posix_api/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arm_gic",
                cargo_toml: "./crates/arm_gic/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "arm_pl011",
                cargo_toml: "./crates/arm_pl011/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axalloc",
                cargo_toml: "./modules/axalloc/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axconfig",
                cargo_toml: "./modules/axconfig/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axdisplay",
                cargo_toml: "./modules/axdisplay/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axdriver",
                cargo_toml: "./modules/axdriver/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axerrno",
                cargo_toml: "./crates/axerrno/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axfeat",
                cargo_toml: "./api/axfeat/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axfs",
                cargo_toml: "./modules/axfs/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axfs_devfs",
                cargo_toml: "./crates/axfs_devfs/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axfs_ramfs",
                cargo_toml: "./crates/axfs_ramfs/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axfs_vfs",
                cargo_toml: "./crates/axfs_vfs/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axhal",
                cargo_toml: "./modules/axhal/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axio",
                cargo_toml: "./crates/axio/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axlibc",
                cargo_toml: "./ulib/axlibc/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axlog",
                cargo_toml: "./modules/axlog/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axnet",
                cargo_toml: "./modules/axnet/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axruntime",
                cargo_toml: "./modules/axruntime/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axstd",
                cargo_toml: "./ulib/axstd/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axsync",
                cargo_toml: "./modules/axsync/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "axtask",
                cargo_toml: "./modules/axtask/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "bwbench-client",
                cargo_toml: "./Cargo.toml",
                workspace_root (file name): "bwbench_client",
            },
            Package {
                name: "capability",
                cargo_toml: "./crates/capability/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "crate_interface",
                cargo_toml: "./crates/crate_interface/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "deptool",
                cargo_toml: "./Cargo.toml",
                workspace_root (file name): "deptool",
            },
            Package {
                name: "driver_block",
                cargo_toml: "./crates/driver_block/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "driver_common",
                cargo_toml: "./crates/driver_common/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "driver_display",
                cargo_toml: "./crates/driver_display/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "driver_net",
                cargo_toml: "./crates/driver_net/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "driver_pci",
                cargo_toml: "./crates/driver_pci/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "driver_virtio",
                cargo_toml: "./crates/driver_virtio/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "dw_apb_uart",
                cargo_toml: "./crates/dw_apb_uart/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "flatten_objects",
                cargo_toml: "./crates/flatten_objects/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "handler_table",
                cargo_toml: "./crates/handler_table/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "kernel_guard",
                cargo_toml: "./crates/kernel_guard/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "lazy_init",
                cargo_toml: "./crates/lazy_init/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "linked_list",
                cargo_toml: "./crates/linked_list/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "memory_addr",
                cargo_toml: "./crates/memory_addr/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "mingo",
                cargo_toml: "./Cargo.toml",
                workspace_root (file name): "chainloader",
            },
            Package {
                name: "page_table",
                cargo_toml: "./crates/page_table/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "page_table_entry",
                cargo_toml: "./crates/page_table_entry/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "percpu",
                cargo_toml: "./crates/percpu/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "percpu_macros",
                cargo_toml: "./crates/percpu_macros/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "ratio",
                cargo_toml: "./crates/ratio/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "scheduler",
                cargo_toml: "./crates/scheduler/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "slab_allocator",
                cargo_toml: "./crates/slab_allocator/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "spinlock",
                cargo_toml: "./crates/spinlock/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "timer_list",
                cargo_toml: "./crates/timer_list/Cargo.toml",
                workspace_root (file name): "arceos",
            },
            Package {
                name: "tuple_for_each",
                cargo_toml: "./crates/tuple_for_each/Cargo.toml",
                workspace_root (file name): "arceos",
            },
        ]
    "#]]
    .assert_debug_eq(&arceos.packages());
}

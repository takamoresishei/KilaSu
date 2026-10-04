# kilasd and kila

The daemon is a Rust 2021 workspace with modules for kernel calls, configuration,
identity, lifecycle, module engine, socket protocol and OS bindings. It has no
external Cargo dependencies. Native zlib/boot helpers are compiled by build.rs.

`kilasd daemon` starts the authenticated control socket, enrolls the pinned
Manager, restores identity-bound profiles, and monitors package-map changes.
It requires an existing root init context; the Manager cannot bootstrap it by
claiming to be root. `kilasd lifecycle <stage>` processes boot stages from ROM
init. `kilasd version` reads its compiled version without requiring root.

The abstract socket uses four-byte little-endian length frames with a 64 KiB
limit and UTF-8 requests. Replies are `{"ok":true,"data":...}` or an explicit
error. Administrative requests include status, apps, modules, permission,
profile, audit, install, module and logs. ZIP upload is an authenticated size
handshake followed by exactly that bounded number of raw bytes. No arbitrary
filesystem path from an unprivileged Manager is executed.

SO_PEERCRED determines the caller. Administrative calls require root or the
unique package `io.github.kilasu.manager` with a hash matching the root-owned
manager.prop. Ordinary application callers may use only authorize-root, which
checks the live package hash against the approval database and the kernel
profile before issuing a five-second task ticket. Shared/isolated UIDs fail.

```sh
kila version
kila status
kila apps
kila modules
kila module list
kila module install /path/to/module.zip
kila module disable module_id
kila module enable module_id
kila module remove module_id
kila log
kila su -c id
```

Administration still requires the authenticated socket identity. `su` requests
authorization in the kernel; without approval it waits up to 60 seconds for
Manager rather than granting root automatically. Successful root enters a
private mount namespace, sanitizes the environment and executes Android's sh.
CAP_SYS_ADMIN is required for that namespace step in API v1.

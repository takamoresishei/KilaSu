# Kernel source integration

Run `scripts/setup.sh /path/to/kernel` from the checked-out KilaSU repository.
For a fetched setup script, set KILASU_REF to a reviewed branch/tag and run from
the intended kernel root:

```sh
curl -LSs https://raw.githubusercontent.com/takamoresishei/KilaSu/main/scripts/setup.sh | bash
```

Prefer a reviewed source checkout for reproducible builds. Integration verifies
the top-level VERSION/PATCHLEVEL/SUBLEVEL, accepts ACK 5.10/5.15/6.1, copies
kernel and UAPI under drivers/kilasu, and adds marked drivers/Kconfig and
drivers/Makefile entries. Repeated runs do not duplicate entries. An existing
unknown drivers/kilasu is refused. Source replacement uses a backup directory
and restores modified files if an operation fails.

Enable CONFIG_KILASU=y and CONFIG_KILASU_SELINUX_INTERNAL=y. The latter uses the
kernel tree's private SELinux credential blob accessor, so the kernel must be
built with SECURITY_SELINUX. It is not an externally loadable root module.
External compile smoke builds intentionally disable credential elevation.

The kernel needs separate ROM policy/init integration. Adding Kconfig source
does not install an APK, daemon, SELinux domain or su CLI. Review ABI/KMI rules,
vendor module vermagic, signing and the original kernel configuration. Preserve
the full release string used by vendor modules.

To remove source integration, remove the two marked source entries and the
owned drivers/kilasu directory. This changes source only; recovering an already
flashed device uses the recovery Uninstaller and its verified boot backup.

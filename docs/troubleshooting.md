# Troubleshooting

| Observation | Interpretation / action |
|---|---|
| Not Installed | /dev/kilasu missing or version ioctl inaccessible; verify CONFIG_KILASU, loaded kernel and device label |
| Policy Required | Driver exists, but its fixed SELinux root domain/adapter is unavailable |
| Daemon Unavailable | Check init service, executable/labels, state directory and actual daemon log |
| Manager APK pin mismatch | Use the exact APK corresponding to the trusted installer; refresh pin through a reviewed release path |
| Application identity is not approved | APK/UID/package differs from saved approval; approve current application again |
| su denied | Default-deny, consumed one-time grant, wrong namespace/thread/caller, missing ticket or policy |
| Namespace setup denied | Capability profile lacks CAP_SYS_ADMIN, policy blocks mount/unshare, or device port is incomplete |
| ZIP rejected | Check root module.prop, duplicates, expansion limits, method, CRC and unsafe paths |
| Module conflict | Two enabled modules own the same target; disable one before reboot |
| Overlay target absent | v1 file engine mounts existing /system files only |
| Source kernel checksum mismatch | Payload does not target the selected original image |
| Exact kernel release mismatch | Rebuild with the original release/vendor ABI instead of forcing a different kernel |
| Recovery source checksum mismatch | Active boot differs from the analyzed image; dump it again before patching |
| Recovery slot unknown/ambiguous | Fix recovery's boot properties/by-name layout; no guessed partition write is made |
| Data unavailable | Mount/decrypt data in recovery so a mandatory verified backup can be made |
| Uninstaller refuses changed partition | Firmware/another tool changed boot; restore deliberately using the proper original image |
| Stable release gate rejects tag | Match component versions, real device validation and signing setup |

Export Manager diagnostics and review kernel/daemon/module/installer categories.
The host test environment cannot prove recovery boot or enforcing hardware root.
Do not use setenforce 0 to turn a policy failure into an apparent successful port.

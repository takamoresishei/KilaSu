# Porting

Start with the exact device's ACK source/configuration and vendor module ABI.
The target matrix is android12-5.10, android13-5.15 and android14-6.1 on arm64.
Do not substitute a different GKI merely because its major/minor version matches.

Kernel-family selection is centralized in compat/ack.h; credential user-count
and seccomp adaptations are in compat/kila_cred.c. Core authorization should
not accumulate scattered release-condition branches. A new family must receive
compile checks, credential/domain tests and a verified enforcing runtime report.

Build the backend into the kernel. Verify all symbols/configs before packaging;
test actual modules with matching vermagic/KMI. Preserve release strings where
the current patcher requires them. Vendor DTB, firmware, init_boot/vendor_boot,
recovery image formats and boot signing remain device-specific.

Integrate Android policy and lifecycle into the ROM; ensure early-init /system
binary availability and correct post-fs-data ordering before zygote. Confirm
label restoration and daemon package-service permissions. Test denied callers,
one-time grant, repeated attempts, revocation, app reinstall/UID reuse, transferred
FDs, PID reuse and invalid UAPI messages. Then exercise module/file mounts and
recovery install/restore on the intended slot with a verified original backup.

Update validation.json only from observed results. Record device codename,
kernel source revision/config, ROM build, recovery version and checks. Do not
replace untested statuses with a generic compatibility checkmark.

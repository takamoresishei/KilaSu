# Enforcing SELinux integration

KilaSU never uses setenforce 0 as a normal installation or root workflow.
The built-in credential adapter resolves `u:r:kilasu:s0`; a missing domain or
failed resolution denies the grant. External smoke modules omit the adapter
and cannot grant KilaSU root.

Kernel credential transitions do not invent an Android policy. The target ROM
must define kilasu, kilasd, device/data/exec types, file labels, property labels,
daemon socket and lifecycle permissions. Starting inputs are provided in
kernel/selinux/kilasu.te, file_contexts, property_contexts and examples/kilasu.rc.
The early-init binary must exist in /system/bin, while mutable userspace binaries
are provisioned under /data/adb after userdata is mounted.
The source-ROM CLI is labelled `kilasu_client_exec` and stays in its caller's
application domain until the kernel grants a validated task ticket. Executing
that file alone never grants root. The private recovery-provisioned CLI is for
the root administrator; its parent directory is not accessible to ordinary apps.

Integrate these inputs into the platform ROM policy with its standard m4 macros
and compile them with the ROM's complete policy. They are a reviewable starting
point, not a certified universal policy. Keep platform neverallow validation
enabled. Adjust scopes using actual AVCs from your own device and test the
exact packages/modules that need those permissions. No wildcard allow rule is
applied to every application, and no domain is declared permissive.

The root credential transition clears previous exec/create/socket SID state;
the fixed target domain defines the resulting SELinux privileges. Successful
authorized root also releases only that task's application seccomp filter so
privileged namespace syscalls can run. Denied tasks retain their credentials,
SELinux context and filters. LSM accounting/seccomp differences live in compat.

Missing policy/init integration on a stock ROM is a current compatibility
limitation. Automatic live modification of arbitrary OEM policies is not
implemented. Check getenforce, real su execution, inherited I/O, daemon package
queries, module labels/mounts and startup ordering before marking a device valid.

# Architecture

The Manager is a controller. Credential changes happen in the kernel, while
module installation, persistent identity records and lifecycle processing live
in `kilasd`. A missing layer is reported as unavailable rather than simulated.

`kernel/core` owns interface registration, caller authorization, profiles and
version reporting. `kernel/security` owns audit records and process tickets.
`kernel/selinux` contains the built-in credential adapter plus source-ROM policy
inputs. `kernel/compat` selects ACK families. The stable ABI is in `uapi/include`.

The daemon uses `/data/adb/kilasu` with mode 0700 and files mode 0600. Its abstract
Unix socket, `kilasu.control.v1`, authenticates kernel-provided peer credentials.
Ordinary apps can only request a process authorization ticket; administrative
RPC requires the pinned Manager APK or an existing root administrator. Root
requests still need a kernel profile and a matching one-time task ticket.

Manager JNI reads version/features and calls the shared boot library. Manager
RPC controls app grants/modules and reads daemon diagnostics. It has no partition
write API. Boot patching requires a compatible compiled kernel; recovery owns
slot resolution, backups, writes and readback verification.

The design intentionally separates three compatibility questions: whether code
compiles for an ACK family, whether the kernel matches a device's vendor ABI,
and whether that device's init/SELinux integration actually works. Only the last
two establish runtime compatibility. Their evidence belongs in validation.json.

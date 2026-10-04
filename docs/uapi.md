# KilaSU UAPI v1

The canonical C header is `uapi/include/kilasu.h`. ioctl is 0xc2004b51, carrying
exactly 512 bytes. Header offsets: magic 0, API 4, command 6, length 8, reserved
12; payload starts at 16 and has 496 bytes. All fields are fixed-width, little
endian on the supported arm64/x86 test targets. No userspace pointers or
variable-length kernel buffers are accepted.

| Command | Payload size | Caller |
|---|---:|---|
| GET_VERSION / GET_FEATURES | 88 | Any permitted device opener |
| GET_MANAGER | 40 | Enrolled Manager / administrator |
| ENROLL_MANAGER | 40 | Initial-namespace root with CAP_SYS_ADMIN |
| GET_ALLOWLIST | 56 | Manager / administrator |
| GET_PROFILE | 48 | Own UID, Manager or administrator |
| SET_PERMISSION / SET_PROFILE | 48 | Privileged daemon / administrator |
| REQUEST_ROOT | 0 | Approved application with its task ticket |
| GET_AUDIT | 32 | Manager / administrator |
| SET_STAGE | 16 | Administrator |
| GET_STATUS | 16 | Manager / administrator |
| ISSUE_TICKET | 16 | Privileged daemon / administrator |

The kernel rejects bad magic, nonzero reserved fields, incorrect exact payload
length, unknown commands and unsupported API versions.
The exception is GET_VERSION: its fixed 88-byte discovery layout remains
available to clients with another API number. This lets Manager identify an old
or new backend, show its supported range, and disable unsupported controls.
All operational commands still require an explicitly supported API number.

Feature bits announce
authorization, profiles, audit, enforcing-domain availability, actual grant
adapter availability, allow-once and process-ticket support. API v1 does not
claim environment profiles, per-module hiding or mount namespace choices.

Root grant checks happen for the current task at ioctl time. Ticket issuance
accepts peer UID/PID from the daemon, validates that task's live credentials,
holds a task reference and sets a five-second expiry. Grant consumes a matching
ticket once and verifies the current UID/profile again. A duplicated descriptor,
PID reuse or a UID table alone cannot authorize another task.

Profiles and commits serialize under a mutex; audit has its own bounded spinlock
ring. Revoke affects future credential grants, not an already running root
process. List cursors index a bounded table and may need retry during updates.
GET_VERSION remains available before Manager enrollment for diagnostics.

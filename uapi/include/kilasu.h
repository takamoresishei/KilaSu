/* SPDX-License-Identifier: GPL-2.0-only WITH Linux-syscall-note */
#ifndef KILASU_UAPI_H
#define KILASU_UAPI_H
#include <linux/types.h>
#include <linux/ioctl.h>
#define KILASU_API_VERSION 1
#define KILASU_KERNEL_VERSION 0x00000100U
#define KILASU_MAGIC 0x4b494c41U
#define KILASU_MAX_APPS 1024
#define KILASU_PAYLOAD_SIZE 496
#define KILASU_F_UID_AUTH (1ULL << 0)
#define KILASU_F_PROFILES (1ULL << 1)
#define KILASU_F_AUDIT (1ULL << 2)
#define KILASU_F_SELINUX (1ULL << 3)
#define KILASU_F_GRANT_ROOT (1ULL << 4)
#define KILASU_F_ALLOW_ONCE (1ULL << 5)
#define KILASU_F_PROCESS_TICKETS (1ULL << 6)
enum kila_command {
 KILASU_CMD_GET_VERSION = 1, KILASU_CMD_GET_FEATURES,
 KILASU_CMD_GET_MANAGER, KILASU_CMD_ENROLL_MANAGER,
 KILASU_CMD_GET_ALLOWLIST, KILASU_CMD_SET_PERMISSION,
 KILASU_CMD_GET_PROFILE, KILASU_CMD_SET_PROFILE,
 KILASU_CMD_REQUEST_ROOT, KILASU_CMD_GET_AUDIT,
 KILASU_CMD_SET_STAGE, KILASU_CMD_GET_STATUS, KILASU_CMD_ISSUE_TICKET
};
enum kila_permission { KILA_DENY = 0, KILA_ALLOW = 1, KILA_ALLOW_ONCE = 2 };
enum kila_stage { KILA_EARLY_INIT, KILA_POST_FS_DATA, KILA_SERVICE, KILA_BOOT_COMPLETE };
struct kila_message {
 __u32 magic; __u16 api; __u16 command;
 __u32 length; __u32 reserved;
 __u8 payload[KILASU_PAYLOAD_SIZE];
};
struct kila_version {
 __u32 api_min; __u32 api_max; __u32 version; __u32 max_apps;
 __u64 features; char release[64];
};
struct kila_manager { __u32 uid; __u32 enrolled; __u8 apk_sha256[32]; };
/* Cursor indexes the bounded sorted in-memory table. List while quiescent. */
struct kila_profile {
 __u32 uid; __u32 permission; __u32 generation; __u32 flags;
 __u64 capabilities; __u64 last_request_ns; __u64 last_grant_ns;
 __u32 grants_remaining; __u32 reserved;
};
struct kila_list { __u32 cursor; __u32 count; struct kila_profile profile; };
struct kila_audit {
 __u64 sequence; __u64 boottime_ns; __u32 uid; __s32 result;
 __u32 command; __u32 reserved;
};
struct kila_status { __u32 stage; __u32 apps; __u32 manager_uid; __u32 reserved; };
/* Daemon supplies peer UID/PID; expiry is kernel-generated, input must be zero. */
struct kila_ticket { __u32 uid; __u32 pid; __u64 expiry_ns; };
#define KILASU_IOCTL _IOWR('K', 0x51, struct kila_message)
#endif

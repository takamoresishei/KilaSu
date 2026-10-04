// SPDX-License-Identifier: GPL-2.0-only
#include <linux/utsname.h>
#include <linux/string.h>
#include "kila_internal.h"
#include "ack.h"
void kila_version_read(struct kila_version *version)
{
 memset(version, 0, sizeof(*version));
 version->api_min = version->api_max = KILASU_API_VERSION;
 version->version = KILASU_KERNEL_VERSION;
 version->max_apps = KILASU_MAX_APPS;
 version->features = KILASU_F_UID_AUTH | KILASU_F_PROFILES |
  KILASU_F_AUDIT | KILASU_F_ALLOW_ONCE | KILASU_F_PROCESS_TICKETS;
 if (kila_selinux_ready()) version->features |= KILASU_F_SELINUX | KILASU_F_GRANT_ROOT;
 strscpy(version->release, init_uts_ns.name.release, sizeof(version->release));
}

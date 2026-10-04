// SPDX-License-Identifier: GPL-2.0-only
#include <linux/capability.h>
#include <linux/user_namespace.h>
#include "kila_internal.h"
bool kila_app_uid(u32 uid)
{
 u32 appid = uid % 100000U;
 return uid <= 2147483647U && appid >= 10000U && appid < 20000U;
}
bool kila_is_admin(void)
{
 return uid_eq(current_euid(), GLOBAL_ROOT_UID) &&
        ns_capable(&init_user_ns, CAP_SYS_ADMIN) &&
        current_user_ns() == &init_user_ns;
}

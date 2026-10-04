// SPDX-License-Identifier: GPL-2.0-only
#include <linux/security.h>
#include <linux/string.h>
#include "kila_internal.h"
#ifdef CONFIG_KILASU_SELINUX_INTERNAL
#include "objsec.h"
/* Built-in only. Accessor accounts for LSM blob offsets in ACK 5.10/5.15/6.1. */
int kila_selinux_root(struct cred *cred)
{
 const char *domain = "u:r:kilasu:s0";
 struct task_security_struct *blob = selinux_cred(cred);
 u32 sid;
 int ret;
 if (!blob) return -EACCES;
 ret = security_secctx_to_secid(domain, strlen(domain), &sid);
 if (ret || !sid) return ret ? ret : -EACCES;
 blob->sid = sid;
 blob->exec_sid = 0;
 blob->create_sid = 0;
 blob->keycreate_sid = 0;
 blob->sockcreate_sid = 0;
 return 0;
}
bool kila_selinux_ready(void)
{
 u32 sid;
 const char *domain = "u:r:kilasu:s0";
 return !security_secctx_to_secid(domain, strlen(domain), &sid) && sid;
}
#else
/* Compile smoke builds cannot turn a loadable test module into root access. */
int kila_selinux_root(struct cred *cred) { return -EOPNOTSUPP; }
bool kila_selinux_ready(void) { return false; }
#endif

// SPDX-License-Identifier: GPL-2.0-only
#include <linux/cred.h>
#include <linux/sched.h>
#include <linux/seccomp.h>
#include <linux/thread_info.h>
#include <linux/version.h>
#include "kila_internal.h"
int kila_root_account(struct cred *cred)
{
#ifdef CONFIG_KILASU_SELINUX_INTERNAL
 struct user_struct *user = alloc_uid(GLOBAL_ROOT_UID);
 if (!user) return -ENOMEM;
 free_uid(cred->user);
 cred->user = user;
#if LINUX_VERSION_CODE >= KERNEL_VERSION(5,14,0)
 return set_cred_ucounts(cred);
#else
 return 0;
#endif
#else
 return -EOPNOTSUPP;
#endif
}
void kila_root_execution(void)
{
 /* Only called after authorized credential commit. Other tasks retain filters. */
#ifdef CONFIG_KILASU_SELINUX_INTERNAL
#ifdef CONFIG_SECCOMP
#ifdef CONFIG_GENERIC_ENTRY
 clear_task_syscall_work(current, SECCOMP);
#else
 clear_thread_flag(TIF_SECCOMP);
#endif
 current->seccomp.mode = SECCOMP_MODE_DISABLED;
 seccomp_filter_release(current);
 atomic_set(&current->seccomp.filter_count, 0);
#endif
#endif
}

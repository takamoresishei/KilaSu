// SPDX-License-Identifier: GPL-2.0-only
#include <linux/capability.h>
#include <linux/ktime.h>
#include <linux/sched.h>
#include <linux/sched/signal.h>
#include <linux/user_namespace.h>
#include "kila_internal.h"
int kila_grant_current(void)
{
 struct cred *cred;
 struct group_info *groups;
 struct kila_profile *profile;
 u32 uid = from_kuid(&init_user_ns, current_uid());
 int ret, bit;
 if (!kila_app_uid(uid) || current_user_ns() != &init_user_ns ||
     !uid_eq(current_uid(), current_euid()) ||
     !thread_group_empty(current)) return -EPERM;
 mutex_lock(&kila_lock);
 profile = kila_authorized_locked(uid);
 if (!profile) { ret = -EACCES; goto out; }
 if (!kila_ticket_take_locked(uid)) { ret = -EACCES; goto out; }
 cred = prepare_creds();
 if (!cred) { ret = -ENOMEM; goto out; }
 groups = groups_alloc(0);
 if (!groups) { abort_creds(cred); ret = -ENOMEM; goto out; }
 set_groups(cred, groups);
 put_group_info(groups);
 cred->uid = cred->euid = cred->suid = cred->fsuid = GLOBAL_ROOT_UID;
 cred->gid = cred->egid = cred->sgid = cred->fsgid = GLOBAL_ROOT_GID;
 cred->securebits = 0;
 cred->cap_permitted = CAP_EMPTY_SET;
 for (bit = 0; bit <= CAP_LAST_CAP; bit++)
  if (profile->capabilities & (1ULL << bit)) cap_raise(cred->cap_permitted, bit);
 cred->cap_effective = cred->cap_permitted;
 cred->cap_bset = cred->cap_permitted;
 cred->cap_inheritable = CAP_EMPTY_SET;
 cred->cap_ambient = CAP_EMPTY_SET;
 ret = kila_root_account(cred);
 if (ret) { abort_creds(cred); goto out; }
 ret = kila_selinux_root(cred);
 if (ret) { abort_creds(cred); goto out; }
 if (profile->permission == KILA_ALLOW_ONCE) {
  profile->grants_remaining = 0;
  profile->permission = KILA_DENY;
 }
 profile->last_grant_ns = ktime_get_boottime_ns();
 ret = commit_creds(cred);
 if (!ret) kila_root_execution();
out:
 mutex_unlock(&kila_lock);
 return ret;
}

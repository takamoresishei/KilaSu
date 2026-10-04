// SPDX-License-Identifier: GPL-2.0-only
#include <linux/capability.h>
#include <linux/ktime.h>
#include "kila_internal.h"
/* Bounded table, all accesses serialize with the authorization/commit lock. */
static struct kila_profile profiles[KILASU_MAX_APPS];
static unsigned int profile_count;
static int find_profile(u32 uid)
{
 unsigned int i;
 for (i = 0; i < profile_count; i++) if (profiles[i].uid == uid) return i;
 return -1;
}
int kila_profile_get(u32 uid, struct kila_profile *profile)
{
 int i;
 mutex_lock(&kila_lock);
 i = find_profile(uid);
 if (i >= 0) *profile = profiles[i];
 mutex_unlock(&kila_lock);
 return i < 0 ? -ENOENT : 0;
}
int kila_profile_set(const struct kila_profile *profile, bool permission_only)
{
 int i;
 u32 generation;
 u64 last_request, last_grant;
 if (!kila_app_uid(profile->uid) || profile->permission > KILA_ALLOW_ONCE ||
     profile->flags || profile->reserved ||
     (profile->capabilities >> (CAP_LAST_CAP + 1))) return -EINVAL;
 mutex_lock(&kila_lock);
 i = find_profile(profile->uid);
 if (i < 0) {
  if (profile_count == KILASU_MAX_APPS) { mutex_unlock(&kila_lock); return -ENOSPC; }
  i = profile_count++;
  profiles[i].uid = profile->uid;
 }
 generation = profiles[i].generation + 1;
 last_request = profiles[i].last_request_ns;
 last_grant = profiles[i].last_grant_ns;
 if (!permission_only) profiles[i] = *profile;
 profiles[i].permission = profile->permission;
 profiles[i].grants_remaining = profile->permission == KILA_ALLOW_ONCE ? 1 : 0;
 profiles[i].generation = generation;
 profiles[i].last_request_ns = last_request;
 profiles[i].last_grant_ns = last_grant;
 mutex_unlock(&kila_lock);
 return 0;
}
int kila_profile_list(struct kila_list *list)
{
 int ret = 0;
 mutex_lock(&kila_lock);
 list->count = profile_count;
 if (list->cursor < profile_count) list->profile = profiles[list->cursor];
 else ret = -ENOENT;
 mutex_unlock(&kila_lock);
 return ret;
}
/* Called with kila_lock held through commit_creds. Revocation is serialized. */
struct kila_profile *kila_authorized_locked(u32 uid)
{
 int i = find_profile(uid);
 if (i < 0) return NULL;
 profiles[i].last_request_ns = ktime_get_boottime_ns();
 if (profiles[i].permission == KILA_ALLOW ||
     (profiles[i].permission == KILA_ALLOW_ONCE && profiles[i].grants_remaining))
  return &profiles[i];
 return NULL;
}

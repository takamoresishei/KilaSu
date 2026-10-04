// SPDX-License-Identifier: GPL-2.0-only
#include <linux/user_namespace.h>
#include "kila_internal.h"
static struct kila_manager enrolled_manager;
bool kila_is_manager(void)
{
 u32 uid = from_kuid(&init_user_ns, current_uid());
 bool ok;
 mutex_lock(&kila_lock);
 ok = enrolled_manager.enrolled && enrolled_manager.uid == uid &&
      uid_eq(current_uid(), current_euid()) && current_user_ns() == &init_user_ns;
 mutex_unlock(&kila_lock);
 return ok;
}
int kila_enroll(const struct kila_manager *manager)
{
 unsigned int i;
 bool digest = false;
 if (!kila_is_admin()) return -EPERM;
 if (manager->enrolled > 1 || (manager->enrolled && !kila_app_uid(manager->uid)))
  return -EINVAL;
 for (i = 0; i < sizeof(manager->apk_sha256); i++) digest |= manager->apk_sha256[i] != 0;
 if (manager->enrolled && !digest) return -EINVAL;
 mutex_lock(&kila_lock);
 enrolled_manager = *manager;
 if (!manager->enrolled) memset(&enrolled_manager, 0, sizeof(enrolled_manager));
 mutex_unlock(&kila_lock);
 return 0;
}
void kila_manager_read(struct kila_manager *manager)
{
 mutex_lock(&kila_lock);
 *manager = enrolled_manager;
 mutex_unlock(&kila_lock);
}

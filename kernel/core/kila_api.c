// SPDX-License-Identifier: GPL-2.0-only
#include <linux/fs.h>
#include <linux/slab.h>
#include <linux/uaccess.h>
#include <linux/user_namespace.h>
#include "kila_internal.h"
static int expected_length(u16 cmd)
{
 switch (cmd) {
 case KILASU_CMD_GET_VERSION: case KILASU_CMD_GET_FEATURES: return sizeof(struct kila_version);
 case KILASU_CMD_GET_MANAGER: case KILASU_CMD_ENROLL_MANAGER: return sizeof(struct kila_manager);
 case KILASU_CMD_GET_ALLOWLIST: return sizeof(struct kila_list);
 case KILASU_CMD_SET_PERMISSION: case KILASU_CMD_GET_PROFILE:
 case KILASU_CMD_SET_PROFILE: return sizeof(struct kila_profile);
 case KILASU_CMD_REQUEST_ROOT: return 0;
 case KILASU_CMD_GET_AUDIT: return sizeof(struct kila_audit);
 case KILASU_CMD_SET_STAGE: case KILASU_CMD_GET_STATUS: return sizeof(struct kila_status);
 case KILASU_CMD_ISSUE_TICKET: return sizeof(struct kila_ticket);
 default: return -EOPNOTSUPP;
 }
}
static int dispatch(struct kila_message *m)
{
 bool admin = kila_is_admin();
 bool manager = kila_is_manager();
 u32 caller = from_kuid(&init_user_ns, current_uid());
 struct kila_profile *p = (void *)m->payload;
 struct kila_status *s = (void *)m->payload;
 struct kila_manager km;
 struct kila_list list = {};
 if (m->command != KILASU_CMD_GET_VERSION && m->command != KILASU_CMD_GET_FEATURES &&
     m->command != KILASU_CMD_REQUEST_ROOT && m->command != KILASU_CMD_GET_PROFILE &&
     !admin && !manager) return -EPERM;
 switch (m->command) {
 case KILASU_CMD_ISSUE_TICKET: return kila_ticket_issue((void *)m->payload);
 case KILASU_CMD_GET_VERSION: case KILASU_CMD_GET_FEATURES:
  kila_version_read((void *)m->payload); return 0;
 case KILASU_CMD_GET_MANAGER: kila_manager_read((void *)m->payload); return 0;
 case KILASU_CMD_ENROLL_MANAGER: return kila_enroll((void *)m->payload);
 case KILASU_CMD_GET_ALLOWLIST: return kila_profile_list((void *)m->payload);
 case KILASU_CMD_GET_PROFILE:
  if (!admin && !manager && p->uid != caller) return -EPERM;
  return kila_profile_get(p->uid, p);
 /* Mutations use the privileged daemon; signed APK is rechecked per RPC. */
 case KILASU_CMD_SET_PERMISSION:
  return admin ? kila_profile_set(p, true) : -EPERM;
 case KILASU_CMD_SET_PROFILE:
  return admin ? kila_profile_set(p, false) : -EPERM;
 case KILASU_CMD_REQUEST_ROOT: return kila_grant_current();
 case KILASU_CMD_GET_AUDIT: return kila_audit_get((void *)m->payload);
 case KILASU_CMD_SET_STAGE:
  if (!admin || s->stage > KILA_BOOT_COMPLETE) return -EPERM;
  if (s->stage < READ_ONCE(kila_boot_stage)) return -EINVAL;
  WRITE_ONCE(kila_boot_stage, s->stage); return 0;
 case KILASU_CMD_GET_STATUS:
  memset(s, 0, sizeof(*s));
  kila_manager_read(&km);
  kila_profile_list(&list);
  s->stage = READ_ONCE(kila_boot_stage); s->apps = list.count; s->manager_uid = km.uid;
  return 0;
 default: return -EOPNOTSUPP;
 }
}
long kila_ioctl(struct file *file, unsigned int cmd, unsigned long arg)
{
 struct kila_message *m;
 int ret, length;
 u32 caller = from_kuid(&init_user_ns, current_uid());
 if (cmd != KILASU_IOCTL) return -ENOTTY;
 m = memdup_user((void __user *)arg, sizeof(*m));
 if (IS_ERR(m)) return PTR_ERR(m);
 length = expected_length(m->command);
 if (m->magic != KILASU_MAGIC || m->reserved || m->length > KILASU_PAYLOAD_SIZE)
  ret = -EINVAL;
 /* Discovery has a frozen 88-byte schema, including the supported API range. */
 else if (m->api != KILASU_API_VERSION && m->command != KILASU_CMD_GET_VERSION)
  ret = -EPROTONOSUPPORT;
 else if (length < 0) ret = length;
 else if (m->length != length) ret = -EMSGSIZE;
 else ret = dispatch(m);
 if (m->command == KILASU_CMD_REQUEST_ROOT || m->command == KILASU_CMD_SET_PERMISSION ||
     m->command == KILASU_CMD_SET_PROFILE || m->command == KILASU_CMD_ENROLL_MANAGER)
  kila_audit_add(caller, m->command, ret);
 if (!ret) {
  memset(m->payload + m->length, 0, sizeof(m->payload) - m->length);
  if (copy_to_user((void __user *)arg, m, sizeof(*m))) ret = -EFAULT;
 }
 kfree_sensitive(m);
 return ret;
}

/* SPDX-License-Identifier: GPL-2.0-only */
#ifndef KILA_INTERNAL_H
#define KILA_INTERNAL_H
#include <linux/cred.h>
#include <linux/mutex.h>
#include <kilasu.h>
extern struct mutex kila_lock;
bool kila_app_uid(u32 uid);
bool kila_is_admin(void);
bool kila_is_manager(void);
int kila_enroll(const struct kila_manager *manager);
void kila_manager_read(struct kila_manager *manager);
int kila_profile_get(u32 uid, struct kila_profile *profile);
int kila_profile_set(const struct kila_profile *profile, bool permission_only);
int kila_profile_list(struct kila_list *list);
int kila_grant_current(void);
void kila_audit_add(u32 uid, u32 command, int result);
int kila_audit_get(struct kila_audit *event);
int kila_selinux_root(struct cred *cred);
bool kila_selinux_ready(void);
void kila_version_read(struct kila_version *version);
long kila_ioctl(struct file *file, unsigned int cmd, unsigned long arg);
extern u32 kila_boot_stage;
struct kila_profile *kila_authorized_locked(u32 uid);
int kila_ticket_issue(struct kila_ticket *ticket);
bool kila_ticket_take_locked(u32 uid);
void kila_ticket_cleanup(void);
int kila_root_account(struct cred *cred);
void kila_root_execution(void);
#endif

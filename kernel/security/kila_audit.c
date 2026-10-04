// SPDX-License-Identifier: GPL-2.0-only
#include <linux/spinlock.h>
#include <linux/ktime.h>
#include <linux/user_namespace.h>
#include "kila_internal.h"
#define KILA_AUDIT_SIZE 256
static DEFINE_SPINLOCK(audit_lock);
static struct kila_audit events[KILA_AUDIT_SIZE];
static u64 next_sequence = 1;
void kila_audit_add(u32 uid, u32 command, int result)
{
 unsigned long flags;
 struct kila_audit *e;
 spin_lock_irqsave(&audit_lock, flags);
 e = &events[next_sequence % KILA_AUDIT_SIZE];
 *e = (struct kila_audit) { .sequence = next_sequence++,
  .boottime_ns = ktime_get_boottime_ns(),
  .uid = uid, .result = result, .command = command };
 spin_unlock_irqrestore(&audit_lock, flags);
}
int kila_audit_get(struct kila_audit *event)
{
 unsigned long flags;
 u64 seq;
 int ret = 0;
 spin_lock_irqsave(&audit_lock, flags);
 seq = event->sequence;
 if (!seq || (next_sequence > KILA_AUDIT_SIZE && seq < next_sequence - KILA_AUDIT_SIZE))
  seq = next_sequence > KILA_AUDIT_SIZE ? next_sequence - KILA_AUDIT_SIZE : 1;
 if (seq >= next_sequence) ret = -ENOENT;
 else *event = events[seq % KILA_AUDIT_SIZE];
 spin_unlock_irqrestore(&audit_lock, flags);
 return ret;
}

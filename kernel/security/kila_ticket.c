// SPDX-License-Identifier: GPL-2.0-only
#include <linux/pid.h>
#include <linux/sched.h>
#include <linux/sched/task.h>
#include <linux/ktime.h>
#include <linux/user_namespace.h>
#include "kila_internal.h"
#define KILA_TICKET_COUNT 256
struct ticket_slot { struct task_struct *task; u32 uid; u64 expires; };
static struct ticket_slot tickets[KILA_TICKET_COUNT];
static void clear_ticket(unsigned int i)
{
 if (tickets[i].task) put_task_struct(tickets[i].task);
 memset(&tickets[i], 0, sizeof(tickets[i]));
}
int kila_ticket_issue(struct kila_ticket *ticket)
{
 struct pid *pid;
 struct task_struct *task;
 const struct cred *cred;
 unsigned int i;
 int slot = -1;
 u64 now = ktime_get_boottime_ns();
 bool matches;
 if (!kila_is_admin()) return -EPERM;
 if (!kila_app_uid(ticket->uid) || !ticket->pid || ticket->expiry_ns) return -EINVAL;
 pid = find_get_pid(ticket->pid);
 if (!pid) return -ESRCH;
 task = get_pid_task(pid, PIDTYPE_PID);
 put_pid(pid);
 if (!task) return -ESRCH;
 cred = get_task_cred(task);
 matches = cred->user_ns == &init_user_ns &&
  from_kuid(&init_user_ns, cred->uid) == ticket->uid && uid_eq(cred->uid, cred->euid);
 put_cred(cred);
 if (!matches) { put_task_struct(task); return -EPERM; }
 mutex_lock(&kila_lock);
 for (i = 0; i < KILA_TICKET_COUNT; i++) {
  if (tickets[i].task && (tickets[i].expires <= now || tickets[i].task == task)) clear_ticket(i);
  if (!tickets[i].task && slot < 0) slot = i;
 }
 if (slot >= 0) {
  ticket->expiry_ns = now + 5ULL * NSEC_PER_SEC;
  tickets[slot] = (struct ticket_slot) { .task = task, .uid = ticket->uid, .expires = ticket->expiry_ns };
 }
 mutex_unlock(&kila_lock);
 if (slot < 0) { put_task_struct(task); return -ENOSPC; }
 return 0;
}
bool kila_ticket_take_locked(u32 uid)
{
 unsigned int i;
 u64 now = ktime_get_boottime_ns();
 for (i = 0; i < KILA_TICKET_COUNT; i++) {
  if (tickets[i].task == current && tickets[i].uid == uid) {
   bool valid = tickets[i].expires > now;
   clear_ticket(i);
   return valid;
  }
 }
 return false;
}
void kila_ticket_cleanup(void)
{
 unsigned int i;
 mutex_lock(&kila_lock);
 for (i = 0; i < KILA_TICKET_COUNT; i++) clear_ticket(i);
 mutex_unlock(&kila_lock);
}

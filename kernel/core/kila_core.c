// SPDX-License-Identifier: GPL-2.0-only
#include <linux/module.h>
#include <linux/miscdevice.h>
#include <linux/fs.h>
#include "kila_internal.h"
DEFINE_MUTEX(kila_lock);
u32 kila_boot_stage;
static const struct file_operations kila_fops = {
 .owner = THIS_MODULE,
 .unlocked_ioctl = kila_ioctl,
#ifdef CONFIG_COMPAT
 .compat_ioctl = kila_ioctl,
#endif
 .llseek = no_llseek,
};
static struct miscdevice kila_device = {
 .minor = MISC_DYNAMIC_MINOR, .name = "kilasu", .fops = &kila_fops,
 .mode = 0666, /* SELinux additionally gates every open and ioctl. */
};
static int __init kila_init(void)
{
 int ret = misc_register(&kila_device);
 if (!ret) pr_info("KilaSU API %u: default-deny interface registered\n", KILASU_API_VERSION);
 return ret;
}
static void __exit kila_exit(void) { misc_deregister(&kila_device); kila_ticket_cleanup(); }
module_init(kila_init);
module_exit(kila_exit);
MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("KilaSU UID authorization core");
MODULE_AUTHOR("KilaSU contributors");

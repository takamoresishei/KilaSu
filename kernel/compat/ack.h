/* SPDX-License-Identifier: GPL-2.0-only */
#ifndef KILA_ACK_COMPAT_H
#define KILA_ACK_COMPAT_H
#include <linux/version.h>
/* Supported adapters currently share the stable credential interfaces.
 * All release-family selection lives here, outside core authorization code.
 */
#ifndef KILASU_COMPILE_SMOKE
#if LINUX_VERSION_CODE >= KERNEL_VERSION(5,10,0) && LINUX_VERSION_CODE < KERNEL_VERSION(5,11,0)
#define KILA_ACK_FAMILY "android12-5.10"
#elif LINUX_VERSION_CODE >= KERNEL_VERSION(5,15,0) && LINUX_VERSION_CODE < KERNEL_VERSION(5,16,0)
#define KILA_ACK_FAMILY "android13-5.15"
#elif LINUX_VERSION_CODE >= KERNEL_VERSION(6,1,0) && LINUX_VERSION_CODE < KERNEL_VERSION(6,2,0)
#define KILA_ACK_FAMILY "android14-6.1"
#else
#error "KilaSU: port and test a compatibility adapter for this kernel family"
#endif
#else
#define KILA_ACK_FAMILY "compile-smoke-only"
#endif
#endif

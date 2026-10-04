# SPDX-License-Identifier: GPL-2.0-only
ifneq ($(KILASU_EXTERNAL),)
obj-m += kilasu.o
ccflags-y += -DKILASU_COMPILE_SMOKE
else
obj-$(CONFIG_KILASU) += kilasu.o
endif
kilasu-y := core/kila_core.o core/kila_api.o core/kila_uid.o \
 core/kila_manager.o core/kila_allowlist.o core/kila_profile.o \
 core/kila_version.o security/kila_audit.o security/kila_ticket.o selinux/kila_selinux.o compat/kila_cred.o
ccflags-y += -I$(src)/include -I$(src)/compat -I$(src)/../uapi/include
ccflags-$(CONFIG_KILASU_SELINUX_INTERNAL) += -I$(srctree)/security/selinux/include

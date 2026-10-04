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
kilasu-$(CONFIG_KILASU_KUNIT_TEST) += tests/kila_allowlist_test.o
kila-src := $(if $(filter /%,$(src)),$(src),$(srctree)/$(src))
ccflags-y += -I$(kila-src)/include -I$(kila-src)/compat -I$(kila-src)/../uapi/include
ccflags-$(CONFIG_KILASU_SELINUX_INTERNAL) += -I$(srctree)/security/selinux/include -I$(obj)/selinux

# Use ACK's own header generator in our output directory. This dependency also
# works for parallel full-kernel builds and driver-only compile checks.
ifeq ($(CONFIG_KILASU_SELINUX_INTERNAL),y)
$(obj)/selinux/kila_selinux.o: $(obj)/selinux/flask.h
quiet_cmd_kila_flask = GEN     $@
      cmd_kila_flask = $< $@ $(obj)/selinux/av_permissions.h
targets += selinux/flask.h selinux/av_permissions.h
$(obj)/selinux/flask.h: scripts/selinux/genheaders/genheaders FORCE
	$(call if_changed,kila_flask)
endif

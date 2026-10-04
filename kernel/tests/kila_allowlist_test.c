// SPDX-License-Identifier: GPL-2.0-only
#include <kunit/test.h>
#include <linux/capability.h>
#include "kila_internal.h"

/* Test-only Android user IDs. No credential grant is performed by these tests. */
#define TEST_UID 190010001U
#define UNKNOWN_UID 190010002U

static void default_deny(struct kunit *test)
{
 struct kila_profile p = { .uid = TEST_UID, .permission = KILA_DENY };
 KUNIT_ASSERT_EQ(test, kila_profile_set(&p, false), 0);
 mutex_lock(&kila_lock);
 KUNIT_EXPECT_FALSE(test, kila_authorized_locked(TEST_UID) != NULL);
 KUNIT_EXPECT_FALSE(test, kila_authorized_locked(UNKNOWN_UID) != NULL);
 mutex_unlock(&kila_lock);
}

static void allow_then_revoke(struct kunit *test)
{
 struct kila_profile out, p = {
  .uid = TEST_UID, .permission = KILA_ALLOW,
  .capabilities = 1ULL << CAP_SYS_ADMIN,
 };
 KUNIT_ASSERT_EQ(test, kila_profile_set(&p, false), 0);
 KUNIT_ASSERT_EQ(test, kila_profile_get(TEST_UID, &out), 0);
 KUNIT_EXPECT_EQ(test, out.capabilities, p.capabilities);
 mutex_lock(&kila_lock);
 KUNIT_EXPECT_TRUE(test, kila_authorized_locked(TEST_UID) != NULL);
 mutex_unlock(&kila_lock);
 p.permission = KILA_DENY;
 KUNIT_ASSERT_EQ(test, kila_profile_set(&p, true), 0);
 mutex_lock(&kila_lock);
 KUNIT_EXPECT_FALSE(test, kila_authorized_locked(TEST_UID) != NULL);
 mutex_unlock(&kila_lock);
 KUNIT_ASSERT_EQ(test, kila_profile_get(TEST_UID, &out), 0);
 KUNIT_EXPECT_EQ(test, out.capabilities, p.capabilities);
}

static void once_exhaustion(struct kunit *test)
{
 struct kila_profile *active, p = { .uid = TEST_UID, .permission = KILA_ALLOW_ONCE };
 KUNIT_ASSERT_EQ(test, kila_profile_set(&p, false), 0);
 mutex_lock(&kila_lock);
 active = kila_authorized_locked(TEST_UID);
 KUNIT_EXPECT_TRUE(test, active != NULL);
 if (active) {
  KUNIT_EXPECT_EQ(test, active->grants_remaining, 1U);
  active->grants_remaining = 0;
 }
 KUNIT_EXPECT_FALSE(test, kila_authorized_locked(TEST_UID) != NULL);
 mutex_unlock(&kila_lock);
 p.permission = KILA_DENY;
 KUNIT_EXPECT_EQ(test, kila_profile_set(&p, false), 0);
}

static void malformed_profile(struct kunit *test)
{
 struct kila_profile p = { .uid = 0, .permission = KILA_ALLOW };
 KUNIT_EXPECT_EQ(test, kila_profile_set(&p, false), -EINVAL);
 p.uid = TEST_UID; p.flags = 1;
 KUNIT_EXPECT_EQ(test, kila_profile_set(&p, false), -EINVAL);
 p.flags = 0; p.permission = 3;
 KUNIT_EXPECT_EQ(test, kila_profile_set(&p, false), -EINVAL);
 p.permission = KILA_ALLOW; p.capabilities = 1ULL << 63;
 KUNIT_EXPECT_EQ(test, kila_profile_set(&p, false), -EINVAL);
}

static struct kunit_case kila_cases[] = {
 KUNIT_CASE(default_deny), KUNIT_CASE(allow_then_revoke),
 KUNIT_CASE(once_exhaustion), KUNIT_CASE(malformed_profile), {}
};
static struct kunit_suite kila_suite = {
 .name = "kilasu-allowlist", .test_cases = kila_cases,
};
kunit_test_suite(kila_suite);

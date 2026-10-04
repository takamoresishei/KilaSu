/* SPDX-License-Identifier: GPL-3.0-only */
#ifndef KILA_BOOT_H
#define KILA_BOOT_H
#include <stddef.h>
#include <stdint.h>
struct kila_boot_info {
 uint32_t header_version, page_size, kernel_size, ramdisk_size, signature_size;
 uint32_t arm64, kilasu, avb_footer;
 uint64_t image_size, kernel_offset, ramdisk_offset;
 char kernel_release[128], image_sha256[65], kernel_sha256[65];
};
void kila_sha256(const uint8_t *data, size_t size, uint8_t out[32]);
int kila_sha256_fd(int fd, uint64_t limit, uint8_t out[32], uint64_t *length);
uint32_t kila_crc32(const uint8_t *data, size_t size);
int kila_inflate_raw(const uint8_t *src, size_t size, uint8_t *out, size_t length);
int kila_boot_parse(const uint8_t *data,size_t size,struct kila_boot_info *out,char *error,size_t capacity);
int kila_boot_analyze(const char *path,struct kila_boot_info *out,char *error,size_t capacity);
int kila_boot_patch(const char *source,const char *kernel,const char *output,
 const char *source_kernel_sha256,int unsigned_output,struct kila_boot_info *out,char *error,size_t capacity);
#endif

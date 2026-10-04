/* SPDX-License-Identifier: GPL-3.0-only */
#include "kila_boot.h"
#include <stdio.h>
#include <string.h>
int main(int argc,char **argv){
 struct kila_boot_info i;char error[256]={0};int ret;
 if(argc==3&&!strcmp(argv[1],"analyze"))ret=kila_boot_analyze(argv[2],&i,error,sizeof(error));
 else if((argc==6||argc==7)&&!strcmp(argv[1],"patch"))ret=kila_boot_patch(argv[2],argv[3],argv[4],argv[5],argc==7&&!strcmp(argv[6],"--unsigned"),&i,error,sizeof(error));
 else {fprintf(stderr,"usage: kila-boot analyze boot.img | patch source.img Image output.img source_kernel_sha256 [--unsigned]\n");return 2;}
 if(ret){fprintf(stderr,"kila-boot: %s\n",error);return 1;}
 printf("header_version=%u\nimage_size=%llu\nkernel_size=%u\nramdisk_size=%u\nkernel_release=%s\narm64=%u\nkilasu=%u\navb_footer=%u\nsignature_size=%u\nimage_sha256=%s\nkernel_sha256=%s\n",i.header_version,(unsigned long long)i.image_size,i.kernel_size,i.ramdisk_size,i.kernel_release,i.arm64,i.kilasu,i.avb_footer,i.signature_size,i.image_sha256,i.kernel_sha256);return 0;
}

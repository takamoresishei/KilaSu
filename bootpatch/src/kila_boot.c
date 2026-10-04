/* SPDX-License-Identifier: GPL-3.0-only
 * Independent boot container implementation, following published AOSP layout.
 * SHA-256 implements the public FIPS 180-4 algorithm; no upstream root code.
 */
#define _POSIX_C_SOURCE 200809L
#include "kila_boot.h"
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <zlib.h>
#define MAX_IMAGE (512U*1024U*1024U)
#define MAX_KERNEL (128U*1024U*1024U)
static uint32_t be32(const uint8_t *p){return ((uint32_t)p[0]<<24)|((uint32_t)p[1]<<16)|((uint32_t)p[2]<<8)|p[3];}
static uint32_t le32(const uint8_t *p){return ((uint32_t)p[3]<<24)|((uint32_t)p[2]<<16)|((uint32_t)p[1]<<8)|p[0];}
static void put32(uint8_t *p,uint32_t v){for(int i=0;i<4;i++)p[i]=(uint8_t)(v>>(i*8));}
static uint32_t rotr(uint32_t x,int n){return (x>>n)|(x<<(32-n));}
static const uint32_t constants[64]={
 0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
 0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
 0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
 0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
 0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
 0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
 0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
 0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2};
static void sha_block(uint32_t h[8],const uint8_t p[64]){
 uint32_t w[64],a=h[0],b=h[1],c=h[2],d=h[3],e=h[4],f=h[5],g=h[6],v=h[7];
 for(int i=0;i<16;i++)w[i]=be32(p+4*i);
 for(int i=16;i<64;i++){uint32_t s0=rotr(w[i-15],7)^rotr(w[i-15],18)^(w[i-15]>>3);uint32_t s1=rotr(w[i-2],17)^rotr(w[i-2],19)^(w[i-2]>>10);w[i]=w[i-16]+s0+w[i-7]+s1;}
 for(int i=0;i<64;i++){uint32_t s1=rotr(e,6)^rotr(e,11)^rotr(e,25);uint32_t ch=(e&f)^(~e&g);uint32_t t1=v+s1+ch+constants[i]+w[i];uint32_t s0=rotr(a,2)^rotr(a,13)^rotr(a,22);uint32_t t2=s0+((a&b)^(a&c)^(b&c));v=g;g=f;f=e;e=d+t1;d=c;c=b;b=a;a=t1+t2;}
 h[0]+=a;h[1]+=b;h[2]+=c;h[3]+=d;h[4]+=e;h[5]+=f;h[6]+=g;h[7]+=v;
}
void kila_sha256(const uint8_t *data,size_t size,uint8_t out[32]){
 uint32_t h[8]={0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19};
 size_t full=size/64;for(size_t i=0;i<full;i++)sha_block(h,data+64*i);
 uint8_t tail[128]={0};size_t rest=size%64;memcpy(tail,data+full*64,rest);tail[rest]=0x80;size_t blocks=rest>=56?2:1;uint64_t bits=(uint64_t)size*8;
 for(int i=0;i<8;i++)tail[blocks*64-1-i]=(uint8_t)(bits>>(8*i));
 sha_block(h,tail);if(blocks==2)sha_block(h,tail+64);
 for(int i=0;i<8;i++)for(int j=0;j<4;j++)out[i*4+j]=(uint8_t)(h[i]>>(24-8*j));
}
int kila_sha256_fd(int fd,uint64_t limit,uint8_t out[32],uint64_t *length){
 uint32_t h[8]={0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19};
 uint8_t buffer[4096],tail[128]={0};size_t used=0;uint64_t total=0;
 if(fd<0||!out||!length||limit>UINT64_MAX/8){errno=EINVAL;return -1;}
 for(;;){
  ssize_t n=read(fd,buffer,sizeof(buffer));
  if(n<0){if(errno==EINTR)continue;return -1;}
  if(!n)break;
  if((uint64_t)n>limit-total){errno=EFBIG;return -1;}
  total+=(uint64_t)n;size_t offset=0;
  while(offset<(size_t)n){
   size_t take=64-used;if(take>(size_t)n-offset)take=(size_t)n-offset;
   memcpy(tail+used,buffer+offset,take);used+=take;offset+=take;
   if(used==64){sha_block(h,tail);used=0;}
  }
 }
 memset(tail+used,0,sizeof(tail)-used);tail[used]=0x80;
 size_t blocks=used>=56?2:1;uint64_t bits=total*8;
 for(int i=0;i<8;i++)tail[blocks*64-1-i]=(uint8_t)(bits>>(8*i));
 sha_block(h,tail);if(blocks==2)sha_block(h,tail+64);
 for(int i=0;i<8;i++)for(int j=0;j<4;j++)out[i*4+j]=(uint8_t)(h[i]>>(24-8*j));
 *length=total;return 0;
}
static void hash_hex(const uint8_t *data,size_t size,char out[65]){uint8_t hash[32];const char *hex="0123456789abcdef";kila_sha256(data,size,hash);for(int i=0;i<32;i++){out[2*i]=hex[hash[i]>>4];out[2*i+1]=hex[hash[i]&15];}out[64]=0;}
uint32_t kila_crc32(const uint8_t *data,size_t size){return (uint32_t)crc32(0,data,(uInt)size);}
int kila_inflate_raw(const uint8_t *src,size_t size,uint8_t *out,size_t length){
 if(size>UINT32_MAX||length>UINT32_MAX)return -1;
 z_stream s={0};s.next_in=(Bytef *)src;s.avail_in=(uInt)size;s.next_out=out;s.avail_out=(uInt)length;
 uint8_t dummy;if(!length){s.next_out=&dummy;s.avail_out=1;}
 if(inflateInit2(&s,-15)!=Z_OK)return -1;
 int ret=inflate(&s,Z_FINISH);int ok=ret==Z_STREAM_END&&s.total_out==length&&s.total_in==size;inflateEnd(&s);return ok?0:-1;
}
static int fail(char *error,size_t capacity,const char *message){if(capacity)snprintf(error,capacity,"%s",message);return -1;}
static uint64_t aligned(uint64_t n,uint32_t page){return (n+page-1)&~((uint64_t)page-1);}
static int kernel_info(const uint8_t *data,size_t size,struct kila_boot_info *out,char *error,size_t capacity){
 uint8_t *unpacked=NULL;
 if(size>=2&&data[0]==0x1f&&data[1]==0x8b){
  unpacked=malloc(MAX_KERNEL);if(!unpacked)return fail(error,capacity,"memory allocation failed");
  z_stream s={0};s.next_in=(Bytef *)data;s.avail_in=(uInt)size;s.next_out=unpacked;s.avail_out=MAX_KERNEL;
  if(inflateInit2(&s,31)!=Z_OK){free(unpacked);return fail(error,capacity,"gzip initialization failed");}
  int ret=inflate(&s,Z_FINISH);size_t n=s.total_out;int valid=ret==Z_STREAM_END&&s.total_in==size;inflateEnd(&s);
  if(!valid){free(unpacked);return fail(error,capacity,"invalid or oversized gzip kernel");}data=unpacked;size=n;
 }
 out->arm64=size>=64&&le32(data+56)==0x644d5241;
 const char *tag="Linux version ";size_t taglen=strlen(tag);
 for(size_t i=0;i+taglen<size;i++)if(!memcmp(data+i,tag,taglen)){size_t j=0;while(i+taglen+j<size&&j<127){uint8_t c=data[i+taglen+j];if(c<=32||c>126)break;out->kernel_release[j++]=(char)c;}out->kernel_release[j]=0;break;}
 const char *marker="KilaSU API";size_t m=strlen(marker);for(size_t i=0;i+m<=size;i++)if(!memcmp(data+i,marker,m)){out->kilasu=1;break;}
 free(unpacked);return 0;
}
int kila_boot_parse(const uint8_t *data,size_t size,struct kila_boot_info *out,char *error,size_t capacity){
 if(size<44||size>MAX_IMAGE)return fail(error,capacity,"boot image size outside bounds");
 if(memcmp(data,"ANDROID!",8))return fail(error,capacity,"unsupported container: expected ANDROID! boot image");
 memset(out,0,sizeof(*out));out->image_size=size;out->header_version=le32(data+40);
 if(out->header_version>4)return fail(error,capacity,"unsupported boot header version");
 uint32_t version=out->header_version;
 if(version>=3){
  uint32_t expected=version==4?1584:1580;if(size<expected||le32(data+20)!=expected)return fail(error,capacity,"invalid GKI header size");
  for(int i=24;i<40;i++)if(data[i])return fail(error,capacity,"nonzero reserved GKI header field");
  out->page_size=4096;out->kernel_size=le32(data+8);out->ramdisk_size=le32(data+12);if(version==4)out->signature_size=le32(data+1580);
 }else{
  size_t minimum=version==0?1632:version==1?1648:1660;if(size<minimum)return fail(error,capacity,"truncated legacy boot header");
  out->page_size=le32(data+36);out->kernel_size=le32(data+8);out->ramdisk_size=le32(data+16);
  if(out->page_size<2048||out->page_size>65536||(out->page_size&(out->page_size-1)))return fail(error,capacity,"invalid boot page size");
 }
 out->kernel_offset=out->page_size;out->ramdisk_offset=out->kernel_offset+aligned(out->kernel_size,out->page_size);
 uint64_t end=out->ramdisk_offset+aligned(out->ramdisk_size,out->page_size)+out->signature_size;
 if(!out->kernel_size||out->kernel_size>MAX_KERNEL||out->kernel_offset+out->kernel_size>size||end>size)return fail(error,capacity,"boot sections outside file bounds");
 if(size>=64&&!memcmp(data+size-64,"AVBf",4))out->avb_footer=1;
 hash_hex(data,size,out->image_sha256);hash_hex(data+out->kernel_offset,out->kernel_size,out->kernel_sha256);
 return kernel_info(data+out->kernel_offset,out->kernel_size,out,error,capacity);
}
static int read_file(const char *path,uint8_t **data,size_t *size,char *error,size_t cap){
 FILE *f=fopen(path,"rb");if(!f)return fail(error,cap,"cannot open input file");
 if(fseek(f,0,SEEK_END)){fclose(f);return fail(error,cap,"input must be a seekable regular file");}
 long n=ftell(f);if(n<0||(unsigned long)n>MAX_IMAGE){fclose(f);return fail(error,cap,"file size outside bounds");}rewind(f);
 *size=(size_t)n;*data=malloc(*size?*size:1);if(!*data){fclose(f);return fail(error,cap,"memory allocation failed");}
 if(fread(*data,1,*size,f)!=*size){fclose(f);free(*data);*data=NULL;return fail(error,cap,"file read failed");}fclose(f);return 0;
}
int kila_boot_analyze(const char *path,struct kila_boot_info *out,char *error,size_t capacity){uint8_t *b=NULL;size_t n=0;if(read_file(path,&b,&n,error,capacity))return -1;int ret=kila_boot_parse(b,n,out,error,capacity);free(b);return ret;}
int kila_boot_patch(const char *source,const char *kernel,const char *output,const char *source_sha,int unsigned_output,struct kila_boot_info *out,char *error,size_t capacity){
 uint8_t *b=NULL,*k=NULL,*result=NULL;size_t n=0,kn=0;int ret=-1;struct kila_boot_info original,replacement={0};
 if(!strcmp(source,output)||!strcmp(kernel,output))return fail(error,capacity,"output must differ from input");
 if(read_file(source,&b,&n,error,capacity)||read_file(kernel,&k,&kn,error,capacity))goto done;
 if(kila_boot_parse(b,n,&original,error,capacity))goto done;
 if(original.header_version<3){fail(error,capacity,"legacy headers can be analyzed; patching requires GKI v3 or v4");goto done;}
 if(!original.arm64){fail(error,capacity,"source kernel is not a supported arm64 Image or Image.gz");goto done;}
 if(!source_sha||strlen(source_sha)!=64||strcmp(original.kernel_sha256,source_sha)){fail(error,capacity,"payload source kernel checksum mismatch");goto done;}
 if((original.signature_size||original.avb_footer)&&!unsigned_output){fail(error,capacity,"signed input requires explicit unsigned export for an unlocked bootloader");goto done;}
 if(!kn||kn>MAX_KERNEL||kernel_info(k,kn,&replacement,error,capacity))goto done;
 if(!replacement.arm64||!replacement.kilasu||!original.kernel_release[0]||strcmp(replacement.kernel_release,original.kernel_release)){fail(error,capacity,"replacement must contain KilaSU and preserve the exact kernel release for vendor module compatibility");goto done;}
 uint64_t ramdisk=4096+aligned(kn,4096);uint64_t end=ramdisk+aligned(original.ramdisk_size,4096);if(end>n){fail(error,capacity,"replacement exceeds original boot image capacity");goto done;}
 result=calloc(n,1);if(!result){fail(error,capacity,"memory allocation failed");goto done;}
 memcpy(result,b,4096);put32(result+8,(uint32_t)kn);if(original.header_version==4)put32(result+1580,0);
 memcpy(result+4096,k,kn);memcpy(result+ramdisk,b+original.ramdisk_offset,original.ramdisk_size);
 if(kila_boot_parse(result,n,out,error,capacity))goto done;
 if(out->ramdisk_size!=original.ramdisk_size||memcmp(result+out->ramdisk_offset,b+original.ramdisk_offset,original.ramdisk_size)){fail(error,capacity,"ramdisk validation failed");goto done;}
 int fd=open(output,O_WRONLY|O_CREAT|O_EXCL|O_CLOEXEC,0600);if(fd<0){fail(error,capacity,"output exists or cannot be created");goto done;}
 size_t written=0;while(written<n){ssize_t count=write(fd,result+written,n-written);if(count<0&&errno==EINTR)continue;if(count<=0)break;written+=(size_t)count;}
 int ok=written==n&&fsync(fd)==0;close(fd);if(!ok){unlink(output);fail(error,capacity,"output write failed");goto done;}ret=0;
done:free(b);free(k);free(result);return ret;
}

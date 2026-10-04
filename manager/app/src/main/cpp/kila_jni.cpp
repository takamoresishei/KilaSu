// SPDX-License-Identifier: GPL-3.0-only
#include <jni.h>
#include <fcntl.h>
#include <sys/ioctl.h>
#include <unistd.h>
#include <cerrno>
#include <cstring>
#include <sstream>
#include <string>
#include "kilasu.h"
extern "C" {
#include "kila_boot.h"
}
static std::string quote(const char *s){std::string r="\"";for(const unsigned char *p=(const unsigned char *)s;*p;p++){if(*p=='"'||*p=='\\')r+='\\';if(*p>=32)r+=(char)*p;}return r+'"';}
static std::string error(const char *s){return "{\"ok\":false,\"error\":"+quote(s)+"}";}
static std::string info(const kila_boot_info &i){std::ostringstream s;s<<"{\"ok\":true,\"data\":{\"headerVersion\":"<<i.header_version<<",\"kernel\":"<<quote(i.kernel_release)<<",\"size\":"<<i.image_size<<",\"kernelSha256\":"<<quote(i.kernel_sha256)<<",\"checksum\":"<<quote(i.image_sha256)<<",\"arm64\":"<<i.arm64<<",\"kilasu\":"<<i.kilasu<<",\"signed\":"<<(i.avb_footer||i.signature_size)<<"}}";return s.str();}
class Utf {JNIEnv *env;jstring source;const char *data;public:Utf(JNIEnv *e,jstring s):env(e),source(s),data(s?e->GetStringUTFChars(s,nullptr):nullptr){}~Utf(){if(data)env->ReleaseStringUTFChars(source,data);}const char *get() const{return data?data:"";}};
extern "C" JNIEXPORT jstring JNICALL Java_io_github_kilasu_manager_Native_status(JNIEnv *e,jobject){
 int fd=open("/dev/kilasu",O_RDWR|O_CLOEXEC);if(fd<0)return e->NewStringUTF(error(strerror(errno)).c_str());
 kila_message m={};m.magic=KILASU_MAGIC;m.api=KILASU_API_VERSION;m.command=KILASU_CMD_GET_VERSION;m.length=sizeof(kila_version);
 if(ioctl(fd,KILASU_IOCTL,&m)<0){std::string s=error(strerror(errno));close(fd);return e->NewStringUTF(s.c_str());}close(fd);
 kila_version v;memcpy(&v,m.payload,sizeof(v));v.release[sizeof(v.release)-1]=0;std::ostringstream s;s<<"{\"ok\":true,\"data\":{\"api\":"<<v.api_max<<",\"apiMin\":"<<v.api_min<<",\"features\":"<<v.features<<",\"kernelVersion\":"<<v.version<<",\"kernel\":"<<quote(v.release)<<"}}";return e->NewStringUTF(s.str().c_str());
}
extern "C" JNIEXPORT jstring JNICALL Java_io_github_kilasu_manager_Native_analyze(JNIEnv *e,jobject,jstring path){Utf p(e,path);kila_boot_info i;char err[256];if(kila_boot_analyze(p.get(),&i,err,sizeof(err)))return e->NewStringUTF(error(err).c_str());return e->NewStringUTF(info(i).c_str());}
extern "C" JNIEXPORT jstring JNICALL Java_io_github_kilasu_manager_Native_patch(JNIEnv *e,jobject,jstring source,jstring kernel,jstring output,jstring hash,jboolean unsigned_output){Utf s(e,source),k(e,kernel),o(e,output),h(e,hash);kila_boot_info i;char err[256];if(kila_boot_patch(s.get(),k.get(),o.get(),h.get(),unsigned_output,&i,err,sizeof(err)))return e->NewStringUTF(error(err).c_str());return e->NewStringUTF(info(i).c_str());}

# SPDX-License-Identifier: GPL-3.0-only
import ctypes as C
import gzip
import hashlib
from pathlib import Path
import random
import struct
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
class Info(C.Structure):
    _fields_ = [(key, C.c_uint32) for key in ['header_version','page_size','kernel_size','ramdisk_size','signature_size','arm64','kilasu','avb_footer']] + [(key,C.c_uint64) for key in ['image_size','kernel_offset','ramdisk_offset']] + [('kernel_release',C.c_char*128),('image_sha256',C.c_char*65),('kernel_sha256',C.c_char*65)]
def kernel(kila=False, release='5.10.198-test'):
    data = bytearray(16000)
    struct.pack_into('<I', data, 56, 0x644d5241)
    text = f'Linux version {release} builder\0'.encode()
    data[100:100+len(text)] = text
    if kila:
        data[600:610] = b'KilaSU API'
    return bytes(data)
def boot(version=4, image=None, signed=False, ramdisk=b'ramdisk-preservation-test'):
    image = image or kernel()
    data = bytearray(131072)
    data[:8] = b'ANDROID!'
    struct.pack_into('<III',data,8,len(image),len(ramdisk),0)
    struct.pack_into('<I',data,20,1584 if version==4 else 1580)
    struct.pack_into('<I',data,40,version)
    data[4096:4096+len(image)] = image
    offset = 4096 + ((len(image)+4095)//4096)*4096
    data[offset:offset+len(ramdisk)] = ramdisk
    if signed:
        data[-64:-60] = b'AVBf'
    return bytes(data)
class BootTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        lib = Path(cls.temp.name)/'lib.so'
        subprocess.run(['gcc','-std=c11','-Wall','-Wextra','-Werror','-fPIC','-shared',str(ROOT/'bootpatch/src/kila_boot.c'),'-lz','-o',str(lib)],check=True)
        cls.lib = C.CDLL(str(lib))
        cls.lib.kila_boot_parse.argtypes = [C.c_void_p,C.c_size_t,C.POINTER(Info),C.c_char_p,C.c_size_t]
        cls.lib.kila_boot_patch.argtypes = [C.c_char_p]*4+[C.c_int,C.POINTER(Info),C.c_char_p,C.c_size_t]
        cls.lib.kila_sha256.argtypes = [C.c_void_p,C.c_size_t,C.c_void_p]
    @classmethod
    def tearDownClass(cls): cls.temp.cleanup()
    def parse(self, data):
        info=Info(); error=C.create_string_buffer(256)
        result=self.lib.kila_boot_parse(data,len(data),C.byref(info),error,256)
        return result,info,error.value
    def test_v3_and_v4(self):
        for version in [3,4]:
            result,info,error=self.parse(boot(version))
            self.assertEqual(result,0,error)
            self.assertEqual(info.header_version,version)
            self.assertEqual(info.image_sha256.decode(),hashlib.sha256(boot(version)).hexdigest())
            self.assertEqual(info.kernel_release,b'5.10.198-test')
    def test_gzip_kernel(self):
        result,info,error=self.parse(boot(image=gzip.compress(kernel())))
        self.assertEqual(result,0,error);self.assertEqual(info.arm64,1)
    def test_truncated_and_unsupported(self):
        for data in [b'',b'ANDROID!',boot()[:1500],boot()[:5000],boot(version=5),b'VNDRBOOT'+bytes(16000)]:
            self.assertNotEqual(self.parse(data)[0],0)
    def test_overflow_and_reserved(self):
        data=bytearray(boot());struct.pack_into('<I',data,8,0xffffffff)
        self.assertNotEqual(self.parse(bytes(data))[0],0)
        data=bytearray(boot());data[24]=1
        self.assertNotEqual(self.parse(bytes(data))[0],0)
    def test_patch_preserves_ramdisk(self):
        with tempfile.TemporaryDirectory() as tmp:
            base=Path(tmp);source=base/'source.img';image=base/'Image';output=base/'output.img'
            original=boot();source.write_bytes(original);image.write_bytes(kernel(kila=True));info=Info();error=C.create_string_buffer(256)
            result=self.lib.kila_boot_patch(str(source).encode(),str(image).encode(),str(output).encode(),hashlib.sha256(kernel()).hexdigest().encode(),0,C.byref(info),error,256)
            self.assertEqual(result,0,error.value);self.assertEqual(info.kilasu,1)
            self.assertEqual(output.stat().st_size,len(original))
            parsed=self.parse(original)[1]
            self.assertEqual(output.read_bytes()[info.ramdisk_offset:info.ramdisk_offset+info.ramdisk_size],original[parsed.ramdisk_offset:parsed.ramdisk_offset+parsed.ramdisk_size])
    def test_signed_and_wrong_payload_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            base=Path(tmp);source=base/'source.img';image=base/'Image';output=base/'output.img'
            source.write_bytes(boot(signed=True));image.write_bytes(kernel(kila=True));info=Info();error=C.create_string_buffer(256)
            args=[str(source).encode(),str(image).encode(),str(output).encode(),hashlib.sha256(kernel()).hexdigest().encode()]
            self.assertNotEqual(self.lib.kila_boot_patch(*args,0,C.byref(info),error,256),0)
            self.assertFalse(output.exists())
            self.assertEqual(self.lib.kila_boot_patch(*args,1,C.byref(info),error,256),0,error.value)
            self.assertEqual(info.avb_footer,0)
    def test_sha_vectors(self):
        for data in [b'',b'abc',bytes(range(255)),b'a'*100000]:
            out=C.create_string_buffer(32);self.lib.kila_sha256(data,len(data),out)
            self.assertEqual(out.raw,hashlib.sha256(data).digest())
    def test_corrupt_headers_fail_without_crash(self):
        rng=random.Random(241)
        for _ in range(100):
            data=bytearray(boot());position=rng.randrange(8,44);data[position]=rng.randrange(256)
            self.parse(bytes(data))

if __name__=='__main__': unittest.main()

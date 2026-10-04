# SPDX-License-Identifier: GPL-3.0-only
from pathlib import Path
import subprocess
import tempfile
import unittest
ROOT=Path(__file__).resolve().parents[1]
class AbiTests(unittest.TestCase):
    def test_c_abi(self):
        source='''#include <kilasu.h>
_Static_assert(sizeof(struct kila_message)==512,"message ABI");
_Static_assert(sizeof(struct kila_version)==88,"version ABI");
_Static_assert(sizeof(struct kila_profile)==48,"profile ABI");
_Static_assert(sizeof(struct kila_list)==56,"list ABI");
_Static_assert(sizeof(struct kila_ticket)==16,"ticket ABI");
_Static_assert(KILASU_IOCTL==0xc2004b51UL,"ioctl ABI");
int main(void){return 0;}
'''
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp);(path/'test.c').write_text(source)
            subprocess.run(['gcc','-std=c11','-Wall','-Werror','-I',str(ROOT/'uapi/include'),str(path/'test.c'),'-o',str(path/'test')],check=True)
            subprocess.run([str(path/'test')],check=True)

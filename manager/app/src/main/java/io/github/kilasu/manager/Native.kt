// SPDX-License-Identifier: GPL-3.0-only
package io.github.kilasu.manager

object Native {
 init { System.loadLibrary("kila-native") }
 external fun status(): String
 external fun analyze(path: String): String
 external fun patch(source: String, kernel: String, output: String, sourceKernelHash: String, unsignedOutput: Boolean): String
}

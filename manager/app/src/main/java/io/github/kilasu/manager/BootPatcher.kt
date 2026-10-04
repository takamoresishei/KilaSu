// SPDX-License-Identifier: GPL-3.0-only
package io.github.kilasu.manager

import android.content.Context
import android.net.Uri
import android.os.Build
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONObject
import java.io.File
import java.io.InputStream
import java.io.ByteArrayOutputStream
import java.security.MessageDigest
import java.util.zip.ZipInputStream
import java.util.zip.ZipOutputStream
import java.util.zip.ZipEntry

class BootPatcher(private val context: Context) {
 data class Source(val file: File, val name: String, val info: JSONObject)
 data class Payload(val image: File, val properties: Map<String, String>)
 data class Result(val file: File, val receipt: File, val info: JSONObject, val log: String)
 private fun copy(uri: Uri, prefix: String, max: Long): File {
  val file = File.createTempFile(prefix, ".bin", context.cacheDir)
  try { context.contentResolver.openInputStream(uri)!!.use { input -> file.outputStream().use { out ->
   val b = ByteArray(65536); var count = 0L
   while (true) { val n = input.read(b); if (n < 0) break; count += n; require(count <= max) { "Input exceeds size limit" }; out.write(b, 0, n) }
  } }; return file } catch (e: Exception) { file.delete(); throw e }
 }
 suspend fun source(uri: Uri): Source = withContext(Dispatchers.IO) {
  val file = copy(uri, "boot-", 512L * 1024 * 1024)
  try { Source(file, uri.lastPathSegment ?: "boot.img", KilaApi.unwrap(Native.analyze(file.path)) as JSONObject) }
  catch (e: Exception) { file.delete(); throw e }
 }
 suspend fun payload(uri: Uri): Payload = withContext(Dispatchers.IO) {
  val image = File.createTempFile("kila-kernel-", ".bin", context.cacheDir)
  var properties: Map<String, String>? = null; var hasImage = false; val names = mutableSetOf<String>()
  try {
   context.contentResolver.openInputStream(uri)!!.use { raw -> ZipInputStream(raw).use { zip ->
    var count = 0
    while (true) { val entry = zip.nextEntry ?: break; count++; require(count <= 2 && names.add(entry.name)) { "Duplicate or excessive payload entries" }
     when (entry.name) {
      "Image" -> { var length = 0L; image.outputStream().use { out -> val b = ByteArray(65536); while (true) { val n = zip.read(b); if (n < 0) break; length += n; require(length <= 128L * 1024 * 1024); out.write(b, 0, n) } }; hasImage = length > 0 }
      "payload.prop" -> { properties = parseProperties(readBounded(zip, 65536).toString(Charsets.UTF_8)) }
      else -> error("Payload only accepts Image and payload.prop")
     }; zip.closeEntry()
    }
   } }
   check(hasImage); val p = checkNotNull(properties)
   require(p["device"] == Build.DEVICE) { "Payload targets a different device" }
   require(p["api"] == KilaApi.API.toString()) { "Payload API mismatch" }
   require(p["policy_in_rom"] == "true" && p["daemon_init_in_rom"] == "true") { "Payload requires enforcing SELinux and init integration in the ROM" }
   require(p["kernel_sha256"] == hash(image)) { "Payload kernel checksum mismatch" }
   Payload(image, p)
  } catch (e: Exception) { image.delete(); throw e }
 }
 suspend fun patch(source: Source, payload: Payload, unsigned: Boolean): Result = withContext(Dispatchers.IO) {
  val release = source.info.getString("kernel").replace(Regex("[^A-Za-z0-9._-]"), "_")
  val output = File(context.cacheDir, "KilaSU-patched-$release-${System.currentTimeMillis()}.img")
  val info = KilaApi.unwrap(Native.patch(source.file.path, payload.image.path, output.path, payload.properties.getValue("source_kernel_sha256"), unsigned)) as JSONObject
  val receipt = File(output.path + ".prop")
  receipt.writeText("device=${Build.DEVICE}\nsource_sha256=${source.info.getString("checksum")}\nsource_size=${source.file.length()}\npatched_sha256=${info.getString("checksum")}\nkernel_release=${info.getString("kernel")}\napi=${KilaApi.API}\npatch_version=1\nkilasu_version=${BuildConfig.VERSION_NAME}\npolicy_in_rom=true\ndaemon_init_in_rom=true\nunsigned_export=$unsigned\n")
  Result(output, receipt, info, "Analyzed header v${info.getInt("headerVersion")}\nVerified source kernel and payload SHA-256\nIntegrated compiled KilaSU kernel\nPreserved original ramdisk\nRepacked and validated output\nSHA-256: ${info.getString("checksum")}")
 }
 suspend fun export(file: File, uri: Uri) = withContext(Dispatchers.IO) { context.contentResolver.openOutputStream(uri, "w")!!.use { out -> file.inputStream().use { it.copyTo(out) } } }
 suspend fun bundle(result: Result, uri: Uri) = withContext(Dispatchers.IO) {
  context.contentResolver.openOutputStream(uri, "w")!!.use { out -> ZipOutputStream(out).use { zip ->
   zip.putNextEntry(ZipEntry("patched.img")); result.file.inputStream().use { it.copyTo(zip) }; zip.closeEntry()
   zip.putNextEntry(ZipEntry("receipt.prop")); result.receipt.inputStream().use { it.copyTo(zip) }; zip.closeEntry()
  } }
 }
 companion object {
  fun readBounded(input: InputStream, limit: Int): ByteArray {
   require(limit in 1..65536)
   val output = ByteArrayOutputStream(); val buffer = ByteArray(4096)
   while (true) {
    val n = input.read(buffer); if (n < 0) break
    require(n > 0 && output.size() + n <= limit) { "Payload properties exceed size limit" }
    output.write(buffer, 0, n)
   }
   return output.toByteArray()
  }
  fun parseProperties(text: String): Map<String, String> {
   val result = mutableMapOf<String, String>()
   for (line in text.lineSequence().filter { it.isNotBlank() && !it.startsWith('#') }) {
    val split = line.split('=', limit = 2); require(split.size == 2 && split[0].matches(Regex("[A-Za-z0-9_.]+")) && !result.containsKey(split[0]) && split[1].none { it.isISOControl() })
    result[split[0]] = split[1]
   }; return result
  }
  fun hash(file: File): String { val md = MessageDigest.getInstance("SHA-256"); file.inputStream().use { val b = ByteArray(65536); while (true) { val n = it.read(b); if (n < 0) break; md.update(b, 0, n) } }; return md.digest().joinToString("") { "%02x".format(it) } }
 }
}

// SPDX-License-Identifier: GPL-3.0-only
package io.github.kilasu.manager

import android.content.Context
import android.net.LocalSocket
import android.net.LocalSocketAddress
import android.net.Uri
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject
import java.io.InputStream
import java.io.OutputStream

object Wire {
 const val MAX_FRAME = 65536
 fun send(out: OutputStream, text: String) {
  val bytes = text.toByteArray(Charsets.UTF_8)
  require(bytes.size in 1..MAX_FRAME)
  repeat(4) { out.write(bytes.size ushr (it * 8) and 255) }
  out.write(bytes); out.flush()
 }
 fun receive(input: InputStream): String {
  var size = 0
  repeat(4) { val byte = input.read(); check(byte >= 0) { "Truncated daemon frame" }; size = size or (byte shl (it * 8)) }
  require(size in 1..MAX_FRAME) { "Invalid daemon frame size" }
  val bytes = ByteArray(size); var offset = 0
  while (offset < size) { val n = input.read(bytes, offset, size - offset); check(n > 0) { "Daemon disconnected" }; offset += n }
  return bytes.toString(Charsets.UTF_8)
 }
}
data class Backend(val kernel: JSONObject? = null, val daemon: JSONObject? = null, val error: String? = null) {
 val installed get() = kernel != null
 val compatible get() = kernel?.let { KilaApi.supportsApi(it.optInt("apiMin", it.optInt("api")), it.optInt("api")) } ?: false
 val rootReady get() = compatible && (kernel!!.optLong("features") and 16L != 0L)
 val operational get() = rootReady && daemon != null
}
data class RootApp(val pkg: String, val uid: Int, val permission: Int, val caps: Long, val lastRequest: Long, val lastGrant: Long)
data class Module(val id: String, val name: String, val version: String, val author: String, val description: String, val enabled: Boolean, val remove: Boolean, val reboot: Boolean)
class KilaApi(private val context: Context) {
 companion object {
  const val API = 1
  const val FULL_CAPS = (1L shl 41) - 1
  fun supportsApi(min: Int, max: Int): Boolean = min > 0 && min <= API && API <= max
  fun unwrap(text: String): Any {
   val response = JSONObject(text)
   check(response.optBoolean("ok")) { response.optString("error", "Backend operation failed") }
   return response.get("data")
  }
 }
 private fun socket() = LocalSocket().apply {
  try {
   soTimeout = 180000
   connect(LocalSocketAddress("kilasu.control.v1", LocalSocketAddress.Namespace.ABSTRACT))
   check(peerCredentials.uid == 0) { "Control socket is not owned by the root daemon" }
  } catch (e: Exception) { close(); throw e }
 }
 suspend fun rpc(command: String): Any = withContext(Dispatchers.IO) {
  socket().use { s -> Wire.send(s.outputStream, command); unwrap(Wire.receive(s.inputStream)) }
 }
 suspend fun backend(): Backend = withContext(Dispatchers.IO) {
  val kernel = runCatching { unwrap(Native.status()) as JSONObject }
  val daemon = runCatching { rpc("status") as JSONObject }
  Backend(kernel.getOrNull(), daemon.getOrNull(), daemon.exceptionOrNull()?.message ?: kernel.exceptionOrNull()?.message)
 }
 suspend fun apps(): List<RootApp> {
  val a = rpc("apps") as JSONArray
  return (0 until a.length()).map { val x = a.getJSONObject(it); RootApp(x.getString("package"), x.getInt("uid"), x.getInt("permission"), x.getLong("capabilities"), x.getLong("lastRequest"), x.getLong("lastGrant")) }
 }
 suspend fun modules(): List<Module> {
  val a = rpc("modules") as JSONArray
  return (0 until a.length()).map { val x = a.getJSONObject(it); Module(x.getString("id"), x.getString("name"), x.getString("version"), x.getString("author"), x.getString("description"), x.getBoolean("enabled"), x.getBoolean("remove"), x.getBoolean("rebootRequired")) }
 }
 suspend fun permission(app: RootApp, permission: Int, caps: Long = if (app.caps != 0L) app.caps else FULL_CAPS) = rpc("permission\n${app.uid}\n$permission\n$caps")
 suspend fun module(id: String, action: String) = rpc("module\n$action\n$id")
 suspend fun upload(uri: Uri, progress: (Float, String) -> Unit): Any = withContext(Dispatchers.IO) {
  val tmp = java.io.File.createTempFile("module-", ".zip", context.cacheDir)
  try {
   context.contentResolver.openInputStream(uri)!!.use { input -> tmp.outputStream().use { out ->
    val buffer = ByteArray(65536); var total = 0L
    while (true) { val n = input.read(buffer); if (n < 0) break; total += n; require(total <= 256L * 1024 * 1024) { "Module ZIP exceeds 256 MiB" }; out.write(buffer, 0, n) }
   } }
   socket().use { s ->
    Wire.send(s.outputStream, "install\n${tmp.length()}")
    val ready = JSONObject(Wire.receive(s.inputStream)); check(ready.optBoolean("ready")) { ready.optString("error", "Upload rejected") }
    tmp.inputStream().use { input -> val bytes = ByteArray(65536); var sent = 0L
     while (true) { val n = input.read(bytes); if (n < 0) break; s.outputStream.write(bytes, 0, n); sent += n; progress(sent.toFloat() / tmp.length(), "Uploading module") }
    }
    s.outputStream.flush(); progress(1f, "Validating ZIP and running installer")
    unwrap(Wire.receive(s.inputStream))
   }
  } finally { tmp.delete() }
 }
}

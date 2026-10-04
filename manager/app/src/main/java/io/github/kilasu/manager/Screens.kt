// SPDX-License-Identifier: GPL-3.0-only
package io.github.kilasu.manager

import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.SystemClock
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.core.graphics.drawable.toBitmap
import kotlinx.coroutines.launch

@Composable fun appLabel(pkg: String): String {
 val pm = LocalContext.current.packageManager
 return remember(pkg) { runCatching { pm.getApplicationLabel(pm.getApplicationInfo(pkg, 0)).toString() }.getOrDefault(pkg) }
}
@Composable fun Heading(title: String, subtitle: String = "") {
 Column(Modifier.fillMaxWidth().padding(top = 12.dp, bottom = 8.dp), verticalArrangement = Arrangement.spacedBy(5.dp)) {
  Text(title, style = MaterialTheme.typography.headlineLarge)
  if (subtitle.isNotEmpty()) Text(subtitle, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
 }
}
@Composable fun Info(label: String, value: String) {
 Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
  Text(label, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.weight(.44f))
  Text(value, style = MaterialTheme.typography.bodyMedium, modifier = Modifier.weight(.56f))
 }
}
@Composable fun HomeScreen(b: Backend, checking: Boolean, count: Int, patch: () -> Unit, refresh: () -> Unit) {
 LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(22.dp), verticalArrangement = Arrangement.spacedBy(18.dp)) {
  item { Row(verticalAlignment = Alignment.CenterVertically) { KilaLogo(); Spacer(Modifier.width(14.dp)); Column { Text("KilaSU", style = MaterialTheme.typography.headlineLarge); Text("Kernel Root Framework", style = MaterialTheme.typography.bodyMedium) } } }
  item { GlassPanel(Modifier.fillMaxWidth()) {
   val title = when { checking -> "Checking backend"; b.operational -> "Working"; b.rootReady -> "Daemon Unavailable"; b.installed -> "Policy Required"; else -> "Not Installed" }
   Text(title, style = MaterialTheme.typography.headlineMedium)
   Text(when { checking -> "Reading the KilaSU kernel interface."; b.operational -> "Kernel Backend Connected"; b.installed -> "${b.error ?: "Enforcing SELinux integration is required."}"; else -> "KilaSU kernel backend was not detected." }, color = MaterialTheme.colorScheme.onSurfaceVariant)
   if (!checking && !b.installed) Button(onClick = patch) { Text("Patch boot.img") }
   if (b.installed) Info("Kernel", b.kernel!!.optString("kernel"))
   if (b.kernel != null) {
    Info("KilaSU Kernel", kernelVersion(b.kernel.getInt("kernelVersion")))
    Info("API", "v${b.kernel.getInt("api")}")
    if (b.kernel.getInt("api") != KilaApi.API) Text("Compatibility warning: Manager API ${KilaApi.API}", color = MaterialTheme.colorScheme.error)
   }
  } }
  item { GlassPanel(Modifier.fillMaxWidth()) {
   Text("Device", style = MaterialTheme.typography.titleLarge)
   Info("Android", "${Build.VERSION.RELEASE} · SDK ${Build.VERSION.SDK_INT}")
   Info("Architecture", Build.SUPPORTED_ABIS.firstOrNull() ?: "Unknown")
   Info("Device", "${Build.MANUFACTURER} ${Build.MODEL}")
   Info("Build", Build.DISPLAY)
   Info("Uptime", "${SystemClock.elapsedRealtime() / 3600000}h ${(SystemClock.elapsedRealtime() / 60000) % 60}m")
   Info("Manager", "v${BuildConfig.VERSION_NAME}")
  } }
  item { GlassPanel(Modifier.fillMaxWidth()) {
   Text("Runtime", style = MaterialTheme.typography.titleLarge)
   Info("Daemon", b.daemon?.let { "Running · v${it.optString("daemon")}" } ?: "Unavailable")
   Info("SELinux", b.daemon?.optString("selinux") ?: readSELinux())
   Info("Modules", b.daemon?.optInt("modules")?.toString() ?: "Unavailable")
   Info("Root apps", b.daemon?.optInt("rootApps")?.toString() ?: "Unavailable")
   TextButton(onClick = refresh) { Text("Refresh") }
  } }
 }
}
fun kernelVersion(v: Int) = "v${v ushr 16}.${(v ushr 8) and 255}.${v and 255}"
fun readSELinux() = runCatching { if (java.io.File("/sys/fs/selinux/enforce").readText().trim() == "1") "Enforcing" else "Permissive" }.getOrDefault("Unavailable")
@Composable fun SuperuserScreen(apps: List<RootApp>, connected: Boolean, change: (RootApp, Int, Long) -> Unit) {
 var search by remember { mutableStateOf("") }
 var profile by remember { mutableStateOf<RootApp?>(null) }
 val pm = LocalContext.current.packageManager
 LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(22.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
  item { Heading("Superuser", "Access is denied until explicitly approved.") }
  item { OutlinedTextField(search, { search = it }, label = { Text("Search applications") }, modifier = Modifier.fillMaxWidth(), singleLine = true) }
  if (!connected) item { GlassPanel { Text("Connect a verified KilaSU Manager to the kernel and daemon to manage authorization.") } }
  items(apps.filter { it.pkg.contains(search, true) }, key = { "${it.uid}:${it.pkg}" }) { app ->
   GlassPanel(Modifier.fillMaxWidth()) {
    Row(verticalAlignment = Alignment.CenterVertically) {
     val icon = remember(app.pkg) { runCatching { pm.getApplicationIcon(app.pkg).toBitmap(96, 96).asImageBitmap() }.getOrNull() }
     icon?.let { Image(it, app.pkg, Modifier.size(40.dp)); Spacer(Modifier.width(12.dp)) }
     Column(Modifier.weight(1f)) { Text(appLabel(app.pkg), style = MaterialTheme.typography.titleMedium); Text(app.pkg, style = MaterialTheme.typography.bodySmall); Text("UID ${app.uid}", style = MaterialTheme.typography.labelSmall) }
     Switch(app.permission != 0, { change(app, if (it) 1 else 0, if (app.caps > 0) app.caps else KilaApi.FULL_CAPS) })
    }
    Info("Last root use", elapsed(app.lastGrant))
    TextButton(onClick = { profile = app }) { Text("App Profile & History") }
   }
  }
 }
 profile?.let { app ->
  var caps by remember(app.uid) { mutableStateOf((if (app.caps > 0) app.caps else KilaApi.FULL_CAPS).toString()) }
  AlertDialog(onDismissRequest = { profile = null }, title = { Text("App Profile") }, text = { Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
   Text("${app.pkg}\nUID ${app.uid}")
   Info("Access", if (app.permission == 0) "Denied" else if (app.permission == 2) "Allow once" else "Allowed")
   Info("Last request", elapsed(app.lastRequest)); Info("Last use", elapsed(app.lastGrant))
   OutlinedTextField(caps, { caps = it }, label = { Text("Linux capability bitmask") }, singleLine = true)
   Text("A root shell uses a private mount namespace. Custom environment and per-module visibility are not exposed by API v1.", style = MaterialTheme.typography.bodySmall)
  } }, confirmButton = { TextButton(onClick = { val mask = caps.toLongOrNull(); if (mask != null && mask in 0..KilaApi.FULL_CAPS) { profile = null; change(app, app.permission, mask) } }, enabled = caps.toLongOrNull()?.let { it in 0..KilaApi.FULL_CAPS } == true) { Text("Save Profile") } }, dismissButton = { TextButton(onClick = { profile = null; change(app, 0, app.caps) }) { Text("Revoke") } })
 }
}
fun elapsed(nanos: Long): String = if (nanos == 0L) "Never this boot" else "${((SystemClock.elapsedRealtimeNanos() - nanos).coerceAtLeast(0)) / 1000000000}s ago"
@Composable fun ModulesScreen(modules: List<Module>, connected: Boolean, api: KilaApi, perform: (suspend () -> Unit) -> Unit) {
 var progress by remember { mutableFloatStateOf(0f) }; var state by remember { mutableStateOf("") }; var busy by remember { mutableStateOf(false) }
 val picker = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri -> if (uri != null) perform {
  busy = true; state = "Preparing module ZIP"; try { api.upload(uri) { p, text -> progress = p; state = text }; state = "Installation complete. Reboot required." } finally { busy = false }
 } }
 LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(22.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
  item { Heading("Modules", "Systemless changes are activated at boot.") }
  item { Button(onClick = { picker.launch(arrayOf("application/zip", "application/octet-stream")) }, enabled = connected && !busy) { Text("Install ZIP") } }
  if (state.isNotEmpty()) item { GlassPanel(Modifier.fillMaxWidth()) { Text(state); if (busy) LinearProgressIndicator(progress = { progress }, modifier = Modifier.fillMaxWidth()) } }
  if (!connected) item { Text("Module engine unavailable. The daemon must be connected.") }
  else if (modules.isEmpty()) item { GlassPanel { Text("No modules installed.") } }
  items(modules, key = { it.id }) { m -> GlassPanel(Modifier.fillMaxWidth()) {
   Row(verticalAlignment = Alignment.CenterVertically) { Text(m.name, style = MaterialTheme.typography.titleLarge, modifier = Modifier.weight(1f)); Switch(m.enabled && !m.remove, { enable -> perform { api.module(m.id, if (enable) "enable" else "disable") } }, enabled = !busy && !m.remove) }
   Text("${m.version} · ${m.author}", style = MaterialTheme.typography.labelLarge); Text(m.description)
   if (m.reboot) Text("Reboot required", color = MaterialTheme.colorScheme.primary)
   if (m.remove) Text("Removal queued") else TextButton(onClick = { perform { api.module(m.id, "remove") } }, enabled = !busy) { Text("Remove") }
  } }
 }
}
@Composable fun SettingsScreen(b: Backend, dark: Boolean, dynamic: Boolean, glass: Float, animation: Boolean, prompts: Boolean,
 onDark: (Boolean) -> Unit, onDynamic: (Boolean) -> Unit, onGlass: (Float) -> Unit, onAnimation: (Boolean) -> Unit, onPrompts: (Boolean) -> Unit, diagnostics: () -> Unit, patch: () -> Unit) {
 val context = LocalContext.current
 fun open(path: String) { context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse("https://github.com/takamoresishei/KilaSu/$path"))) }
 LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(22.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
  item { Heading("Settings") }
  item { GlassPanel(Modifier.fillMaxWidth()) { Text("Appearance", style = MaterialTheme.typography.titleLarge)
   Toggle("Dark Mode", dark, onDark); Toggle("Dynamic Color", dynamic, onDynamic); Toggle("Animation", animation, onAnimation)
   Text("Glass intensity · ${(glass * 100).toInt()}%"); Slider(glass, onGlass, valueRange = 0f..1f)
  } }
  item { GlassPanel(Modifier.fillMaxWidth()) { Text("KilaSU", style = MaterialTheme.typography.titleLarge)
   Info("Manager", "v${BuildConfig.VERSION_NAME}"); Info("Kernel", b.kernel?.let { kernelVersion(it.getInt("kernelVersion")) } ?: "Unavailable")
   Info("API", b.kernel?.optInt("api")?.toString() ?: "Unavailable"); Info("Daemon", b.daemon?.optString("daemon") ?: "Unavailable")
   Info("Default Access", "Deny"); Toggle("Superuser dialogs", prompts, onPrompts)
   TextButton(onClick = patch) { Text("Patch boot.img") }; TextButton(onClick = diagnostics) { Text("Diagnostics & Export Logs") }
  } }
  item { GlassPanel(Modifier.fillMaxWidth()) { Text("About", style = MaterialTheme.typography.titleLarge)
   Text("Independent kernel, UAPI and userspace implementation. Manager GPL-3.0; kernel GPL-2.0.")
   TextButton(onClick = { open("") }) { Text("GitHub") }; TextButton(onClick = { open("blob/main/THIRD_PARTY_NOTICES.md") }) { Text("Licenses") }; TextButton(onClick = { open("graphs/contributors") }) { Text("Contributors") }
  } }
 }
}
@Composable fun Toggle(label: String, value: Boolean, change: (Boolean) -> Unit) { Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) { Text(label, Modifier.weight(1f)); Switch(value, change) } }
@Composable fun DiagnosticsScreen(api: KilaApi, b: Backend, back: () -> Unit) {
 var text by remember { mutableStateOf("") }; var logs by remember { mutableStateOf("") }; val scope = rememberCoroutineScope(); val context = LocalContext.current
 val export = rememberLauncherForActivityResult(ActivityResultContracts.CreateDocument("text/plain")) { uri -> if (uri != null) scope.launch { runCatching { kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.IO) { context.contentResolver.openOutputStream(uri, "w")!!.use { it.write((text + "\n" + logs).toByteArray()) } } } } }
 LaunchedEffect(b) { text = "Manager ${BuildConfig.VERSION_NAME}\nKernel backend: ${b.kernel ?: "Unavailable"}\nDaemon: ${b.daemon ?: "Unavailable"}\nSELinux: ${b.daemon?.optString("selinux") ?: readSELinux()}\nError: ${b.error ?: "None"}" }
 LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(22.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
  item { Heading("Diagnostics"); TextButton(onClick = back) { Text("Back") } }
  item { GlassPanel(Modifier.fillMaxWidth()) { Text(text, style = MaterialTheme.typography.bodySmall) } }
  item { Button(onClick = { scope.launch { logs = runCatching { api.rpc("logs").toString() }.getOrElse { it.message ?: "Logs unavailable" } } }) { Text("Read daemon logs") } }
  if (logs.isNotEmpty()) item { GlassPanel { Text(logs, style = MaterialTheme.typography.bodySmall) } }
  item { Button(onClick = { export.launch("KilaSU-diagnostics.txt") }) { Text("Export diagnostic report") } }
 }
}

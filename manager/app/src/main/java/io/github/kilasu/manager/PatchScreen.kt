// SPDX-License-Identifier: GPL-3.0-only
package io.github.kilasu.manager

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch

@Composable fun PatchScreen(back: () -> Unit) {
 val context = LocalContext.current; val patcher = remember { BootPatcher(context) }; val scope = rememberCoroutineScope()
 var source by remember { mutableStateOf<BootPatcher.Source?>(null) }; var payload by remember { mutableStateOf<BootPatcher.Payload?>(null) }; var result by remember { mutableStateOf<BootPatcher.Result?>(null) }
 var unsigned by remember { mutableStateOf(false) }; var busy by remember { mutableStateOf(false) }; var log by remember { mutableStateOf("") }
 fun run(action: suspend () -> Unit) { scope.launch { busy = true; try { action() } catch (e: Exception) { log = e.message ?: "Patch failed" } finally { busy = false } } }
 val bootPicker = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri -> if (uri != null) run { source?.file?.delete(); source = patcher.source(uri); result = null; log = "Source boot image analyzed" } }
 val payloadPicker = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri -> if (uri != null) run { payload?.image?.delete(); payload = patcher.payload(uri); result = null; log = "KilaSU payload checksums verified" } }
 val imageExport = rememberLauncherForActivityResult(ActivityResultContracts.CreateDocument("application/octet-stream")) { uri -> if (uri != null) result?.let { r -> run { patcher.export(r.file, uri); log = "Patched image exported" } } }
 val bundleExport = rememberLauncherForActivityResult(ActivityResultContracts.CreateDocument("application/zip")) { uri -> if (uri != null) result?.let { r -> run { patcher.bundle(r, uri); log = "Recovery patch bundle exported. Flash the official installer in recovery." } } }
 LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(22.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
  item { Heading("Boot Patcher", "Patch & export. Installation is performed in recovery."); TextButton(onClick = back) { Text("Back") } }
  item { GlassPanel(Modifier.fillMaxWidth()) {
   Text("1 · Source image", style = MaterialTheme.typography.titleLarge); Button(onClick = { bootPicker.launch(arrayOf("application/octet-stream", "*/*")) }, enabled = !busy) { Text("Select boot.img") }
   source?.let { s -> Text(s.name, style = MaterialTheme.typography.bodySmall); Info("Header", "v${s.info.getInt("headerVersion")}"); Info("Kernel", s.info.getString("kernel")); Info("Size", "${s.file.length() / 1024 / 1024} MiB") }
  } }
  item { GlassPanel(Modifier.fillMaxWidth()) {
   Text("2 · Kernel payload", style = MaterialTheme.typography.titleLarge)
   Text("Use a device payload compiled with CONFIG_KILASU=y. The ROM must include enforcing KilaSU SELinux policy and init lifecycle integration.", style = MaterialTheme.typography.bodySmall)
   Button(onClick = { payloadPicker.launch(arrayOf("application/zip", "application/octet-stream")) }, enabled = !busy) { Text("Select KilaSU payload") }
   payload?.let { Info("Device", it.properties.getValue("device")); Info("API", it.properties.getValue("api")) }
  } }
  item { GlassPanel(Modifier.fillMaxWidth()) {
   Text("3 · Patch", style = MaterialTheme.typography.titleLarge)
   Row(verticalAlignment = Alignment.CenterVertically) { Checkbox(unsigned, { unsigned = it }); Text("Unsigned export for an unlocked bootloader", style = MaterialTheme.typography.bodySmall) }
   Text("Signed boot containers require explicit unsigned export or an external device signing workflow.", style = MaterialTheme.typography.bodySmall)
   Button(onClick = { val s = source; val p = payload; if (s != null && p != null) run { log = "Patching and validating output"; result = patcher.patch(s, p, unsigned); log = result!!.log } }, enabled = !busy && source != null && payload != null) { Text("Patch boot.img") }
   if (busy) LinearProgressIndicator(Modifier.fillMaxWidth())
   if (log.isNotEmpty()) Text(log, style = MaterialTheme.typography.bodySmall)
  } }
  result?.let { r -> item { GlassPanel(Modifier.fillMaxWidth()) {
   Text("Validated output", style = MaterialTheme.typography.titleLarge); Info("Patch version", "1"); Info("KilaSU", BuildConfig.VERSION_NAME)
   Text("SHA-256\n${r.info.getString("checksum")}", style = MaterialTheme.typography.bodySmall)
   Button(onClick = { imageExport.launch("KilaSU-patched-${r.info.getString("kernel")}.img") }, enabled = !busy) { Text("Export patched image") }
   Button(onClick = { bundleExport.launch("KilaSU-patch.zip") }, enabled = !busy) { Text("Export recovery bundle") }
  } } }
 }
}

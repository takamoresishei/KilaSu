// SPDX-License-Identifier: GPL-3.0-only
package io.github.kilasu.manager

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import dev.chrisbanes.haze.HazeState
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import org.json.JSONArray

class MainActivity : ComponentActivity() {
 override fun onCreate(savedInstanceState: Bundle?) {
  super.onCreate(savedInstanceState); enableEdgeToEdge()
  setContent { KilaApp() }
 }
}
@Composable fun KilaApp() {
 val context = LocalContext.current
 val api = remember { KilaApi(context) }
 val prefs = remember { context.getSharedPreferences("appearance", 0) }
 var dark by remember { mutableStateOf(prefs.getBoolean("dark", true)) }
 var dynamic by remember { mutableStateOf(prefs.getBoolean("dynamic", false)) }
 var intensity by remember { mutableFloatStateOf(prefs.getFloat("glass", .7f)) }
 var animation by remember { mutableStateOf(prefs.getBoolean("animation", true)) }
 var prompts by remember { mutableStateOf(prefs.getBoolean("prompts", true)) }
 var opened by rememberSaveable { mutableStateOf(false) }
 var tab by rememberSaveable { mutableStateOf("Home") }
 var backend by remember { mutableStateOf(Backend()) }
 var checking by remember { mutableStateOf(true) }
 var apps by remember { mutableStateOf(emptyList<RootApp>()) }
 var modules by remember { mutableStateOf(emptyList<Module>()) }
 var request by remember { mutableStateOf<RootApp?>(null) }
 var lastSequence by remember { mutableLongStateOf(0L) }
 val haze = remember { HazeState() }
 val scope = rememberCoroutineScope()
 val snackbar = remember { SnackbarHostState() }
 val perform: (suspend () -> Unit) -> Unit = { action -> scope.launch { try { action() } catch (e: Exception) { snackbar.showSnackbar(e.message ?: "Operation failed") } } }
 suspend fun refresh() {
  backend = api.backend(); checking = false
  if (backend.daemon != null) {
   runCatching { api.apps() }.onSuccess { apps = it }
   runCatching { api.modules() }.onSuccess { modules = it }
   if (prompts) runCatching {
    val events = api.rpc("audit\n$lastSequence") as JSONArray
    for (i in 0 until events.length()) {
     val e = events.getJSONObject(i); val seq = e.getLong("sequence")
     if (seq >= lastSequence) { lastSequence = seq + 1
      if (request == null && e.getInt("command") == 9 && e.getInt("result") == -13) request = apps.find { it.uid == e.getInt("uid") }
     }
    }
   }
  } else { apps = emptyList(); modules = emptyList() }
 }
 LaunchedEffect(Unit) { while (true) { refresh(); delay(5000) } }
 LaunchedEffect(animation) { if (!opened) { if (animation) delay(1100); opened = true } }
 KilaTheme(dynamic, dark) {
  CompositionLocalProvider(LocalGlass provides haze, LocalIntensity provides intensity) {
   Box(Modifier.fillMaxSize()) {
    GlassBackground(haze, animation)
    AnimatedVisibility(opened, enter = fadeIn(tween(350)) + scaleIn(initialScale = .98f), exit = fadeOut()) {
     Scaffold(containerColor = Color.Transparent, snackbarHost = { SnackbarHost(snackbar) }, bottomBar = {
      GlassPanel(Modifier.padding(horizontal = 14.dp).navigationBarsPadding().padding(bottom = 8.dp)) {
       Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceEvenly) {
        val icons = listOf(Icons.Outlined.Home, Icons.Outlined.Shield, Icons.Outlined.Extension, Icons.Outlined.Settings)
        listOf("Home", "Superuser", "Modules", "Settings").forEachIndexed { i, name ->
         TextButton(onClick = { tab = name }, contentPadding = PaddingValues(4.dp)) {
          Column(horizontalAlignment = Alignment.CenterHorizontally) {
           Icon(icons[i], contentDescription = name, tint = if (tab == name) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant)
           Text(name, style = MaterialTheme.typography.labelSmall)
          }
         }
        }
       }
      }
     }) { padding ->
      Box(Modifier.padding(padding).fillMaxSize()) {
       when (tab) {
        "Home" -> HomeScreen(backend, checking, modules.size, { tab = "Patch" }, { perform { refresh() } })
        "Superuser" -> SuperuserScreen(apps, backend.daemon != null) { app, access, caps -> perform { api.permission(app, access, caps); refresh() } }
        "Modules" -> ModulesScreen(modules, backend.daemon != null, api, { action -> perform { action(); refresh() } })
        "Patch" -> PatchScreen { tab = "Home" }
        "Diagnostics" -> DiagnosticsScreen(api, backend) { tab = "Settings" }
        else -> SettingsScreen(backend, dark, dynamic, intensity, animation, prompts,
         { dark = it; prefs.edit().putBoolean("dark", it).apply() },
         { dynamic = it; prefs.edit().putBoolean("dynamic", it).apply() },
         { intensity = it; prefs.edit().putFloat("glass", it).apply() },
         { animation = it; prefs.edit().putBoolean("animation", it).apply() },
         { prompts = it; prefs.edit().putBoolean("prompts", it).apply() },
         { tab = "Diagnostics" }, { tab = "Patch" })
       }
      }
     }
    }
    AnimatedVisibility(!opened, exit = fadeOut(tween(300))) {
     val alpha by animateFloatAsState(if (opened) 0f else 1f, tween(700), label = "logo-opacity")
     val scale by animateFloatAsState(if (opened) 1.05f else 1f, spring(), label = "logo-scale")
     Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
      Column(Modifier.graphicsLayer { alpha.let { this.alpha = it }; scaleX = scale; scaleY = scale }, horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(16.dp)) {
       KilaLogo(Modifier.size(82.dp)); Text("KilaSU", style = MaterialTheme.typography.headlineLarge); Text("Kernel Root Framework", style = MaterialTheme.typography.labelLarge)
      }
     }
    }
    request?.let { app ->
     AlertDialog(onDismissRequest = { request = null }, title = { Text("Superuser request") }, text = { Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
      Text("${appLabel(app.pkg)} is requesting Superuser access."); Text("${app.pkg}\nUID ${app.uid}", style = MaterialTheme.typography.bodySmall)
     } }, confirmButton = { Column {
      TextButton(onClick = { request = null; perform { api.permission(app, 2); refresh() } }) { Text("Allow once") }
      TextButton(onClick = { request = null; perform { api.permission(app, 1); refresh() } }) { Text("Allow permanently") }
     } }, dismissButton = { TextButton(onClick = { request = null; perform { api.permission(app, 0); refresh() } }) { Text("Deny") } })
    }
   }
  }
 }
}

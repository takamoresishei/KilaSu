// SPDX-License-Identifier: GPL-3.0-only
package io.github.kilasu.manager

import androidx.compose.animation.core.*
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.border
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.*
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import dev.chrisbanes.haze.*

val LocalGlass = staticCompositionLocalOf { HazeState() }
val LocalIntensity = staticCompositionLocalOf { 0.7f }
@Composable fun KilaTheme(dynamic: Boolean, dark: Boolean, content: @Composable () -> Unit) {
 val context = LocalContext.current
 val colors = if (dynamic) { if (dark) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context) }
 else if (dark) darkColorScheme(primary = Color(0xFFB6D1DE), background = Color(0xFF0C1017), surface = Color(0xFF151D28))
 else lightColorScheme(primary = Color(0xFF476776), background = Color(0xFFF0F3F7), surface = Color(0xFFE5ECF1))
 MaterialTheme(colorScheme = colors) {
  CompositionLocalProvider(LocalContentColor provides colors.onBackground, content = content)
 }
}
@Composable fun GlassBackground(haze: HazeState, animation: Boolean) {
 val transition = rememberInfiniteTransition(label = "ambient")
 val phase by transition.animateFloat(0f, 1f, infiniteRepeatable(tween(18000, easing = LinearEasing), RepeatMode.Reverse), label = "drift")
 val dark = MaterialTheme.colorScheme.background.luminance() < 0.5f
 val base = MaterialTheme.colorScheme.background
 val accent = MaterialTheme.colorScheme.primary.copy(alpha = if (dark) .20f else .13f)
 Canvas(Modifier.fillMaxSize().hazeSource(haze)) {
  drawRect(base)
  val p = if (animation) phase else .4f
  drawCircle(Brush.radialGradient(listOf(accent, Color.Transparent), Offset(size.width * (.2f + p * .15f), size.height * .26f), size.width * .85f), size.width * .85f, Offset(size.width * (.2f + p * .15f), size.height * .26f))
  drawCircle(Brush.radialGradient(listOf(Color(0xFF8B99B2).copy(alpha = .16f), Color.Transparent), Offset(size.width * .85f, size.height * (.7f - p * .12f)), size.width), size.width, Offset(size.width * .85f, size.height * (.7f - p * .12f)))
  drawLine(Color.White.copy(alpha = .09f), Offset(-size.width * .1f, size.height * .32f), Offset(size.width, size.height * .07f), 55.dp.toPx())
  drawLine(accent.copy(alpha = .18f), Offset(-size.width * .1f, size.height * .73f), Offset(size.width, size.height * .5f), 30.dp.toPx())
 }
}
@Composable fun GlassPanel(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
 val shape = RoundedCornerShape(26.dp)
 val intensity = LocalIntensity.current
 val dark = MaterialTheme.colorScheme.background.luminance() < .5f
 val tint = if (dark) Color(0xFF23313F).copy(alpha = .20f + .25f * intensity) else Color.White.copy(alpha = .25f + .30f * intensity)
 Column(modifier.shadow(14.dp, shape, ambientColor = Color.Black.copy(alpha = .14f)).clip(shape)
  .hazeEffect(LocalGlass.current, HazeStyle(backgroundColor = MaterialTheme.colorScheme.background, tint = HazeTint(tint), blurRadius = (10 + 32 * intensity).dp, noiseFactor = .025f))
  .border(1.dp, Brush.linearGradient(listOf(Color.White.copy(alpha = .28f), Color.White.copy(alpha = .04f))), shape)
  .padding(20.dp), verticalArrangement = Arrangement.spacedBy(12.dp), content = content)
}
@Composable fun KilaLogo(modifier: Modifier = Modifier) {
 val color = MaterialTheme.colorScheme.onBackground
 Canvas(modifier.size(58.dp)) {
  val p = Path().apply { moveTo(size.width / 2, 2.dp.toPx()); lineTo(size.width - 2.dp.toPx(), size.height / 2); lineTo(size.width / 2, size.height - 2.dp.toPx()); lineTo(2.dp.toPx(), size.height / 2); close() }
  drawPath(p, color.copy(alpha = .08f)); drawPath(p, color.copy(alpha = .85f), style = androidx.compose.ui.graphics.drawscope.Stroke(1.2.dp.toPx()))
  val k = Path().apply { moveTo(size.width * .35f, size.height * .29f); lineTo(size.width * .35f, size.height * .7f); moveTo(size.width * .66f, size.height * .29f); lineTo(size.width * .38f, size.height * .52f); lineTo(size.width * .67f, size.height * .7f) }
  drawPath(k, color, style = androidx.compose.ui.graphics.drawscope.Stroke(2.2.dp.toPx(), cap = StrokeCap.Round))
 }
}

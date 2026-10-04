// SPDX-License-Identifier: GPL-3.0-only
package io.github.kilasu.manager
import androidx.compose.ui.test.*
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.UiDevice
import java.io.File
import org.junit.Rule
import org.junit.Test
class HomeScreenshotTest {
 @get:Rule val compose = createAndroidComposeRule<MainActivity>()
 @Test fun realDisconnectedState() {
  compose.waitUntil(15000) { compose.onAllNodesWithText("Not Installed").fetchSemanticsNodes().isNotEmpty() }
  compose.onNodeWithText("Not Installed").assertIsDisplayed()
  val instrumentation = InstrumentationRegistry.getInstrumentation()
  UiDevice.getInstance(instrumentation).takeScreenshot(File(instrumentation.targetContext.getExternalFilesDir(null), "manager-home.png"))
 }
}

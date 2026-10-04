// SPDX-License-Identifier: GPL-3.0-only
package io.github.kilasu.manager
import androidx.test.core.app.ActivityScenario
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.By
import androidx.test.uiautomator.Until
import java.io.File
import org.junit.Test
import org.junit.Assert.assertTrue
import org.junit.runner.RunWith
@RunWith(AndroidJUnit4::class)
class HomeScreenshotTest {
 @Test fun realDisconnectedState() {
  val instrumentation = InstrumentationRegistry.getInstrumentation()
  val device = UiDevice.getInstance(instrumentation)
  // Use the production frame clock for screenshots of the completed reveal.
  ActivityScenario.launch(MainActivity::class.java).use {
   assertTrue("Disconnected state was not shown", device.wait(Until.hasObject(By.text("Not Installed")), 15000))
   assertTrue("Navigation was not shown", device.wait(Until.hasObject(By.text("Home")), 15000))
   device.waitForIdle(1500)
   assertTrue("Screenshot capture failed", device.takeScreenshot(File(instrumentation.targetContext.getExternalFilesDir(null), "manager-home.png")))
  }
 }
}

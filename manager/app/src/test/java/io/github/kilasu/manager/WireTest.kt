// SPDX-License-Identifier: GPL-3.0-only
package io.github.kilasu.manager
import org.junit.Assert.*
import org.junit.Test
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
class WireTest {
 @Test fun roundTrip() { val output = ByteArrayOutputStream(); Wire.send(output, "status"); assertEquals("status", Wire.receive(ByteArrayInputStream(output.toByteArray()))) }
 @Test fun rejectsLargeFrame() { try { Wire.receive(ByteArrayInputStream(byteArrayOf(-1,-1,-1,-1))); fail("accepted oversized frame") } catch (_: IllegalArgumentException) { } }
 @Test fun truncatedHeader() { try { Wire.receive(ByteArrayInputStream(byteArrayOf(1,0))); fail("accepted truncated header") } catch (_: IllegalStateException) { } }
 @Test fun payloadProperties() { assertEquals("a", BootPatcher.parseProperties("device=a\napi=1")["device"]); try { BootPatcher.parseProperties("api=1\napi=2"); fail("accepted duplicate key") } catch (_: IllegalArgumentException) { } }
}

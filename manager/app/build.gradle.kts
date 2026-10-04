plugins {
 id("com.android.application")
 id("org.jetbrains.kotlin.android")
 id("org.jetbrains.kotlin.plugin.compose")
}
android {
 namespace = "io.github.kilasu.manager"
 compileSdk = 35
 ndkVersion = "27.2.12479018"
 defaultConfig {
  applicationId = "io.github.kilasu.manager"
  minSdk = 31
  targetSdk = 35
  versionCode = 1
  versionName = "0.1.0"
  testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
  ndk { abiFilters += listOf("arm64-v8a", "x86_64") }
  externalNativeBuild { cmake { cppFlags += "-std=c++17" } }
 }
 signingConfigs {
  create("release") {
   storeFile = System.getenv("KILASU_KEYSTORE")?.let { file(it) }
   storePassword = System.getenv("KILASU_STORE_PASSWORD")
   keyAlias = System.getenv("KILASU_KEY_ALIAS")
   keyPassword = System.getenv("KILASU_KEY_PASSWORD")
  }
 }
 buildTypes {
  release { isMinifyEnabled = false; signingConfig = signingConfigs.getByName("release") }
 }
 buildFeatures { compose = true; buildConfig = true }
 compileOptions { sourceCompatibility = JavaVersion.VERSION_17; targetCompatibility = JavaVersion.VERSION_17 }
 kotlinOptions { jvmTarget = "17" }
 externalNativeBuild { cmake { path = file("src/main/cpp/CMakeLists.txt"); version = "3.22.1" } }
 sourceSets.getByName("main").assets.srcDir(layout.buildDirectory.dir("generated/notices"))
}
val copyNotices by tasks.registering(Copy::class) {
 from(rootProject.file("../LICENSE"))
 from(rootProject.file("../THIRD_PARTY_NOTICES.md"))
 from(rootProject.file("../LICENSES"))
 into(layout.buildDirectory.dir("generated/notices/licenses"))
}
tasks.named("preBuild").configure { dependsOn(copyNotices) }
dependencies {
 implementation(platform("androidx.compose:compose-bom:2025.05.01"))
 implementation("androidx.activity:activity-compose:1.10.1")
 implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.9.0")
 implementation("androidx.compose.ui:ui")
 implementation("androidx.compose.ui:ui-tooling-preview")
 implementation("androidx.compose.material3:material3")
 implementation("androidx.compose.material:material-icons-extended")
 implementation("androidx.core:core-ktx:1.16.0")
 implementation("dev.chrisbanes.haze:haze:1.6.10")
 debugImplementation("androidx.compose.ui:ui-tooling")
 testImplementation("junit:junit:4.13.2")
 androidTestImplementation("androidx.test.ext:junit:1.2.1")
 androidTestImplementation("androidx.test:core:1.6.1")
 androidTestImplementation("androidx.test.uiautomator:uiautomator:2.3.0")
}

import org.jetbrains.compose.desktop.application.dsl.TargetFormat

plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.composeMultiplatform)
    alias(libs.plugins.composeCompiler)
    alias(libs.plugins.composeHotReload)
    alias(libs.plugins.kotlin.serialization)
}

kotlin {
    jvm()
    
    sourceSets {
        commonMain.dependencies {
            implementation(compose.runtime)
            implementation(compose.foundation)
            implementation(compose.material3)
            implementation(compose.ui)
            implementation(compose.components.resources)
            implementation(compose.components.uiToolingPreview)
            implementation(libs.androidx.lifecycle.viewmodelCompose)
            implementation(libs.androidx.lifecycle.runtimeCompose)
            implementation(libs.kotlinx.serialization.json)
        }

        commonTest.dependencies {
            implementation(libs.kotlin.test)
        }

        jvmMain.dependencies {
            implementation(compose.desktop.currentOs)
            implementation(libs.kotlinx.coroutinesSwing)
            implementation("net.java.dev.jna:jna:5.14.0")
            implementation("net.java.dev.jna:jna-platform:5.14.0")
        }
    }

    jvmToolchain(22)
}


compose.desktop {
    application {
        mainClass = "io.github.padconnect.receiver.MainKt"

        jvmArgs += listOf("--enable-native-access=ALL-UNNAMED")

        nativeDistributions {
            appResourcesRootDir.set(project.layout.projectDirectory.dir("src/jvmMain/resources"))
            targetFormats(TargetFormat.Msi, TargetFormat.Deb, TargetFormat.AppImage, TargetFormat.Rpm)
            packageName = "PadConnectReceiver"
            packageVersion = "0.4.0"

            windows {
                console = true
                iconFile.set(project.file("logos/icon.ico"))
                upgradeUuid = "90f3c261-723f-454b-acc6-82756b665976"
            }

            jvmArgs("--enable-native-access=ALL-UNNAMED")
        }

        buildTypes.release {
            proguard {
                isEnabled = false
            }
        }
    }
}

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "ru.game.menu"
    compileSdk = 35

    defaultConfig {
        applicationId = "ru.game.menu"
        minSdk = 29
        targetSdk = 35
        versionCode = 1
        versionName = "0.4.0"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }
}

// Зависимости не нужны: используется системный android.webkit.WebView.

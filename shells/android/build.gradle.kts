// Корневой build-скрипт оболочки Android.
// Сама игра — web-сборка Bevy (WASM) в assets/web; здесь только WebView-оболочка.
plugins {
    id("com.android.application") version "8.7.3" apply false
    id("org.jetbrains.kotlin.android") version "2.0.21" apply false
}

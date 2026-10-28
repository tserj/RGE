package ru.game.menu

import android.annotation.SuppressLint
import android.app.Activity
import android.os.Bundle
import android.webkit.WebView

/**
 * Нативная оболочка Android: лёгкий системный WebView с web-сборкой игры.
 *
 * Сама игра одна для всех платформ: Bevy компилируется в WebAssembly
 * (см. корневой README, раздел «Web»), а оболочки лишь показывают её
 * на весь экран. Логика не дублируется.
 */
class MainActivity : Activity() {

    private lateinit var web: WebView

    @SuppressLint("SetJavaScriptEnabled")
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        web = WebView(this)
        setContentView(web)

        web.settings.apply {
            javaScriptEnabled = true   // нужен для wasm-модуля и WebGL
            domStorageEnabled = true
            allowFileAccess = true
        }
        web.setBackgroundColor(0xFF000000.toInt())

        // Релиз: web-сборка лежит в assets/web (как её туда положить —
        // см. корневой README, раздел «Android»).
        web.loadUrl("file:///android_asset/web/index.html")

        // Dev-режим: запустить `trunk serve` в корне репозитория и раскомментировать
        // (10.0.2.2 — loopback хоста из эмулятора Android):
        // web.loadUrl("http://10.0.2.2:8080/")
    }

    @Deprecated("Deprecated in Java")
    override fun onBackPressed() {
        // Esc обрабатывается самим меню; системную «назад» глушим,
        // чтобы случайно не свернуть игру.
    }
}

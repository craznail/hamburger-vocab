package com.vocab.flashcards

import android.content.Context
import android.graphics.Color
import android.net.Uri
import android.os.Bundle
import android.provider.OpenableColumns
import android.speech.tts.TextToSpeech
import android.webkit.JavascriptInterface
import android.webkit.WebView
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.core.view.updatePadding
import java.util.Locale

class MainActivity : TauriActivity() {
  private var ttsBridge: TtsBridge? = null

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)

    // Keep drawing edge-to-edge, then apply the actual system and cutout
    // insets to the WebView below. This works on gesture/3-button navigation,
    // display cutouts, landscape, and Android's enforced edge-to-edge mode.
    WindowCompat.setDecorFitsSystemWindows(window, false)
    window.statusBarColor = Color.TRANSPARENT
    window.navigationBarColor = Color.TRANSPARENT
    WindowInsetsControllerCompat(window, window.decorView).apply {
      isAppearanceLightStatusBars = true
      isAppearanceLightNavigationBars = true
    }
  }

  override fun onWebViewCreate(webView: WebView) {
    super.onWebViewCreate(webView)
    ttsBridge = TtsBridge(this)
    webView.addJavascriptInterface(ttsBridge!!, "NativeTts")
    webView.addJavascriptInterface(FileResolver(this), "NativeFileResolver")

    ViewCompat.setOnApplyWindowInsetsListener(webView) { view, windowInsets ->
      val safeInsets = windowInsets.getInsets(
        WindowInsetsCompat.Type.systemBars() or
          WindowInsetsCompat.Type.displayCutout()
      )
      view.updatePadding(
        left = safeInsets.left,
        top = safeInsets.top,
        right = safeInsets.right,
        bottom = safeInsets.bottom
      )
      WindowInsetsCompat.CONSUMED
    }
    ViewCompat.requestApplyInsets(webView)
  }

  override fun onDestroy() {
    ttsBridge?.shutdown()
    ttsBridge = null
    super.onDestroy()
  }
}

private class TtsBridge(context: Context) {
  @Volatile
  private var ready = false
  private var engine: TextToSpeech? = null

  init {
    engine = TextToSpeech(context.applicationContext) { status ->
      ready = status == TextToSpeech.SUCCESS
      if (ready) {
        engine?.language = Locale.US
      }
    }
  }

  @JavascriptInterface
  fun isAvailable(): Boolean = ready

  @JavascriptInterface
  fun speak(text: String) {
    if (!ready || text.isBlank()) return
    engine?.speak(text, TextToSpeech.QUEUE_FLUSH, null, "vocab-${System.nanoTime()}")
  }

  @JavascriptInterface
  fun pause() {
    engine?.stop()
  }

  @JavascriptInterface
  fun isSpeaking(): Boolean = ready && engine?.isSpeaking == true

  fun shutdown() {
    ready = false
    engine?.stop()
    engine?.shutdown()
    engine = null
  }
}

private class FileResolver(context: Context) {
  private val contentResolver = context.applicationContext.contentResolver

  @JavascriptInterface
  fun getDisplayName(uriValue: String): String? {
    return runCatching {
      val uri = Uri.parse(uriValue)
      contentResolver.query(
        uri,
        arrayOf(OpenableColumns.DISPLAY_NAME),
        null,
        null,
        null
      )?.use { cursor ->
        val nameColumn = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
        if (nameColumn >= 0 && cursor.moveToFirst()) cursor.getString(nameColumn) else null
      }
    }.getOrNull()
  }
}

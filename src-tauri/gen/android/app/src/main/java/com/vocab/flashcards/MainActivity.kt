package com.vocab.flashcards

import android.graphics.Color
import android.os.Bundle
import android.webkit.WebView
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.core.view.updatePadding

class MainActivity : TauriActivity() {
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
    webView.addJavascriptInterface(TtsBridge(this), "NativeTts")
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
}

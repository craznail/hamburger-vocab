#!/bin/bash
# 单词卡片 - Android 开发调试脚本

export ANDROID_HOME=/usr/local/share/android-commandlinetools
export PATH="$PATH:$ANDROID_HOME/platform-tools"

case "${1:-help}" in
  emulator)
    echo "📱 启动安卓平板模拟器 (Tablet_API34)..."
    echo "   模拟器启动后约需 1-2 分钟加载完毕"
    echo "   加载完成后执行: ./dev-android.sh run"
    "$ANDROID_HOME/emulator/emulator" -avd Tablet_API34 -no-snapshot &
    ;;
  run)
    echo "🚀 编译并安装开发版 APK（带热重载）..."
    npm run tauri android dev
    ;;
  logcat)
    echo "📋 查看应用日志（Ctrl+C 退出）..."
    adb logcat | grep -E "(Tauri|console|Error|readTxtFile|com.vocab.flashcards)"
    ;;
  inspect)
    echo "🔍 Chrome DevTools 调试步骤:"
    echo "   1. 确保安卓设备/模拟器已连接 (adb devices)"
    echo "   2. 在 Chrome 地址栏打开: chrome://inspect"
    echo "   3. 点击单词卡片的 WebView 开始调试"
    ;;
  build)
    echo "🔨 构建正式 APK..."
    npm run tauri android build
    echo "   APK: src-tauri/gen/android/app/build/outputs/apk/universal/release/"
    ;;
  apk)
    echo "📦 APK 文件在:"
    ls -lh /Users/caogenyan/Documents/hamburger/发布包/单词卡片-v0.2.0.apk
    ;;
  *)
    echo "用法: ./dev-android.sh <命令>"
    echo ""
    echo "命令:"
    echo "  emulator   启动安卓平板模拟器"
    echo "  run        编译并安装开发版（热重载）"
    echo "  logcat     查看应用日志"
    echo "  inspect    打开 Chrome DevTools 调试指南"
    echo "  build      构建正式 APK"
    echo "  apk        查看发布 APK 信息"
    echo ""
    echo "首次调试流程:"
    echo "  步骤1: ./dev-android.sh emulator   # 在另一个终端中启动"
    echo "  步骤2: ./dev-android.sh run        # 编译并安装到模拟器"
    echo "  步骤3: ./dev-android.sh logcat     # 看日志定位问题"
    echo "  步骤4: Chrome 打开 chrome://inspect # Debug WebView"
    ;;
esac

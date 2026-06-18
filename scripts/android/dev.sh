#!/bin/bash
# 单词卡片 - Android 开发调试脚本

export ANDROID_HOME=${ANDROID_HOME:-/usr/local/share/android-commandlinetools}
export PATH="$HOME/.cargo/bin:$ANDROID_HOME/platform-tools:$PATH"
EMULATOR_BIN="$ANDROID_HOME/emulator/emulator"

# Android Rust targets are managed by rustup. Keep cargo and rustc on the
# same toolchain instead of accidentally mixing in a Homebrew installation.
if ! rustup which cargo >/dev/null 2>&1; then
  echo "❌ rustup toolchain 中缺少 Cargo，请执行: rustup component add cargo"
  exit 1
fi

if [ -z "$ANDROID_NDK_HOME" ]; then
  LATEST_NDK=$(ls -d "$ANDROID_HOME/ndk/"*/ 2>/dev/null | sort -V | tail -1)
  if [ -n "$LATEST_NDK" ]; then
    export ANDROID_NDK_HOME="${LATEST_NDK%/}"
    export NDK_HOME="$ANDROID_NDK_HOME"
  fi
fi

list_avds() {
  "$EMULATOR_BIN" -list-avds
}

avd_exists() {
  local target="$1"
  while IFS= read -r avd; do
    [ "$avd" = "$target" ] && return 0
  done < <(list_avds)

  return 1
}

classify_avd() {
  local name
  name=$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')

  case "$name" in
    *tablet*|*pixel_tablet*|*pixel_c*|*nexus_7*|*nexus_9*|*nexus_10*)
      printf 'tablet'
      ;;
    *fold*)
      printf 'foldable'
      ;;
    *)
      printf 'phone'
      ;;
  esac
}

choose_default_avd() {
  local avd

  while IFS= read -r avd; do
    [ "$(classify_avd "$avd")" = "phone" ] && {
      printf '%s\n' "$avd"
      return 0
    }
  done < <(list_avds)

  list_avds | head -n 1
}

resolve_avd() {
  local requested_avd="$1"
  local selected_avd="$requested_avd"
  local avds=()
  local avd

  if [ ! -x "$EMULATOR_BIN" ]; then
    echo "❌ 未找到 Android Emulator: $EMULATOR_BIN" >&2
    exit 1
  fi

  while IFS= read -r avd; do
    avds+=("$avd")
  done < <(list_avds)

  if [ "${#avds[@]}" -eq 0 ]; then
    echo "❌ 当前没有可用模拟器，请先用 avdmanager 创建 AVD" >&2
    exit 1
  fi

  if [ -n "$selected_avd" ]; then
    if ! avd_exists "$selected_avd"; then
      echo "❌ 未找到模拟器: $selected_avd" >&2
      echo "可用模拟器:" >&2
      printf '  - %s\n' "${avds[@]}" >&2
      exit 1
    fi
  else
    selected_avd=$(choose_default_avd)
  fi

  printf '%s\n' "$selected_avd"
}

start_emulator() {
  local requested_avd="$1"
  local selected_avd=""
  local default_avd=""
  local avds=()
  local selection=""
  local label=""
  local i=1

  while IFS= read -r avd; do
    avds+=("$avd")
  done < <(list_avds)

  if [ -z "$requested_avd" ]; then
    default_avd=$(choose_default_avd)

    echo "📱 可用模拟器:"
    for avd in "${avds[@]}"; do
      label=$(classify_avd "$avd")
      [ "$avd" = "$default_avd" ] && label="$label, default"
      echo "  $i. $avd [$label]"
      i=$((i + 1))
    done
    echo ""

    if [ -t 0 ]; then
      read -r -p "选择要启动的模拟器（输入序号或名称，回车默认 $default_avd）: " selection
    fi

    if [ -z "$selection" ]; then
      selected_avd="$default_avd"
    elif [[ "$selection" =~ ^[0-9]+$ ]] && [ "$selection" -ge 1 ] && [ "$selection" -le "${#avds[@]}" ]; then
      selected_avd="${avds[$((selection - 1))]}"
    else
      selected_avd="$selection"
    fi

    if ! avd_exists "$selected_avd"; then
      echo "❌ 无效选择: $selected_avd"
      exit 1
    fi
  else
    if ! selected_avd=$(resolve_avd "$requested_avd"); then
      exit 1
    fi
  fi

  echo "🚀 启动安卓模拟器: $selected_avd"
  echo "   模拟器启动后约需 1-2 分钟加载完毕"
  echo "   加载完成后执行: bash scripts/android/dev.sh run"
  "$EMULATOR_BIN" -avd "$selected_avd" -no-snapshot &
}

case "${1:-help}" in
  emulator)
    start_emulator "${2:-}"
    ;;
  run)
    if ! selected_avd=$(resolve_avd "${2:-}"); then
      exit 1
    fi
    echo "🚀 编译并安装开发版 APK（带热重载）..."
    echo "   目标设备: $selected_avd"
    export TAURI_DEV_HOST=10.0.2.2
    node scripts/tauri-dev.mjs android "$selected_avd"
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
    npx tauri android build
    echo "   APK: src-tauri/gen/android/app/build/outputs/apk/universal/release/"
    ;;
  apk)
    echo "📦 APK 文件在:"
    ls -lh src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
    ;;
  *)
    echo "用法: ./dev-android.sh <命令> [参数]"
    echo ""
    echo "命令:"
    echo "  emulator [AVD名称]  列出模拟器并启动选中的设备"
    echo "  run [AVD名称]      编译并安装开发版（默认优先手机模拟器）"
    echo "  logcat     查看应用日志"
    echo "  inspect    打开 Chrome DevTools 调试指南"
    echo "  build      构建正式 APK"
    echo "  apk        查看发布 APK 信息"
    echo ""
    echo "首次调试流程:"
    echo "  步骤1: ./dev-android.sh emulator              # 列出后选择模拟器"
    echo "          ./dev-android.sh emulator Pixel_8_API34  # 直接启动指定设备"
    echo "  步骤2: ./dev-android.sh run                  # 默认优先安装到手机模拟器"
    echo "          ./dev-android.sh run Pixel_8_API34   # 直接安装到指定设备"
    echo "  步骤3: ./dev-android.sh logcat     # 看日志定位问题"
    echo "  步骤4: Chrome 打开 chrome://inspect # Debug WebView"
    ;;
esac

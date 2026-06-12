param(
    [Parameter(Position=0)]
    [ValidateSet("emulator", "run", "logcat", "inspect", "build", "init", "help")]
    [string]$Command = "help"
)

$ANDROID_HOME = $env:ANDROID_HOME
if (-not $ANDROID_HOME) {
    $ANDROID_HOME = "D:\program\android\android-sdk-windows"
}

$NDK_HOME = $env:ANDROID_NDK_HOME
if (-not $NDK_HOME) {
    $ndkDirs = Get-ChildItem "$ANDROID_HOME\ndk" -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending
    if ($ndkDirs) {
        $NDK_HOME = $ndkDirs[0].FullName
    }
}

# Gradle/AGP 8.x requires Java 11+. Set JAVA_HOME to JDK 17 first
if (Test-Path "C:\Program Files\Java\jdk-17\bin\java.exe") {
    $env:JAVA_HOME = "C:\Program Files\Java\jdk-17"
    $env:Path = "C:\Program Files\Java\jdk-17\bin;$env:Path"
} elseif (Test-Path "C:\Program Files\Android\Android Studio\jbr\bin\java.exe") {
    $env:JAVA_HOME = "C:\Program Files\Android\Android Studio\jbr"
    $env:Path = "C:\Program Files\Android\Android Studio\jbr\bin;$env:Path"
}

$env:Path = "$env:USERPROFILE\.cargo\bin;$ANDROID_HOME\platform-tools;$ANDROID_HOME\emulator;$env:Path"
if ($NDK_HOME) {
    $env:ANDROID_NDK_HOME = $NDK_HOME
    $env:NDK_HOME = $NDK_HOME
}

# Android cross-compilation: configure NDK clang for cc-rs and cargo
# Must use clang.exe directly (NOT .cmd wrappers) - Rust cannot spawn .cmd files on Windows
if ($NDK_HOME) {
    $NDK_BIN = "$NDK_HOME\toolchains\llvm\prebuilt\windows-x86_64\bin"
    $env:Path = "$NDK_BIN;$env:Path"

    # Tell cc-rs which C compiler to use for each Android target
    $env:CC_aarch64_linux_android = "$NDK_BIN\clang.exe"
    $env:CC_armv7_linux_androideabi = "$NDK_BIN\clang.exe"
    $env:CC_i686_linux_android = "$NDK_BIN\clang.exe"
    $env:CC_x86_64_linux_android = "$NDK_BIN\clang.exe"

    # Tell cargo which linker to use (must use clang, not cc)
    $env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = "$NDK_BIN\clang.exe"
    $env:CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER = "$NDK_BIN\clang.exe"
    $env:CARGO_TARGET_I686_LINUX_ANDROID_LINKER = "$NDK_BIN\clang.exe"
    $env:CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER = "$NDK_BIN\clang.exe"

    # Pass --target to clang for proper sysroot/library linking
    $env:CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS = "-C link-arg=--target=aarch64-linux-android34"
    $env:CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_RUSTFLAGS = "-C link-arg=--target=armv7a-linux-androideabi34"
    $env:CARGO_TARGET_I686_LINUX_ANDROID_RUSTFLAGS = "-C link-arg=--target=i686-linux-android34"
    $env:CARGO_TARGET_X86_64_LINUX_ANDROID_RUSTFLAGS = "-C link-arg=--target=x86_64-linux-android34"
}

function Show-Help {
    Write-Host ""
    Write-Host "用法: .\dev-android.ps1 <命令>"
    Write-Host ""
    Write-Host "命令:"
    Write-Host "  init       初始化 Android 项目（首次必须执行）"
    Write-Host "  emulator   列出可用模拟器"
    Write-Host "  run        编译并安装开发版（热重载）"
    Write-Host "  logcat     查看应用日志"
    Write-Host "  inspect    打开 Edge DevTools 调试指南"
    Write-Host "  build      构建正式 APK"
    Write-Host "  help       显示此帮助"
    Write-Host ""
    Write-Host "首次调试流程:"
    Write-Host "  步骤1: .\dev-android.ps1 init        # 初始化 Android 项目"
    Write-Host "  步骤2: Android Studio 启动模拟器"
    Write-Host "  步骤3: .\dev-android.ps1 run         # 编译并安装到模拟器"
    Write-Host "  步骤4: .\dev-android.ps1 logcat      # 看日志定位问题"
    Write-Host ""
}

function Start-EmulatorList {
    Write-Host "列出可用模拟器..."
    & "$ANDROID_HOME\emulator\emulator" -list-avds
    Write-Host ""
    Write-Host "启动模拟器命令示例:"
    Write-Host ('  ' + [char]38 + ' "' + $ANDROID_HOME + '\emulator\emulator" -avd ' + [char]60 + '模拟器名称' + [char]62 + ' ' + [char]38)
    Write-Host ""
    Write-Host "建议: 使用 Android Studio GUI 启动模拟器更直观"
}

function Start-Run {
    Write-Host "编译并安装开发版 APK（带热重载）..."
    $env:TAURI_DEV_HOST = "10.0.2.2"
    node scripts/tauri-dev.mjs android
}

function Start-Logcat {
    Write-Host "查看应用日志（Ctrl+C 退出）..."
    Write-Host ""
    adb logcat | Select-String -Pattern "Tauri|console|Error|readTxtFile|com.vocab.flashcards"
}

function Show-Inspect {
    Write-Host ""
    Write-Host "Edge DevTools 调试步骤:"
    Write-Host "  1. 确保安卓设备/模拟器已连接 (adb devices)"
    Write-Host "  2. 在 Edge 地址栏打开: edge://inspect"
    Write-Host "  3. 点击单词卡片的 WebView 开始调试"
    Write-Host ""
    Write-Host "启动 Edge inspect 页面..."
    Start-Process "msedge://inspect"
}

function Start-Build {
    Write-Host "构建正式 APK..."
    npx tauri android build
    Write-Host ""
    Write-Host "APK 输出目录:"
    Write-Host "  src-tauri\gen\android\app\build\outputs\apk\"
}

function Start-Init {
    Write-Host "初始化 Android 项目..."
    Write-Host ""
    Write-Host "检查环境..."
    
    $rustOk = $false
    $androidOk = $false
    $ndkOk = $false
    
    try {
        $rustVersion = rustc --version 2>$null
        if ($rustVersion) {
            Write-Host "  [OK] Rust: $rustVersion"
            $rustOk = $true
        }
    } catch {}
    
    if (Test-Path $ANDROID_HOME) {
        Write-Host "  [OK] Android SDK: $ANDROID_HOME"
        $androidOk = $true
    } else {
        Write-Host "  [ERROR] Android SDK 未找到: $ANDROID_HOME"
    }
    
    $ndkPath = Get-ChildItem "$ANDROID_HOME\ndk" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($ndkPath) {
        Write-Host "  [OK] NDK: $($ndkPath.FullName)"
        $ndkOk = $true
    } else {
        Write-Host "  [WARN] NDK 未安装，请通过 Android Studio SDK Manager 安装"
    }
    
    Write-Host ""
    
    if (-not $rustOk) {
        Write-Host "请先安装 Rust: https://rustup.rs"
        return
    }
    
    if (-not $androidOk) {
        Write-Host "请先安装 Android SDK"
        return
    }
    
    Write-Host "执行 tauri android init..."
    npx tauri android init
}

switch ($Command) {
    "emulator" { Start-EmulatorList }
    "run" { Start-Run }
    "logcat" { Start-Logcat }
    "inspect" { Show-Inspect }
    "build" { Start-Build }
    "init" { Start-Init }
    default { Show-Help }
}

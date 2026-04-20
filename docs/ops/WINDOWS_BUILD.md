# Windows 本機編譯 Rust 後端

> Rust 在 Windows 上預設使用 MSVC toolchain，需 `link.exe`。若未安裝 Visual Studio，會出現 `error: linker 'link.exe' not found`。

## 方式 1：載入 MSVC 環境後編譯（推薦）

專案提供腳本自動尋找並載入 vcvars64：

```powershell
# 在專案根目錄執行
. .\scripts\load-msvc-env.ps1
cd backend
cargo build
# 或 cargo run、cargo test 等
```

## 方式 2：使用 build-backend 腳本

```powershell
.\scripts\build-backend.ps1           # 等同 cargo build
.\scripts\build-backend.ps1 --release # 等同 cargo build --release
```

## 方式 3：使用 Developer Command Prompt

若已安裝 Visual Studio：

1. 開啟「**x64 Native Tools Command Prompt for VS 2022**」（或 2019）
2. `cd` 到專案目錄
3. 執行 `cargo build` 等指令

## 安裝 Visual Studio Build Tools

### 方式 A：命令列一鍵安裝（需管理員）

```powershell
# 以管理員身分開啟 PowerShell，執行：
cd "c:\System Coding\ipig_system"
.\scripts\install-msvc-buildtools.ps1
```

或使用 CMD（右鍵「以系統管理員身分執行」）：
```cmd
cd "c:\System Coding\ipig_system"
scripts\install-msvc-buildtools.cmd
```

腳本會自動下載並安裝 C++ Build Tools（VCTools workload），約 10–30 分鐘。

### 方式 B：手動安裝

1. 下載：<https://visualstudio.microsoft.com/visual-cpp-build-tools/>
2. 執行安裝程式
3. 勾選 **「使用 C++ 的桌面開發」**（Desktop development with C++）
4. 或至少勾選：**MSVC v143**、**Windows 11 SDK**（或 10.0.22621.0）
5. 安裝完成後重新開啟終端機

## 替代方案：改用 GNU toolchain

若 MSVC 環境難以設定，可改用 GNU toolchain（需安裝 [MinGW-w64](https://www.mingw-w64.org/)）：

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
```

注意：GNU 與 MSVC 編譯的程式無法互相連結，若專案其他相依使用 MSVC 編譯，可能會有相容性問題。

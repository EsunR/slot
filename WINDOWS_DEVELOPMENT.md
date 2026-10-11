# Windows 桌面端编译与调试

本文说明如何在 Windows 上编译、运行和调试 `slot` 的桌面前端。桌面模式使用与掌机版相同的界面代码，但通过窗口、键盘和手柄代替掌机的屏幕与按键；它适合开发和预览，不需要连接掌机。

## 环境准备

1. 安装 Rustup 和 Rust 工具链。仓库的 `rust-toolchain.toml` 指定 Rust 1.96；进入仓库时，Rustup 会按该文件选择工具链。首次构建需要联网下载工具链和 Cargo 依赖。
2. 安装 Visual Studio 或 Build Tools 的 **使用 C++ 的桌面开发**工作负载，并包含 MSVC 编译工具与 Windows SDK。Rust 的 Windows MSVC 目标需要这些链接工具。
3. 确认显卡驱动支持 OpenGL 3.3 Core Profile。桌面窗口通过 OpenGL 绘制画面。
4. 在 PowerShell 中进入仓库根目录，检查工具链：

   ```powershell
   cd D:\Code\Github\slot
   rustup toolchain install 1.96 --profile minimal --component rustfmt,clippy
   cargo --version
   rustc --version
   ```

如果安装 Rustup 后 PowerShell 仍提示找不到 `cargo`，重新打开终端，让 `%USERPROFILE%\.cargo\bin` 加入当前会话的 `PATH`。

## 编译与运行

在仓库根目录运行：

```powershell
cargo run -p slot
```

`slot` 默认启用 `host` 功能，因此这条命令会编译并启动桌面窗口。首次构建需要下载依赖，后续构建通常只重新编译变更的部分。窗口关闭后，命令才会返回 PowerShell。

也可以分开编译和启动：

```powershell
cargo build -p slot
.\target\debug\slot.exe
```

调试构建的程序位于 `target\debug\slot.exe`。如需优化构建，使用 `cargo build --release -p slot`，生成文件位于 `target\release\slot.exe`。构建掌机版则是另一套命令：`cargo build --release -p slot --no-default-features --features device`，并且需要相应的 Linux/aarch64 构建环境；不要在 Windows 桌面调试时加 `device` 功能。

### 指定测试数据目录

桌面版默认从**当前工作目录**下的 `sdcard` 读取游戏、图片和设置。为避免在仓库根目录生成测试数据，可以在同一个 PowerShell 会话中指定独立目录：

```powershell
$env:SLOT_ROOT = (Join-Path (Get-Location) 'target\desktop-sdcard')
cargo run -p slot
```

程序会创建 `BIOS`、`Games`、`Labels`、`Saves`、`States`、`System`、`Wallpapers` 等目录。测试完成后运行 `Remove-Item Env:SLOT_ROOT`，即可恢复默认路径。没有游戏时，前端仍会启动，只是游戏列表为空。

如果需要测试实际游戏，把合法持有的 ROM 放入测试数据目录的 `Games` 中。模拟器核心优先从该目录的 `System` 中加载，例如 Windows 上的 `mgba_libretro.dll` 或 `gpsp_libretro.dll`；缺少核心时，界面仍可运行，启动游戏会使用内置测试图案并在标准错误输出中提示查找过的路径。也可以用 `SLOT_CORE` 指向某个核心 DLL 进行开发调试。

## 桌面输入

| 前端按键 | 键盘 | 手柄位置 |
| --- | --- | --- |
| A / B | `Z` / `X` | 右侧 / 下方面键 |
| X / Y | `C` / `V` | 上方 / 左侧面键 |
| 方向键 | 键盘方向键 | 十字键 |
| L1 / R1 | `A` / `S` | 左 / 右肩键 |
| L2 / R2 | `Q` / `W` | 左 / 右扳机键 |
| Start / Select | `Enter` / 右 `Shift` | Start / Select |
| Menu | `Tab` 或反引号键 | Mode |
| 音量增 / 减 | `=` / `-` | — |
| 电源 / 合盖 | 反斜杠键 / `]` | — |

手柄面键按**位置**映射，可能与手柄外壳印的 A/B 字母不同。键盘映射按物理键位处理。桌面端没有真实铰链，按一次 `]` 合盖，再按一次开盖。

## 调试与验证

开发时使用 `cargo run -p slot`，编译错误和程序的诊断信息会直接输出在终端。遇到 Rust panic，可在启动前打开回溯：

```powershell
$env:RUST_BACKTRACE = '1'
cargo run -p slot
```

也可以先用 `cargo build -p slot` 构建，再在 Visual Studio 中通过“调试 > 附加到进程”选择 `slot.exe`。调试构建包含符号信息，适合设置断点。修改源码后，先关闭正在运行的 `slot.exe`，再重新构建或运行；Windows 会锁定运行中的可执行文件，导致 Cargo 报“failed to remove file ... slot.exe / 拒绝访问”。

需要核对输入解析时，可以运行：

```powershell
cargo test -p slot --test evdev
```

## 从 Windows 构建 Linux 掌机版

掌机版面向 aarch64 Linux。Windows 不能直接执行生成的 ELF 文件；可在 WSL2 的 Ubuntu 22.04 中使用 aarch64 GCC 交叉编译。以下准备步骤只需执行一次。先在 PowerShell 中运行 `wsl -d Ubuntu` 进入 WSL，然后在 WSL 终端运行：

```bash
sudo apt-get update
sudo apt-get install -y build-essential gcc-aarch64-linux-gnu libc6-dev-arm64-cross python3 file curl unzip
curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none
. "$HOME/.cargo/env"
rustup toolchain install 1.96 --profile minimal --component rustfmt,clippy --target aarch64-unknown-linux-gnu
```

如果 WSL 当前用户是 `root`，安装系统包时可以省略 `sudo`。若 `apt-get update` 被已有的第三方软件源报错中断，先修复该源；Ubuntu 官方软件包索引已更新时，也可以继续尝试 `apt-get install`。WSL 需要能访问仓库所在的 Windows 盘符；本仓库在 WSL 中对应 `/mnt/d/Code/Github/slot`。

完成准备后，回到仓库根目录的 PowerShell，执行：

```powershell
.\tools\build-device-wsl.ps1
```

脚本会在 WSL 中调用 `aarch64-linux-gnu-gcc` 链接器，以 `--no-default-features --features device` 构建，然后检查 ELF 架构、中文界面字符串和 glibc 依赖。生成文件位于 `target-device\aarch64-unknown-linux-gnu\release\slot`。若 WSL 发行版名称不是 `Ubuntu`，传入 `-Distro <发行版名称>`。

安伯尼克 [RG35XXSP 官方规格](https://win.anbernic.com/product/409.html)标明屏幕为 640×480。针对它编译时使用：

```powershell
.\tools\build-device-wsl.ps1 -Rg35xxsp
```

此命令启用 `rg35xxsp` 功能，让内部界面直接按 640×480 绘制，产物位于 `target-rg35xxsp\aarch64-unknown-linux-gnu\release\slot`。GBA 游戏画面保持 3:2 比例，在屏幕宽度内居中显示；GB/GBC 仍以整数 3 倍显示。默认的 720×480 构建保留在单独目录中。

上面的路径是 Cargo 构建目录，只有程序文件。要生成与 `target\desktop-sdcard` 同结构的内容卡目录（并放入 AArch64 版 mGBA、gpSP 核心及许可证），执行：

```powershell
.\tools\package-rg35xxsp-wsl.ps1
```

完整目录在 `target-rg35xxsp-sdcard\`，可将其中的 `BIOS`、`Games`、`Labels`、`Overlay`、`Saves`、`States`、`System`、`Wallpapers` 复制到内容卡根目录；程序位于 `System\slot`。`System\fonts` 包含 Plix Bold 字体及许可证；程序内也有后备字体。首次打包需要联网下载两个核心、gpSP 对应源码和 Plix 字体。此目录提供应用文件，掌机仍需能启动该程序的 Linux 固件或启动脚本；仅复制文件不能替换固件。

`Overlay` 包含 GBA 遮罩 `GBA.png`，来自用户提供的 `assets/overlays/default.png`；随附原配置保存在 `assets/overlays/default.cfg`。RG35XXSP 的 GBA 游戏画面按配置放在左上角 (0,0)，大小为 640×426，图片原样铺满 640×480 屏幕。遮罩边缘会盖住游戏最外侧的约 8 像素；`SELECT+X` 仍切换 LCD 滤镜，滤镜的 3×3 单元按屏幕像素排列。应用直接加载 PNG，不读取 RetroArch 的 `.cfg`。[RetroGame66 的 2026-10-02 Release](https://github.com/RetroGame66/slot/releases/tag/v2026.10.02) 中另外五张 GB/GBC 遮罩均为 720×480；放入内容卡后，前端会在 RG35XXSP 上从左右各裁去 40 像素，使中央游戏开口与 480×432 画面对齐。

手动构建时，在 WSL 的仓库根目录使用等价命令：

```bash
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
cargo build --release --target aarch64-unknown-linux-gnu --target-dir target-device \
  -p slot --no-default-features --features device
python3 verify_slot.py target-device/aarch64-unknown-linux-gnu/release/slot
python3 check_glibc.py target-device/aarch64-unknown-linux-gnu/release/slot
```

`check_glibc.py` 输出的最高 `GLIBC_*` 版本不得高于目标系统提供的版本。AGS-102 使用 glibc 2.35；本仓库当前构建出的二进制经检查最高要求为 2.29。其他 Linux 掌机还必须分别核对 CPU 架构、系统库、显示驱动和输入设备。

如需在没有掌机时确认 ARM 程序至少能够启动，可以在 WSL 中安装 `qemu-user`，运行只创建目录、不访问图形硬件的入口：

```bash
sudo apt-get install -y qemu-user
qemu-aarch64 -L /usr/aarch64-linux-gnu \
  target-device/aarch64-unknown-linux-gnu/release/slot \
  --init-root /tmp/slot-device-init-check
```

这一步不能验证掌机的 EGL、按键、音频或实际游戏运行。

单独运行 `build-device-wsl.ps1` 只生成前端程序。实际放到 AGS-102 内容卡时，还需要 `System/slot`、对应 aarch64 版的 `mgba_libretro.so` 和 `gpsp_libretro.so`，以及 README 中的卡目录结构。RG35XXSP 可使用上面的专用打包脚本；默认设备版本可使用仓库的 `task dist:device` 流程，它面向 arm64 Linux 构建环境。没有连接掌机时，上述检查只验证构建产物，无法代替设备上的启动测试。

## 常见问题

| 现象 | 处理方式 |
| --- | --- |
| `cargo` 或 `rustc` 找不到 | 安装 Rustup，重开 PowerShell，确认 `%USERPROFILE%\.cargo\bin` 在 `PATH` 中。 |
| 找不到 `link.exe` 或 Windows SDK | 安装 Visual Studio/Build Tools 的 C++ 桌面开发工作负载及 Windows SDK。 |
| OpenGL 上下文创建失败 | 更新显卡驱动，确认当前环境支持 OpenGL 3.3；远程桌面或虚拟机也可能限制图形能力。 |
| `slot.exe` 无法删除或覆盖 | 关闭正在运行的 `slot.` 窗口，再执行 `cargo build`、`cargo run` 或测试。 |
| 画面中没有游戏 | 检查 `SLOT_ROOT` 指向的目录以及其中的 `Games`。 |
| 启动游戏后出现测试图案 | 检查核心 DLL 是否存在，查看终端中的 `slot: no ... core found` 提示。 |

桌面窗口默认大小为 1440×960；内部界面按 720×480 绘制，再按窗口尺寸缩放。调整窗口大小不会改变内部绘制分辨率。

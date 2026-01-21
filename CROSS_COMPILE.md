# 交叉编译说明

## 从 Linux 编译 Windows 可执行文件

### 1. 安装 mingw 工具链

**Debian/Ubuntu:**
```bash
sudo apt-get install gcc-mingw-w64-x86-64 g++-mingw-w64-x86-64
```

**Fedora/RHEL:**
```bash
sudo dnf install mingw64-gcc mingw64-gcc-c++ mingw64-winpthreads-static
```

**Arch Linux:**
```bash
sudo pacman -S mingw-w64
```

### 2. 添加 Rust Windows 目标

```bash
rustup target add x86_64-pc-windows-gnu
```

### 3. 交叉编译

```bash
# 编译 Windows 可执行文件
cargo build --release --target x86_64-pc-windows-gnu

# 产物位于: target/x86_64-pc-windows-gnu/release/game-archive-manager.exe
```

### 4. 脚本方式

```bash
# 一键交叉编译 (需要 mingw 已安装)
./cross-build.bat.sh
```

### 注意事项

1. **静态链接**: 配置已启用静态链接，生成的 .exe 是独立可执行文件，无需运行时
2. **文件大小**: 静态链接会导致可执行文件较大（通常 2-5MB）
3. **无依赖**: 目标机器无需安装任何运行时库

### 故障排除

如果遇到链接错误:
```bash
# 检查 mingw 版本
x86_64-w64-mingw32-gcc --version

# 如果找不到 linker，确保工具链已正确安装
which x86_64-w64-mingw32-gcc
```

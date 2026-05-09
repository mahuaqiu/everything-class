# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

Window Handle Finder 是一个 Windows 窗口查找工具，用于快速定位窗口 Handle、Class Name、标题、PID 和进程名称。类似 Everything 的即时搜索风格，目标用户是自动化测试和 UI 自动化工具开发者。

## 构建与运行

```bash
# 开发构建
cargo build

# Release 构建（优化体积）
cargo build --release

# 运行程序
cargo run

# 代码检查
cargo check
cargo clippy
```

## 架构

三层架构结构：

```
src/
├── main.rs           # UI 层 - egui/eframe GUI 应用入口
├── windows_api.rs    # Windows API 封装层 - 窗口枚举和信息获取
└── window_info.rs    # 数据层 - WindowInfo 结构体定义
```

**WindowsApi** 模块封装关键 Windows API：
- `EnumWindows` / `EnumChildWindows` - 窗口枚举
- `GetWindowTextW` / `GetClassNameW` - 窗口信息获取
- `GetWindowThreadProcessId` / `GetProcessImageFileNameW` - 进程信息

**树形模式采用延迟加载**：子窗口仅在用户展开节点时才遍历，避免启动时遍历所有子窗口的性能开销。

## 依赖

| Crate | 用途 |
|-------|------|
| eframe/egui | GUI 框架（即时模式 UI） |
| windows | Windows API 绑定 |
| arboard | 剪贴板操作 |

## Release 构建优化

Cargo.toml 已配置体积优化：
- `opt-level = "z"` - 最小体积优化
- `lto = true` - 链接时优化
- `codegen-units = 1` - 单编译单元
- `panic = "abort"` - 移除 panic 展开
- `strip = true` - 移除符号

目标：可执行文件 < 2MB。
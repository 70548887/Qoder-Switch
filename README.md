# Qoder Switch

**Qoder IDE 多账号无感切换与会话管理工具**

Qoder Switch 是一款基于本地 MITM 代理的桌面端工具，通过透明劫持 IDE 网络请求实现多账号自动切换，同时提供完整的会话管理、流量审计与配额监控能力。

---

## 功能特性

**代理引擎**
- 基于 Hudsucker 的本地 MITM 代理，透明劫持 IDE 认证流量
- 自动替换请求头中的认证信息，实现无感账号切换
- 支持 HTTP/HTTPS 流量解析与 WebSocket 协议检测
- 自动 CA 证书生成与系统信任链安装

**账号管理**
- 多账号池管理（添加 / 编辑 / 删除 / 启用 / 禁用）
- 余额阈值自动轮换策略
- 远程账号池拉取（密钥认证）
- 实时额度查询与状态监控

**会话管理**
- 自动发现本地 Qoder 工作区
- 对话历史浏览与全文搜索
- 会话备份 / 恢复 / 导出 Markdown
- 批量工作区备份

**流量日志**
- 完整 HTTP 请求/响应日志（含 Header 与 Body）
- 请求域名自动发现与归类
- 目标域名白名单管理

**仪表盘**
- 账号总数、活跃账号、会话数、备份数统计
- 代理运行状态实时监控

---

## 快速开始

### 1. 下载安装

从 [Releases](https://github.com/70548887/Qoder-Switch/releases) 页面下载最新安装包（`.exe`，约 6MB），双击运行 NSIS 安装向导完成安装。

> 系统要求：Windows 10/11 x64

### 2. 安装证书

首次启动后，进入「使用指南」页面，点击「安装证书」将 CA 证书添加到系统信任存储。此步骤为代理解析 HTTPS 流量的前置条件。

### 3. 添加账号

在账号管理页面添加一个或多个账号（粘贴 Token 即可），设置其中一个为当前使用账号。

### 4. 启动代理

点击顶部状态栏的启动按钮，代理将在本地 `127.0.0.1:5888` 监听。配合 IDE 代理设置指向该地址即可生效。

### 5. 开始使用

代理运行后，IDE 的所有认证请求将自动使用当前选中账号的凭证。可在仪表盘实时查看请求统计与账号状态。

---

## 截图预览

> 截图待补充。主界面包含左侧导航栏（仪表盘 / 账号 / 会话 / 指南 / 日志 / 设置）与右侧内容面板，整体采用深色主题。

---

## 技术架构

### 技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| 前端 | Vue 3 + TypeScript + TailwindCSS | SPA，Pinia 状态管理 |
| 桌面框架 | Tauri v2 | 原生窗口、系统托盘、IPC 通信 |
| 后端 | Rust (tokio 异步运行时) | 代理引擎、加密、数据持久化 |
| 代理核心 | Hudsucker 0.24 | MITM 代理实现，基于 hyper |
| 数据存储 | SQLite (rusqlite) | 账号、日志、会话数据本地持久化 |
| 证书 | rcgen + winreg | 动态 CA 生成与 Windows 证书存储操作 |

### 项目结构

```
qoder-proxy-rust/
├── src/                          # 前端源码
│   ├── components/               # Vue 组件
│   │   ├── AccountForm.vue       # 账号添加表单
│   │   ├── AccountList.vue       # 账号列表管理
│   │   ├── ChatHistoryPanel.vue  # 会话历史面板
│   │   ├── ConfigPanel.vue       # 设置面板
│   │   ├── LogsPanel.vue         # 流量日志
│   │   ├── MetricsPanel.vue      # 仪表盘
│   │   └── ...
│   ├── stores/app.ts             # Pinia 全局状态
│   ├── i18n/                     # 国际化（中/英）
│   └── App.vue                   # 根组件
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── proxy/                # MITM 代理引擎
│   │   ├── auth/                 # 认证与加密
│   │   ├── commands/             # Tauri IPC 命令
│   │   ├── account_pool.rs       # 账号池逻辑
│   │   ├── chat.rs               # 会话管理
│   │   ├── config.rs             # 配置管理
│   │   ├── metrics.rs            # 统计指标
│   │   ├── quota.rs              # 额度查询
│   │   ├── state.rs              # 应用状态
│   │   ├── tray.rs               # 系统托盘
│   │   └── lib.rs                # 入口
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── vite.config.ts
```

### 核心模块

- **proxy** — MITM 代理引擎，负责流量拦截、请求改写、日志记录
- **auth** — 认证信息加解密、Token 管理
- **account_pool** — 多账号池调度、阈值轮换策略
- **chat** — 工作区发现、对话解析、备份/恢复
- **commands** — Tauri IPC 命令层，桥接前端调用与后端逻辑
- **config** — 应用配置持久化（代理端口、自动启动、语言等）

---

## 开发指南

### 环境要求

- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/) >= 8
- [Rust](https://rustup.rs/) >= 1.75 (MSVC toolchain)
- Visual Studio Build Tools 2022 (C++ 桌面开发工作负载)
- Windows 10/11 SDK

### 本地开发

```bash
# 安装前端依赖
pnpm install

# 启动开发模式（前端热重载 + Rust 编译）
pnpm tauri dev
```

### 构建发布包

```bash
pnpm tauri build
```

产物位于 `src-tauri/target/release/bundle/nsis/`。

---

## 路线图

### 已完成

- [x] 本地 MITM 代理引擎
- [x] 多账号池管理与自动切换
- [x] 余额阈值自动轮换
- [x] 完整流量日志记录
- [x] 会话管理（发现/浏览/备份/恢复/导出）
- [x] 仪表盘统计
- [x] 域名管理与代理配置
- [x] CA 证书自动安装
- [x] 系统托盘与后台运行
- [x] 中英文国际化

### 计划中

- [ ] macOS / Linux 支持
- [ ] 自动更新（Tauri Updater）
- [ ] 账号分组与标签
- [ ] 流量统计图表
- [ ] 插件系统

---

## 许可证

本项目基于 [MIT License](./LICENSE) 开源。

---

## 致谢

本项目在开发过程中参考并借鉴了以下优秀项目的设计思路：

- [qoderwork-account-switcher](https://github.com/963072676/qoderwork-account-switcher)
- [QoderSessionManager](https://github.com/luckySpro/QoderSessionManager)

感谢以上项目作者的贡献与启发。

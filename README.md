# 东神脚手架 · 环境安装器

> Windows 开发环境一键安装桌面应用，基于 Tauri 2 + Vite 构建。

## 功能概览

自动化下载、安装和配置以下开发工具，支持版本选择、环境检测、安装取消：

### 核心环境（自动安装 + 配置环境变量）

| 组件 | 默认版本 | 来源 | 说明 |
|------|----------|------|------|
| **Node.js** | v20.19.0 | 清华 / npmmirror | MSI 安装 + `NODE_HOME` + `PATH` |
| **JDK** | 17 (OpenJDK) | 华为云 | 解压安装 + `JAVA_HOME` + `PATH` |
| **Maven** | 3.9.6 | 华为云 | 解压安装 + `MAVEN_HOME` + `PATH` + 阿里云 `settings.xml` |
| **MySQL** | 8.0.36 | 清华 / 阿里云 | 绿色版：初始化 + 注册服务 + 设置 root 密码 + `MYSQL_HOME` + `PATH` |

### 附加工具（仅下载，需用户手动安装）

| 组件 | 默认版本 | 来源 | 说明 |
|------|----------|------|------|
| **IntelliJ IDEA** | 2023.3.8 | JetBrains 中国 CDN | 下载 exe 到安装目录 |
| **Navicat Premium** | 16.2 | Navicat 中国站 | 下载 exe 到安装目录 |
| **Redis** | 5.0.14.1 | 华为云 / GitHub | 下载 ZIP 解压到安装目录 |

## 安装流程

```
Step 1 配置  →  Step 2 检测  →  Step 3 安装  →  Step 4 完成
选择安装路径      扫描已安装环境     冲突清理 + 下载     验证 + 结果展示
选择版本          版本匹配判断       安装 + 配置         一键验证命令
MySQL 密码        绿/黄/红指示灯     实时进度
附加工具勾选
模拟测试开关
```

### 环境冲突处理

当主机已存在旧版本开发环境但与用户选择的版本不匹配时，安装器会在安装前自动执行冲突清理：

| 组件 | 冲突处理策略 |
|------|-------------|
| **Node.js** | 从注册表查找旧版 MSI 产品代码 → `msiexec /x` 静默卸载 → 清理 PATH → 重置 NODE_HOME |
| **JDK** | 移除 PATH 中旧 JDK 条目 → 清理 Oracle javapath 快捷方式目录 → 重置 JAVA_HOME |
| **Maven** | 移除 PATH 中旧 Maven 条目(mvn.cmd/mvn.bat) → 重置 MAVEN_HOME 和 M2_HOME |
| **MySQL** | 移除 PATH 中旧 MySQL 条目 → 重置 MYSQL_HOME → 停止/删除旧服务 |

## 项目结构

```
web_manage_install/
├── src/                           # 前端（纯 HTML + CSS + JS）
│   ├── index.html                 #   主页面（四步骤 SPA）
│   ├── main.js                    #   入口：认证守卫 + 动态加载业务模块
│   ├── styles.css                 #   样式
│   └── js/
│       ├── navigation.js          #     步骤导航
│       ├── detect.js              #     Step 2 环境检测 + installFlags 维护
│       ├── installer.js           #     Step 3 安装逻辑 + 取消 + 回滚
│       ├── results.js             #     Step 4 结果展示
│       ├── versions.js            #     版本目录加载（后端实时获取）
│       ├── auth.js                #     认证逻辑
│       └── authUI.js              #     认证 UI
├── src-tauri/                     # Rust 后端
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                #   Tauri 入口 + 命令注册
│       ├── common/                #   公共能力
│       │   ├── types.rs           #     InstallConfig / CancelToken 等类型
│       │   ├── process.rs         #     跨平台进程工具（隐藏窗口）
│       │   ├── version_policy.rs  #     版本策略集中定义
│       │   └── ...
│       ├── detection/             #   环境检测
│       │   ├── mod.rs             #     检测总调度（detect_environment 命令）
│       │   ├── components/        #     各组件检测器（node/jdk/maven/mysql）
│       │   ├── env/               #     注册表环境变量读取 + 命令执行
│       │   ├── finder/            #     多来源扫描（where/注册表/Program Files）
│       │   └── verify.rs          #     安装后验证命令
│       ├── download/              #   下载能力
│       │   ├── sources/           #     各组件镜像源定义
│       │   ├── stream.rs          #     流式下载 + 进度回调
│       │   ├── cache.rs           #     下载缓存
│       │   └── proxy.rs           #     代理绕过配置
│       ├── install/               #   安装业务
│       │   ├── components/        #     各组件安装器（node/jdk/maven/mysql/bundled）
│       │   ├── conflict/          #     环境冲突解决（安装前清理旧版本）
│       │   │   ├── mod.rs         #       统一 resolve_conflicts() 调度入口
│       │   │   ├── path_sanitizer.rs #    通用 PATH 净化器
│       │   │   ├── node_cleanup.rs #      Node.js MSI 卸载 + PATH 清理
│       │   │   ├── jdk_cleanup.rs  #      JDK PATH + Oracle javapath 清理
│       │   │   ├── maven_cleanup.rs #     Maven PATH + M2_HOME 清理
│       │   │   └── mysql_cleanup.rs #     MySQL PATH + 环境变量清理
│       │   ├── mysql/             #     MySQL 专用安装细节（service/config/password/...）
│       │   └── workflow/          #     安装流程编排（orchestrator/cancel/rollback）
│       ├── system/                #   系统能力
│       │   ├── env_config.rs      #     环境变量配置门面（HKLM→HKCU→setx 三级回退）
│       │   ├── env_registry.rs    #     注册表读写
│       │   ├── env_broadcast.rs   #     WM_SETTINGCHANGE 广播
│       │   └── path_entry.rs      #     PATH 字符串操作
│       └── version_catalog/       #   版本目录（实时获取各组件可用版本）
└── package.json
```

## 技术特性

### 下载镜像（全部国内优先）

| 组件 | 镜像优先级 |
|------|-----------|
| Node.js | 清华源 → npmmirror → nodejs.org |
| JDK | 华为云 → java.net → Adoptium |
| Maven | 华为云 → Apache Archive |
| MySQL | 清华源 → 阿里云 → MySQL CDN |
| IDEA | JetBrains 中国 CDN → JetBrains 国际 CDN |
| Navicat | Navicat 中国站 → Navicat 国际站 |
| Redis | 华为云 → GitHub |

### 代理兼容

应用启动时自动检测系统代理（`HTTP_PROXY` / `ALL_PROXY`），为所有下载域名配置 `NO_PROXY` 绕过，避免代理导致国内镜像变慢或失败。

### 动态环境检测

采用多策略检测，从 Windows 注册表实时读取最新环境变量：
1. 注册表 `HKLM` / `HKCU` 动态读取 `PATH` 和 `*_HOME`
2. `where` 命令搜索
3. 注册表 App Paths / Uninstall 键
4. Program Files + 常见安装目录扫描

### 环境变量写入

三级回退策略：`HKLM`（系统级，需管理员）→ `HKCU`（用户级）→ `setx` 命令，确保无管理员权限也能配置。

### MySQL 安装加固

- 路径 ASCII 校验 + 正斜杠
- `chcp 65001` 强制 UTF-8 控制台
- `my.ini` 精简配置（对齐已验证模板，不含 `skip-name-resolve`）
- 密码设置双保险：直连失败则自动 `skip-grant-tables` 安全模式重置
- VC++ Runtime 检查

## 开发

### 环境要求

- Node.js >= 18
- Rust >= 1.70
- [Tauri 2 CLI](https://v2.tauri.app/start/prerequisites/)

### 启动开发服务器

```bash
npm install
npm run tauri:dev
```

### 构建生产包

```bash
npm run tauri:build
```

构建产物位于 `src-tauri/target/release/bundle/`。

## 模拟测试模式

Step 1 可勾选「模拟测试模式」：
- 全部 7 个资源（4 核心 + 3 附加）均会下载到临时目录验证镜像可用性
- 不执行任何安装操作，不修改系统环境变量
- 适合在部署前预检网络和资源

## License

Internal use only.

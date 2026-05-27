# 东神脚手架 · 环境安装器

> Windows 开发环境一键安装桌面应用，基于 Tauri 2 + Vite 构建。

## 功能概览

自动化下载、安装和配置以下开发工具，支持环境检测、安装取消与回滚：

| 组件 | 来源 | 说明 |
|------|------|------|
| **Node.js** v20.19.0 | 国内镜像下载 | 自动配置 `NODE_HOME` + `PATH` |
| **JDK 17** (OpenJDK) | 国内镜像下载 | 自动配置 `JAVA_HOME` + `PATH` |
| **Maven** 3.9.6 | 国内镜像下载 | 自动配置 `MAVEN_HOME` + `PATH` + 阿里云 `settings.xml` |
| **MySQL** 8.0.36 | 国内镜像下载 | 自动初始化数据库、配置服务、设置 root 密码 |
| **IntelliJ IDEA** 2023.3.8 | JetBrains 中国 CDN 下载 | 静默安装，激活工具随 exe 打包 |
| **Navicat Premium** 16.2 | Navicat 中国站下载 | 静默安装，激活工具随 exe 打包 |
| **Redis** 3.2.100 | 随 exe 打包 | 解压即用（绿色免安装） |

## 安装流程

```
Step 1 配置  →  Step 2 检测  →  Step 3 安装  →  Step 4 完成
选择安装路径      扫描已安装环境     下载/复制/安装     验证 + 结果展示
MySQL 密码       跳过已有组件       支持取消+回滚      一键验证命令
附加工具勾选
```

## 项目结构

```
web_manage_install/
├── src/                           # 前端（纯 HTML + CSS + JS）
│   ├── index.html                 #   主页面（四步骤 SPA）
│   ├── main.js                    #   入口：初始化、事件绑定
│   ├── styles.css                 #   样式
│   └── js/
│       ├── navigation.js          #     步骤导航
│       ├── detect.js              #     Step 2 环境检测
│       ├── installer.js           #     Step 3 安装逻辑 + 取消/回滚
│       ├── results.js             #     Step 4 结果展示
│       └── versions.js            #     版本号常量
├── public/                        # 小体积资源（打包进 exe）
│   ├── idea-activation.7z         #   IDEA 激活工具 (~70KB)
│   ├── navicat-activation.7z      #   Navicat 激活工具 (~600KB)
│   └── Redis-x64-3.2.100.zip     #   Redis 压缩包 (~5MB)
├── src-tauri/                     # Rust 后端
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                 #   IPC 命令注册 + 应用启动
│       ├── main.rs                #   Tauri 入口
│       ├── types.rs               #   InstallConfig / CancelToken 等类型
│       ├── download.rs            #   HTTP 下载 + 代理绕过
│       ├── env_config.rs          #   环境变量读写 + WM_SETTINGCHANGE 广播
│       ├── detect/
│       │   ├── mod.rs             #     环境检测总调度
│       │   ├── env_reader.rs      #     动态发现工具（where/注册表/目录扫描）
│       │   ├── node.rs            #     Node.js 检测
│       │   ├── jdk.rs             #     JDK 检测
│       │   ├── maven.rs           #     Maven 检测
│       │   ├── mysql.rs           #     MySQL 检测
│       │   └── verify.rs          #     安装后验证（带重试）
│       └── installers/
│           ├── mod.rs             #     安装总调度 + 回滚
│           ├── node.rs            #     Node.js 安装
│           ├── jdk.rs             #     JDK 安装
│           ├── maven.rs           #     Maven 安装
│           ├── mysql.rs           #     MySQL 安装（中文系统适配）
│           ├── bundled.rs         #     本地资源安装器（IDEA/Navicat/Redis）
│           └── utils.rs           #     解压、文件操作工具
└── package.json
```

## 技术特性

### 代理兼容
应用启动时自动检测系统代理（`HTTP_PROXY` / `ALL_PROXY`），为所有下载域名配置 `NO_PROXY` 绕过：
- 清华源、阿里云、华为云、npmmirror 等国内镜像直连
- JetBrains 中国 CDN / Navicat 中国站直连
- 避免代理导致国内下载变慢或失败

### 动态环境检测
采用多策略检测，避免硬编码路径：
1. `PATH` 中执行命令
2. `*_HOME` 环境变量
3. `where` 命令搜索
4. Windows 注册表 App Paths / Uninstall 键
5. Program Files 目录扫描
6. 常见安装目录扫描

### 中文系统适配
MySQL 安装针对中文 Windows 做了加固：
- 路径 ASCII 校验 + 前向斜杠
- `chcp 65001` 强制 UTF-8 控制台
- `my.ini` 含 `lc-messages-dir` + `skip-name-resolve`
- VC++ Runtime 检查

### 安装取消与回滚
- 安装过程中随时取消（基于 `AtomicBool` 令牌）
- 回滚已安装组件（删除文件/目录 + 清理环境变量）

### 资源分发策略
- **大体积安装包**（IDEA ~700MB / Navicat ~95MB）从官方中国 CDN 实时下载，不打包进 exe
- **小体积工具**（激活工具 + Redis ZIP，共 ~6MB）打包在 exe 内随应用分发
- 安装时自动将激活工具和 Redis 复制到用户指定的安装路径

### 下载镜像
所有组件均优先使用国内镜像/CDN，失败自动切换下一个：

| 组件 | 镜像优先级 |
|------|-----------|
| Node.js | 清华源 → npmmirror → nodejs.org |
| JDK | 华为云 → java.net → Adoptium |
| Maven | 华为云 → Apache Archive |
| MySQL | MySQL CDN |
| IDEA | JetBrains 中国 CDN → JetBrains 国际 CDN |
| Navicat | Navicat 中国站 → Navicat 国际站 |

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

> **注意**：IDEA 和 Navicat 安装包从网络下载（不打包），`public/` 下仅存放小体积的激活工具和 Redis（~6MB），会打包进 exe。

## 模拟测试模式

Step 1 可勾选「模拟测试模式」：
- 6 个网络资源（Node/JDK/Maven/MySQL/IDEA/Navicat）：下载到临时目录验证镜像可用性
- Redis：验证本地 ZIP 文件是否存在
- 不执行任何安装操作，不修改系统环境变量

适合在部署前预检网络和资源。

## License

Internal use only.

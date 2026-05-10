# Codexx

Codexx 是基于 OpenAI `codex` 仓库维护的一个 CLI / TUI fork，目标是把它调整成更适合国内上游、第三方 OpenAI 兼容接口和日常本地代理工作流的版本。

这个 fork 当前重点补充了以下方向：

- 更灵活的上游接入方式，包括全局 `wire_api`、HTTP 默认连接和可选 WebSocket
- 初始化时可直接填写自定义 `base_url` 与 API Key
- 配置项扩展，例如 `yolo`、`auto_goal`、`auto_commit`
- `/model` 动态读取真实 `/models` 接口并支持自定义模型名
- `/commit` 快捷命令和目标完成后的自动 commit 提示词流程
- 独立的 GitHub Actions 发布工作流，直接产出 `codexx` 二进制

## 适用场景

- 你想把 Codex CLI 接到自定义 OpenAI 兼容上游
- 你需要一个支持实验性工作流、自动目标、自动 commit 提示词的终端代理
- 你希望直接下载预编译 `codexx` 二进制，而不是自己从上游发行版里手动改名

## 快速开始

### 1. 直接运行

如果你已经拿到发行版：

- macOS / Linux：解压后给 `codexx` 添加执行权限，再直接运行
- Windows：解压后运行 `codexx.exe`

### 2. 从源码构建

仓库根目录已经内置 fork 专用脚本，按速度和用途分成几条路径：

```bash
scripts/bootstrap-build-env.sh
scripts/verify-codexx.sh
```

日常本地运行调试，直接走只编译不打包的调试入口：

```bash
scripts/run-codexx-debug.sh -- --help
```

如果需要本地可交付产物：

```bash
scripts/build-codexx.sh
```

最终 release 打包使用：

```bash
scripts/build-codexx-release.sh
```

构建完成后，稳定产物路径为：

```bash
build/codexx
```

调试运行路径不会生成 `build/codexx`，而是直接使用：

```bash
codex-rs/target/debug/codex
```

### 3. 启动

```bash
./build/codexx
```

首次初始化配置时，程序会支持直接输入：

- 自定义 `base_url`
- API key
- 模型名称
- 是否启用部分自动化行为

## 配置增强

相对上游，这个 fork 额外关注 OpenAI 兼容服务和自动化工作流，当前包含的典型配置能力有：

- `wire_api`：支持全局默认值，也兼容渠道级覆盖
- `yolo`：一键切换到 YOLO 模式
- `auto_goal`：每轮任务自动启用 goal 模式
- `auto_commit`：任务目标完成后自动触发 `/commit` 提示词流程

如果你使用的是不支持 WebSocket 的兼容客户端，当前默认会走 HTTP，只有显式启用 WebSocket 配置时才会使用 WebSocket。

## 发布方式

本项目包含独立的 GitHub Actions 发布工作流，不依赖上游私有签名或专用 runner。

- 推送 `v0.0.1` 这类 tag 时，会自动触发构建与发布
- 默认发布 Linux x86_64、macOS arm64、Windows x86_64 三个平台产物
- Release 文案使用中文模板，并自动汇总两个 tag 之间的提交记录

示例：

```bash
git tag v0.0.1
git push origin v0.0.1
```

## 文档索引

- 安装和构建说明：[`docs/install.md`](docs/install.md)
- 贡献说明：[`docs/contributing.md`](docs/contributing.md)
- 旧版上游 README 归档：[`README_OLD.md`](README_OLD.md)

## 说明

- 本项目仍继承上游 `codex` 的主体架构与绝大多数 crate 命名
- fork 产物名固定为 `codexx`，方便发布和交付
- 许可证沿用仓库原有的 [Apache-2.0 License](LICENSE)

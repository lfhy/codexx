# Codexx {{VERSION}} 发布说明

## 本次更新
{{ALL_UPDATES}}

## 重点变更
{{HIGHLIGHTS}}

## 修复
{{FIXES}}

## 使用说明
- 首次使用建议先阅读仓库 `README.md`，按其中说明完成配置。
- 发行版中的 CLI 主程序名统一为 `codexx`，适合直接下载后解压运行。
- 如果你使用 API key 模式，可以在初始化配置时直接填写自定义 `base_url`、`api_key`、模型和相关开关。

## 已知说明
- 当前 GitHub Release 为未签名构建产物，不包含私有签名或 notarization。
- macOS 首次运行若提示安全限制，可先执行：`xattr -dr com.apple.quarantine ./codexx`
- Windows 与 Linux 发行包默认只包含主程序，不附带额外安装器。

## 附件说明
- Linux：`codexx-x86_64-unknown-linux-gnu.tar.gz`
- macOS：`codexx-aarch64-apple-darwin.tar.gz`
- Windows：`codexx-x86_64-pc-windows-msvc.zip`
- 校验：`SHA256SUMS`

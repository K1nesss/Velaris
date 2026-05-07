# Update Release Guide

Velaris 使用 Tauri v2 updater 实现应用内更新。这个文件记录手动发布流程，后续可以再迁移到 GitHub Actions 自动化。

## 更新源

客户端会检查 GitHub Releases 中的：

```text
https://github.com/K1nesss/Velaris/releases/latest/download/latest.json
```

因此最新版本的 GitHub Release 必须包含 `latest.json`。

## 签名密钥

Updater 公钥已经写入：

```text
src-tauri/tauri.conf.json
```

私钥保存在本机：

```text
C:\Users\swp\.tauri\velaris-updater.key
```

私钥密码保存在本机：

```text
C:\Users\swp\.tauri\velaris-updater-password.txt
```

私钥和密码不能提交到仓库。丢失私钥或密码后，旧版本应用将无法校验新的更新包。

后续如果迁移到 CI，建议把私钥内容和密码保存到 GitHub Actions Secret，例如 `TAURI_SIGNING_PRIVATE_KEY` 和 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。

## 发布流程

1. 修改版本号：

```json
{
	"version": "0.2.0"
}
```

位置：

```text
src-tauri/tauri.conf.json
```

2. 构建安装包和 updater artifacts：

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY=(Get-Content -Raw "C:\Users\swp\.tauri\velaris-updater.key")
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD=(Get-Content -Raw "C:\Users\swp\.tauri\velaris-updater-password.txt")
cargo tauri build
```

3. 在 `src-tauri/target/release/bundle/` 中找到安装包、updater 包和 `.sig` 签名文件。

4. 创建 GitHub Release，例如：

```text
v0.2.0
```

5. 上传安装包、updater 包、签名文件和 `latest.json`。

## latest.json

`latest.json` 需要包含最新版本、更新说明、发布时间、下载地址和签名。

示例：

```json
{
	"version": "0.2.0",
	"notes": "新增手动添加游戏\n优化 Timeline 图标\n修复设置页动效开关",
	"pub_date": "2026-05-07T12:00:00Z",
	"platforms": {
		"windows-x86_64": {
			"signature": "这里填写 .sig 文件内容",
			"url": "https://github.com/K1nesss/Velaris/releases/download/v0.2.0/Velaris_0.2.0_x64-setup.nsis.zip"
		}
	}
}
```

文件名需要和实际构建产物一致。

## 应用内交互

- 启动主界面后延迟检查更新。
- 有新版本时右下角显示提醒。
- 同一版本每天只提醒一次。
- 用户可以进入设置页点击“检查更新”。
- 有更新时显示当前版本、新版本和更新内容。
- 用户点击“立即更新”后自动下载、安装并重启。

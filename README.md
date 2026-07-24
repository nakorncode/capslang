# CapsLang

![CapsLang](assets/capslang-icon.png)

Switch Windows input languages with **CapsLock**. Keep the real Caps Lock behind **Alt+CapsLock**. That is the whole product.

CapsLang lives in the tray, stays out of the way, and runs elevated so the remap still works when the focused window is an administrator app.

## Install

Grab the latest Windows build from
[Releases](https://github.com/nakorncode/capslang/releases/latest):

| File | Use |
| --- | --- |
| `CapsLang-Setup-win-x64.exe` | Installer |
| `CapsLang-Portable-win-x64.zip` | Unpack and run `CapsLang.exe` |
| `CapsLang-SHA256SUMS.txt` | Checksums |

No separate .NET install is required. Builds are self-contained `win-x64`.

## How it works

| Input | Result |
| --- | --- |
| `CapsLock` | Next Windows input language |
| `Alt+CapsLock` | Toggle real Caps Lock |

Language switching uses `WM_INPUTLANGCHANGEREQUEST` on the foreground window. CapsLang does **not** fake `Win+Space`, so you avoid chord accidents like `Win+Space+1`.

### Elevation (one UAC, then quiet)

Windows blocks low-level keyboard hooks from a normal process when an elevated window is focused. CapsLang therefore runs as administrator.

1. First launch asks for UAC once.
2. CapsLang registers a logon scheduled task (`NakornCode\CapsLang`) with highest privileges.
3. Later logons and tray restarts go through that task — no repeated prompts.

Defaults: remap **Enabled**, **Launch on startup**, and elevated task registration are all on.

### Tray menu

- **Enabled** — remap on/off
- **Launch on startup** — enable/disable the elevated logon task
- **Help** — short overview
- **Exit** — quit

Settings file: `%LOCALAPPDATA%\CapsLang\settings.json`

## Tips

- If PowerToys (or anything else) also remaps CapsLock, turn that remap off.
- Uninstall/disable startup from the tray, or remove the `NakornCode\CapsLang` task in Task Scheduler.
- CapsLang is Windows-only.

## Build

Needs .NET 8 SDK. Installer packaging also needs [Inno Setup 6](https://jrsoftware.org/isinfo.php).

```powershell
dotnet build
dotnet run --configuration Release
```

Ship assets locally:

```powershell
.\scripts\publish-release.ps1
```

Output lands in `artifacts\release\`.

## Release CI

Tag and push:

```powershell
git tag v0.2.0
git push origin v0.2.0
```

GitHub Actions publishes the portable ZIP, Inno installer, and checksums.

## Credit

[nakorncode](https://github.com/nakorncode)

## License

MIT

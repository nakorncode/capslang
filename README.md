# CapsLang

![CapsLang icon](assets/capslang-icon.png)

CapsLang is a tiny Windows tray app that turns `CapsLock` into an input-language
switch key.

It posts `WM_INPUTLANGCHANGEREQUEST` to the foreground window instead of
sending `Win+Space`, so fast typing cannot accidentally trigger shortcuts such
as `Win+Space+1` or `Win+Space+D`.

## Features

- `CapsLock` switches to the next Windows input language
- `Alt+CapsLock` toggles real CapsLock
- Quiet tray process (no settings window)
- Tray menu: Enabled, Launch on startup, Help, Exit
- Portable ZIP and Inno Setup installer from GitHub Releases

## Download

- [CapsLang-Setup-win-x64.exe](https://github.com/nakorncode/capslang/releases/latest/download/CapsLang-Setup-win-x64.exe) — installer
- [CapsLang-Portable-win-x64.zip](https://github.com/nakorncode/capslang/releases/latest/download/CapsLang-Portable-win-x64.zip) — portable build
- [SHA256 checksums](https://github.com/nakorncode/capslang/releases/latest/download/CapsLang-SHA256SUMS.txt)

Release builds are self-contained. No separate .NET runtime install is required.

## Key bindings

| Shortcut | Action |
| --- | --- |
| `CapsLock` | Switch to the next input language (keep CapsLock off) |
| `Alt+CapsLock` | Toggle real CapsLock |

## Tray menu

Right-click the CapsLang tray icon:

- **Enabled** — turn the CapsLock remap on or off
- **Launch on startup** — create or remove the Windows Startup shortcut
- **Help** — overview, bindings, limitations, and credit
- **Exit** — quit CapsLang

Defaults on first run: Enabled **on**, Launch on startup **on**.

Settings are saved under `%LOCALAPPDATA%\CapsLang\settings.json`.

## Notes

- Elevated apps (for example Task Manager running as administrator) may not
  receive CapsLang key handling unless CapsLang is also elevated. That is a
  Windows UIPI limitation.
- Disable any PowerToys CapsLock remap while CapsLang is running.
- CapsLang is Windows-only.

## Build from source

Requirements:

- Windows
- .NET 8 SDK
- [Inno Setup 6](https://jrsoftware.org/isinfo.php) (for installer packaging)

```powershell
dotnet build
```

Publish portable ZIP + installer locally:

```powershell
.\scripts\publish-release.ps1
```

Assets are written to `artifacts\release`.

## GitHub release

```powershell
git tag v0.1.0
git push origin v0.1.0
```

The release workflow builds self-contained `win-x64` assets and uploads:

- `CapsLang-Portable-win-x64.zip`
- `CapsLang-Setup-win-x64.exe`
- `CapsLang-SHA256SUMS.txt`

## Credit

Created by [nakorncode](https://github.com/nakorncode).

## License

MIT

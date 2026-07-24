using System.ComponentModel;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Security.Principal;
using System.Text.Json;
using System.Windows.Forms;

namespace CapsLang;

internal static class Program
{
    private const string MutexName = "Global\\NakornCode.CapsLang";
    private const int WH_KEYBOARD_LL = 13;
    private const int WM_KEYDOWN = 0x0100;
    private const int WM_KEYUP = 0x0101;
    private const int WM_SYSKEYDOWN = 0x0104;
    private const int WM_SYSKEYUP = 0x0105;
    private const int VK_CAPITAL = 0x14;
    private const int VK_MENU = 0x12;
    private const int WM_INPUTLANGCHANGEREQUEST = 0x0050;
    private const int INPUTLANGCHANGE_FORWARD = 0x0002;
    private static readonly IntPtr HKL_NEXT = new(1);
    private const uint KEYEVENTF_EXTENDEDKEY = 0x0001;
    private const uint KEYEVENTF_KEYUP = 0x0002;

    private static IntPtr hookId = IntPtr.Zero;
    private static LowLevelKeyboardProc? hookProc;
    private static AppSettings appSettings = new();
    private static Mutex? singleInstance;

    [STAThread]
    private static void Main()
    {
        ApplicationConfiguration.Initialize();

        if (!IsElevated())
        {
            HandOffToElevatedInstance();
            return;
        }

        singleInstance = new Mutex(true, MutexName, out var createdNew);
        if (!createdNew)
        {
            return;
        }

        var firstRun = !SettingsStore.Exists();
        appSettings = SettingsStore.Load();

        LegacyStartupShortcut.RemoveIfPresent();
        ElevatedStartup.EnsureRegistered(enableAtLogon: firstRun || ElevatedStartup.IsEnabled());

        if (firstRun)
        {
            SettingsStore.Save(appSettings);
        }

        hookProc = HookCallback;
        hookId = SetHook(hookProc);
        ForceCapsLockOff();

        using var trayIcon = CreateTrayIcon();
        Application.ApplicationExit += (_, _) =>
        {
            trayIcon.Visible = false;
            if (hookId != IntPtr.Zero)
            {
                UnhookWindowsHookEx(hookId);
            }

            singleInstance?.ReleaseMutex();
            singleInstance?.Dispose();
        };

        Application.Run();
    }

    private static void HandOffToElevatedInstance()
    {
        if (ElevatedStartup.Exists())
        {
            try
            {
                ElevatedStartup.Run();
                return;
            }
            catch (Exception)
            {
                // Fall through to a one-time UAC relaunch.
            }
        }

        try
        {
            Process.Start(new ProcessStartInfo
            {
                FileName = Application.ExecutablePath,
                WorkingDirectory = AppContext.BaseDirectory,
                UseShellExecute = true,
                Verb = "runas"
            });
        }
        catch (Win32Exception)
        {
            MessageBox.Show(
                """
                CapsLang needs one administrator approval so it can run elevated.

                That lets CapsLock switch languages inside elevated apps such as
                Task Manager, and lets Windows start CapsLang later without more UAC prompts.

                Run CapsLang again and choose Yes.
                """,
                "CapsLang",
                MessageBoxButtons.OK,
                MessageBoxIcon.Information);
        }
    }

    private static bool IsElevated()
    {
        using var identity = WindowsIdentity.GetCurrent();
        return new WindowsPrincipal(identity).IsInRole(WindowsBuiltInRole.Administrator);
    }

    private static NotifyIcon CreateTrayIcon()
    {
        var menu = new ContextMenuStrip();
        var enabledItem = new ToolStripMenuItem("Enabled")
        {
            CheckOnClick = true,
            Checked = appSettings.IsEnabled
        };
        var startupItem = new ToolStripMenuItem("Launch on startup")
        {
            CheckOnClick = true,
            Checked = ElevatedStartup.IsEnabled()
        };

        enabledItem.CheckedChanged += (_, _) =>
        {
            appSettings.IsEnabled = enabledItem.Checked;
            SettingsStore.Save(appSettings);
        };

        startupItem.CheckedChanged += (_, _) =>
        {
            if (startupItem.Checked)
            {
                ElevatedStartup.EnsureRegistered(enableAtLogon: true);
            }
            else
            {
                ElevatedStartup.SetEnabled(false);
            }
        };

        menu.Items.Add(enabledItem);
        menu.Items.Add(startupItem);
        menu.Items.Add(new ToolStripSeparator());
        menu.Items.Add("Help", null, (_, _) => ShowHelp());
        menu.Items.Add(new ToolStripSeparator());
        menu.Items.Add("Exit", null, (_, _) => Application.Exit());

        return new NotifyIcon
        {
            Icon = Icon.ExtractAssociatedIcon(Application.ExecutablePath) ?? SystemIcons.Application,
            Text = "CapsLang: CapsLock switches input language",
            ContextMenuStrip = menu,
            Visible = true
        };
    }

    private static void ShowHelp()
    {
        MessageBox.Show(
            """
            CapsLang is a tiny tray tool that remaps CapsLock to switch the
            Windows input language.

            CapsLock — next input language
            Alt+CapsLock — real CapsLock toggle

            CapsLang runs elevated by default so the remap also works in
            administrator windows. The first launch asks for UAC once, then a
            logon scheduled task starts CapsLang quietly afterward.

            Tray menu: Enabled, Launch on startup, Help, Exit.

            Turn off any PowerToys CapsLock remap while CapsLang is running.

            Credit by nakorncode
            https://github.com/nakorncode/capslang
            """,
            "CapsLang Help",
            MessageBoxButtons.OK,
            MessageBoxIcon.Information);
    }

    private static IntPtr SetHook(LowLevelKeyboardProc proc)
    {
        using var currentProcess = Process.GetCurrentProcess();
        using var currentModule = currentProcess.MainModule;
        var moduleHandle = currentModule?.ModuleName is { Length: > 0 }
            ? GetModuleHandle(currentModule.ModuleName)
            : IntPtr.Zero;

        return SetWindowsHookEx(WH_KEYBOARD_LL, proc, moduleHandle, 0);
    }

    private static IntPtr HookCallback(int nCode, IntPtr wParam, IntPtr lParam)
    {
        if (!appSettings.IsEnabled)
        {
            return CallNextHookEx(hookId, nCode, wParam, lParam);
        }

        if (nCode >= 0)
        {
            var message = wParam.ToInt32();
            var vkCode = Marshal.ReadInt32(lParam);

            if (vkCode == VK_CAPITAL)
            {
                if (message is WM_KEYDOWN or WM_SYSKEYDOWN)
                {
                    if (IsKeyDown(VK_MENU))
                    {
                        ToggleCapsLock();
                    }
                    else
                    {
                        ForceCapsLockOff();
                        SwitchToNextInputLanguage();
                    }
                }

                if (message is WM_KEYDOWN or WM_KEYUP or WM_SYSKEYDOWN or WM_SYSKEYUP)
                {
                    return new IntPtr(1);
                }
            }
        }

        return CallNextHookEx(hookId, nCode, wParam, lParam);
    }

    private static void SwitchToNextInputLanguage()
    {
        var foregroundWindow = GetForegroundWindow();
        if (foregroundWindow != IntPtr.Zero)
        {
            PostMessage(foregroundWindow, WM_INPUTLANGCHANGEREQUEST, new IntPtr(INPUTLANGCHANGE_FORWARD), HKL_NEXT);
        }
    }

    private static void ForceCapsLockOff()
    {
        if (IsCapsLockOn())
        {
            ToggleCapsLock();
        }
    }

    private static bool IsCapsLockOn() => (GetKeyState(VK_CAPITAL) & 1) != 0;

    private static bool IsKeyDown(int virtualKey) => (GetAsyncKeyState(virtualKey) & 0x8000) != 0;

    private static void ToggleCapsLock()
    {
        keybd_event(VK_CAPITAL, 0x45, KEYEVENTF_EXTENDEDKEY, UIntPtr.Zero);
        keybd_event(VK_CAPITAL, 0x45, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP, UIntPtr.Zero);
    }

    private delegate IntPtr LowLevelKeyboardProc(int nCode, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    private static extern IntPtr SetWindowsHookEx(int idHook, LowLevelKeyboardProc lpfn, IntPtr hMod, uint dwThreadId);

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool UnhookWindowsHookEx(IntPtr hhk);

    [DllImport("user32.dll")]
    private static extern IntPtr CallNextHookEx(IntPtr hhk, int nCode, IntPtr wParam, IntPtr lParam);

    [DllImport("kernel32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    private static extern IntPtr GetModuleHandle(string lpModuleName);

    [DllImport("user32.dll")]
    private static extern short GetKeyState(int nVirtKey);

    [DllImport("user32.dll")]
    private static extern short GetAsyncKeyState(int vKey);

    [DllImport("user32.dll")]
    private static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);

    [DllImport("user32.dll")]
    private static extern IntPtr GetForegroundWindow();

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool PostMessage(IntPtr hWnd, int msg, IntPtr wParam, IntPtr lParam);
}

internal sealed class AppSettings
{
    public bool IsEnabled { get; set; } = true;
}

internal static class SettingsStore
{
    private static readonly JsonSerializerOptions JsonOptions = new() { WriteIndented = true };

    private static string SettingsDirectory =>
        Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "CapsLang");

    private static string SettingsPath => Path.Combine(SettingsDirectory, "settings.json");

    public static bool Exists() => File.Exists(SettingsPath);

    public static AppSettings Load()
    {
        try
        {
            if (!File.Exists(SettingsPath))
            {
                return new AppSettings();
            }

            return JsonSerializer.Deserialize<AppSettings>(File.ReadAllText(SettingsPath), JsonOptions)
                ?? new AppSettings();
        }
        catch (JsonException)
        {
            return new AppSettings();
        }
        catch (IOException)
        {
            return new AppSettings();
        }
        catch (UnauthorizedAccessException)
        {
            return new AppSettings();
        }
    }

    public static void Save(AppSettings settings)
    {
        Directory.CreateDirectory(SettingsDirectory);
        File.WriteAllText(SettingsPath, JsonSerializer.Serialize(settings, JsonOptions));
    }
}

internal static class LegacyStartupShortcut
{
    private static string ShortcutPath =>
        Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Startup), "CapsLang.lnk");

    public static void RemoveIfPresent()
    {
        try
        {
            if (File.Exists(ShortcutPath))
            {
                File.Delete(ShortcutPath);
            }
        }
        catch (IOException)
        {
        }
        catch (UnauthorizedAccessException)
        {
        }
    }
}

internal static class ElevatedStartup
{
    private const string FolderName = "NakornCode";
    private const string TaskName = "CapsLang";
    private const int TaskCreateOrUpdate = 6;
    private const int TaskLogonInteractiveToken = 3;
    private const int TaskRunLevelHighest = 1;
    private const int TaskTriggerLogon = 9;
    private const int TaskActionExec = 0;

    public static bool Exists()
    {
        try
        {
            return GetTask() is not null;
        }
        catch (Exception)
        {
            return false;
        }
    }

    public static bool IsEnabled()
    {
        try
        {
            dynamic? task = GetTask();
            return task is not null && (bool)task.Enabled;
        }
        catch (Exception)
        {
            return false;
        }
    }

    public static void EnsureRegistered(bool enableAtLogon)
    {
        var service = Connect();
        var folder = GetOrCreateFolder(service);
        dynamic definition = service.NewTask(0);
        definition.RegistrationInfo.Description = "Start CapsLang elevated at logon so CapsLock can switch input languages in all apps.";
        definition.Principal.UserId = WindowsIdentity.GetCurrent().Name;
        definition.Principal.LogonType = TaskLogonInteractiveToken;
        definition.Principal.RunLevel = TaskRunLevelHighest;
        definition.Settings.Enabled = enableAtLogon;
        definition.Settings.StartWhenAvailable = true;
        definition.Settings.DisallowStartIfOnBatteries = false;
        definition.Settings.StopIfGoingOnBatteries = false;
        definition.Settings.AllowDemandStart = true;
        definition.Settings.ExecutionTimeLimit = "PT0S";
        definition.Settings.MultipleInstances = 0; // Ignore new

        dynamic trigger = definition.Triggers.Create(TaskTriggerLogon);
        trigger.UserId = WindowsIdentity.GetCurrent().Name;

        dynamic action = definition.Actions.Create(TaskActionExec);
        action.Path = Application.ExecutablePath;
        action.WorkingDirectory = AppContext.BaseDirectory;

        folder.RegisterTaskDefinition(
            TaskName,
            definition,
            TaskCreateOrUpdate,
            null,
            null,
            TaskLogonInteractiveToken);
    }

    public static void SetEnabled(bool enabled)
    {
        if (!Exists())
        {
            if (enabled)
            {
                EnsureRegistered(enableAtLogon: true);
            }

            return;
        }

        dynamic task = GetTask()!;
        task.Enabled = enabled;
    }

    public static void Run()
    {
        dynamic task = GetTask() ?? throw new InvalidOperationException("CapsLang startup task is missing.");
        task.Run(null);
    }

    private static dynamic Connect()
    {
        var serviceType = Type.GetTypeFromProgID("Schedule.Service")
            ?? throw new InvalidOperationException("Task Scheduler COM service is unavailable.");
        dynamic service = Activator.CreateInstance(serviceType)
            ?? throw new InvalidOperationException("Could not create Task Scheduler service.");
        service.Connect();
        return service;
    }

    private static dynamic GetOrCreateFolder(dynamic service)
    {
        dynamic root = service.GetFolder("\\");
        try
        {
            return root.GetFolder(FolderName);
        }
        catch (Exception)
        {
            return root.CreateFolder(FolderName);
        }
    }

    private static dynamic? GetTask()
    {
        var service = Connect();
        try
        {
            dynamic folder = service.GetFolder("\\" + FolderName);
            return folder.GetTask(TaskName);
        }
        catch (Exception)
        {
            return null;
        }
    }
}

/// 开机自启动管理（通过 Windows 注册表 HKCU Run 键）
/// 不依赖任何额外 crate，直接使用 windows-rs
use log::info;
use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_SZ,
};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "InsightFlow";

/// 将字符串转成以 null 结尾的 UTF-16 Vec
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 获取当前可执行文件路径（带引号，支持路径中含空格）
fn current_exe_quoted() -> Option<String> {
    let path = std::env::current_exe().ok()?;
    Some(format!("\"{}\"", path.to_string_lossy()))
}

/// 设置开机自启动（写入注册表）
pub fn enable_autostart() -> Result<(), String> {
    let exe = current_exe_quoted().ok_or("无法获取可执行路径")?;
    let key_wide = to_wide(RUN_KEY);
    let app_wide = to_wide(APP_NAME);
    let exe_wide: Vec<u16> = exe.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let mut hkey = windows::Win32::System::Registry::HKEY::default();
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_wide.as_ptr()),
            Some(0),
            KEY_SET_VALUE,
            &mut hkey,
        );
        if result.is_err() {
            return Err(format!("无法打开注册表键: {:?}", result));
        }

        let data = exe_wide.as_ptr() as *const u8;
        let data_len = exe_wide.len() * 2; // UTF-16 每字符 2 字节

        let result = RegSetValueExW(
            hkey,
            PCWSTR(app_wide.as_ptr()),
            Some(0),
            REG_SZ,
            Some(std::slice::from_raw_parts(data, data_len)),
        );
        let _ = RegCloseKey(hkey);

        if result.is_err() {
            return Err(format!("写入注册表失败: {:?}", result));
        }
    }

    info!("Autostart enabled: {exe}");
    Ok(())
}

/// 禁用开机自启动（删除注册表项）
pub fn disable_autostart() -> Result<(), String> {
    let key_wide = to_wide(RUN_KEY);
    let app_wide = to_wide(APP_NAME);

    unsafe {
        let mut hkey = windows::Win32::System::Registry::HKEY::default();
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_wide.as_ptr()),
            Some(0),
            KEY_SET_VALUE,
            &mut hkey,
        );
        if result.is_err() {
            return Err(format!("无法打开注册表键: {:?}", result));
        }

        let result = RegDeleteValueW(hkey, PCWSTR(app_wide.as_ptr()));
        let _ = RegCloseKey(hkey);

        // 若键不存在（ERROR_FILE_NOT_FOUND = 0x2）视为成功
        if result.is_err() && result.0 != 0x2 {
            return Err(format!("删除注册表项失败: {:?}", result));
        }
    }

    info!("Autostart disabled");
    Ok(())
}

/// 查询当前自启动状态
pub fn is_autostart_enabled() -> bool {
    let key_wide = to_wide(RUN_KEY);
    let app_wide = to_wide(APP_NAME);

    unsafe {
        let mut hkey = windows::Win32::System::Registry::HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_wide.as_ptr()),
            Some(0),
            KEY_READ,
            &mut hkey,
        )
        .is_err()
        {
            return false;
        }

        let mut data_type = REG_SZ;
        let mut data_len = 0u32;
        let exists = RegQueryValueExW(
            hkey,
            PCWSTR(app_wide.as_ptr()),
            None,
            Some(&mut data_type),
            None,
            Some(&mut data_len),
        )
        .is_ok();

        let _ = RegCloseKey(hkey);
        exists
    }
}

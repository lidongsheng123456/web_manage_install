/// 检查当前进程是否以管理员身份运行。
pub fn is_elevated() -> bool {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    hklm.open_subkey_with_flags(
        r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
        KEY_SET_VALUE,
    )
    .is_ok()
}

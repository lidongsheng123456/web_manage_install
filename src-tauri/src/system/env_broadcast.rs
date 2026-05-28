/// 广播 WM_SETTINGCHANGE 消息通知其他进程刷新环境变量。
///
/// Windows 修改注册表环境变量后需要发送该广播，资源管理器和新进程
/// 才能尽快读取最新值。
pub fn broadcast_env_change() {
    #[cfg(windows)]
    {
        #[link(name = "user32")]
        extern "system" {
            fn SendMessageTimeoutA(
                hwnd: isize,
                msg: u32,
                wparam: usize,
                lparam: *const u8,
                flags: u32,
                timeout: u32,
                result: *mut usize,
            ) -> isize;
        }

        const HWND_BROADCAST: isize = 0xFFFF;
        const WM_SETTINGCHANGE: u32 = 0x001A;
        const SMTO_ABORTIFHUNG: u32 = 0x0002;

        let param = b"Environment\0";
        let mut result: usize = 0;
        unsafe {
            SendMessageTimeoutA(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                param.as_ptr(),
                SMTO_ABORTIFHUNG,
                5000,
                &mut result,
            );
        }
    }
}

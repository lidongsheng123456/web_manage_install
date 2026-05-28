use crate::system::env_config;

/// 配置 MYSQL_HOME 和 PATH 环境变量。
pub fn configure_env_vars(mysql_home: &str) -> Result<(), String> {
    let mysql_bin = format!("{mysql_home}\\bin");
    env_config::set_system_env("MYSQL_HOME", mysql_home)?;
    env_config::append_to_path(&mysql_bin)
}

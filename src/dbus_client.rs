use zbus::proxy;

#[proxy(
    interface = "org.aged.Daemon",
    default_service = "org.aged.Daemon",
    default_path = "/org/aged/Daemon"
)]
pub trait AgedDaemon {
    async fn set_birthdate(&self, date_str: &str) -> zbus::Result<()>;
    async fn get_age_bracket(&self, jurisdiction: &str) -> zbus::Result<String>;
    async fn list_jurisdictions(&self) -> zbus::Result<Vec<String>>;
    async fn get_default_jurisdiction(&self) -> zbus::Result<String>;
    async fn set_default_jurisdiction(&self, name: &str) -> zbus::Result<()>;
    async fn detect_jurisdiction(&self) -> zbus::Result<String>;
}

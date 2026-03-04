use zbus::Connection;
use zbus::proxy;
use zbus::zvariant::OwnedObjectPath;

use crate::error::Error;
use crate::jurisdiction::JurisdictionRegistry;

#[proxy(
    interface = "org.freedesktop.GeoClue2.Manager",
    default_service = "org.freedesktop.GeoClue2",
    default_path = "/org/freedesktop/GeoClue2/Manager"
)]
trait GeoClueManager {
    async fn get_client(&self) -> zbus::Result<OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.GeoClue2.Client",
    default_service = "org.freedesktop.GeoClue2"
)]
trait GeoClueClient {
    #[zbus(property)]
    fn set_desktop_id(&self, id: &str) -> zbus::Result<()>;

    #[zbus(property)]
    fn set_requested_accuracy_level(&self, level: u32) -> zbus::Result<()>;

    #[zbus(property)]
    fn location(&self) -> zbus::Result<OwnedObjectPath>;

    async fn start(&self) -> zbus::Result<()>;
    async fn stop(&self) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.GeoClue2.Location",
    default_service = "org.freedesktop.GeoClue2"
)]
trait GeoClueLocation {
    #[zbus(property)]
    fn latitude(&self) -> zbus::Result<f64>;

    #[zbus(property)]
    fn longitude(&self) -> zbus::Result<f64>;
}

pub async fn detect_jurisdiction(
    registry: &JurisdictionRegistry,
) -> Result<String, Error> {
    let connection = Connection::system()
        .await
        .map_err(|e| Error::GeoClue(format!("connect to system bus: {e}")))?;

    let manager = GeoClueManagerProxy::new(&connection)
        .await
        .map_err(|e| Error::GeoClue(format!("create manager proxy: {e}")))?;

    let client_path = manager
        .get_client()
        .await
        .map_err(|e| Error::GeoClue(format!("get client: {e}")))?;

    let client = GeoClueClientProxy::builder(&connection)
        .path(client_path)
        .map_err(|e| Error::GeoClue(format!("build client proxy: {e}")))?
        .build()
        .await
        .map_err(|e| Error::GeoClue(format!("create client proxy: {e}")))?;

    client
        .set_desktop_id("org.aged.Daemon")
        .await
        .map_err(|e| Error::GeoClue(format!("set desktop id: {e}")))?;

    // Request city-level accuracy (4)
    client
        .set_requested_accuracy_level(4)
        .await
        .map_err(|e| Error::GeoClue(format!("set accuracy level: {e}")))?;

    client
        .start()
        .await
        .map_err(|e| Error::GeoClue(format!("start client: {e}")))?;

    // Brief delay to let GeoClue acquire a location
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let location_path = client
        .location()
        .await
        .map_err(|e| Error::GeoClue(format!("get location: {e}")))?;

    let location = GeoClueLocationProxy::builder(&connection)
        .path(location_path)
        .map_err(|e| Error::GeoClue(format!("build location proxy: {e}")))?
        .build()
        .await
        .map_err(|e| Error::GeoClue(format!("create location proxy: {e}")))?;

    let lat = location
        .latitude()
        .await
        .map_err(|e| Error::GeoClue(format!("get latitude: {e}")))?;
    let lon = location
        .longitude()
        .await
        .map_err(|e| Error::GeoClue(format!("get longitude: {e}")))?;

    let _ = client.stop().await;

    tracing::debug!(lat, lon, "got location from GeoClue2");

    let finder = tzf_rs::DefaultFinder::new();
    let tz = finder.get_tz_name(lon, lat);
    tracing::debug!(timezone = tz, "resolved timezone");

    registry
        .find_by_timezone(tz)
        .ok_or_else(|| Error::GeoClue(format!("no jurisdiction for timezone: {tz}")))
}

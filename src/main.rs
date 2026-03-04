mod age;
mod config;
mod dbus_client;
mod dbus_service;
mod error;
#[cfg(feature = "geoclue")]
mod geoclue;
mod jurisdiction;
mod storage;

use std::sync::Arc;

use clap::{Parser, Subcommand};
use zbus::connection::Builder;

use crate::config::Config;
use crate::dbus_client::AgedDaemonProxy;
use crate::dbus_service::AgedDbus;
use crate::jurisdiction::JurisdictionRegistry;

#[derive(Parser)]
#[command(name = "aged", about = "Age Bracket Verification Daemon")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the D-Bus daemon (foreground)
    Daemon,

    /// Store birthdate (ISO 8601: YYYY-MM-DD)
    SetBirthdate {
        /// Birthdate in YYYY-MM-DD format
        date: String,
    },

    /// Query age bracket
    GetAgeBracket {
        /// Jurisdiction name (uses default if omitted)
        #[arg(long, default_value = "")]
        jurisdiction: String,
    },

    /// List configured jurisdictions
    ListJurisdictions,

    /// Set default jurisdiction
    SetDefaultJurisdiction {
        /// Jurisdiction name
        name: String,
    },

    /// Detect jurisdiction from location
    DetectJurisdiction,
}

fn load_jurisdictions(config: &Config) -> JurisdictionRegistry {
    let mut registry = JurisdictionRegistry::new();
    registry.load_builtin();

    for path in &config.jurisdictions.extra_paths {
        if let Err(e) = registry.load_file(path) {
            tracing::warn!(?path, "failed to load extra jurisdiction file: {e}");
        }
    }

    // Also try loading from XDG config
    if let Some(dirs) = directories::ProjectDirs::from("org", "aged", "aged") {
        let user_jurisdictions = dirs.config_dir().join("jurisdictions.toml");
        if user_jurisdictions.exists() {
            if let Err(e) = registry.load_file(&user_jurisdictions) {
                tracing::warn!("failed to load user jurisdictions: {e}");
            }
        }
    }

    registry
}

fn config_path() -> std::path::PathBuf {
    directories::ProjectDirs::from("org", "aged", "aged")
        .map(|d| d.config_dir().join("config.toml"))
        .unwrap_or_else(|| std::path::PathBuf::from(".config/aged/config.toml"))
}

async fn run_daemon(config: Config) -> anyhow::Result<()> {
    let jurisdictions = Arc::new(load_jurisdictions(&config));
    let storage = Arc::from(storage::create_storage(&config).await);

    let service = AgedDbus {
        storage,
        jurisdictions,
    };

    let _conn = Builder::session()?
        .name("org.aged.Daemon")?
        .serve_at("/org/aged/Daemon", service)?
        .build()
        .await?;

    tracing::info!("daemon started, acquired org.aged.Daemon on session bus");

    #[cfg(feature = "systemd")]
    {
        let _ = sd_notify::notify(true, &[sd_notify::NotifyState::Ready]);
    }

    // Wait forever (until signal)
    std::future::pending::<()>().await;
    Ok(())
}

async fn try_dbus_client() -> Option<AgedDaemonProxy<'static>> {
    let connection = zbus::Connection::session().await.ok()?;
    let proxy = AgedDaemonProxy::new(&connection).await.ok()?;
    // Check if the daemon is actually running by pinging it
    proxy.list_jurisdictions().await.ok()?;
    Some(proxy)
}

async fn run_cli(command: Command) -> anyhow::Result<()> {
    match command {
        Command::Daemon => unreachable!(),

        Command::SetBirthdate { date } => {
            if let Some(proxy) = try_dbus_client().await {
                proxy.set_birthdate(&date).await?;
            } else {
                tracing::debug!("daemon not running, using direct storage");
                let config = Config::load(&config_path())?;
                let storage = storage::create_storage(&config).await;
                let parsed = date
                    .parse::<chrono::NaiveDate>()
                    .map_err(|_| error::Error::InvalidDate(date))?;
                storage.store_birthdate(parsed).await?;
            }
            println!("Birthdate stored.");
        }

        Command::GetAgeBracket { jurisdiction } => {
            if let Some(proxy) = try_dbus_client().await {
                let bracket = proxy.get_age_bracket(&jurisdiction).await?;
                println!("{bracket}");
            } else {
                tracing::debug!("daemon not running, using direct storage");
                let config = Config::load(&config_path())?;
                let storage = storage::create_storage(&config).await;
                let jurisdictions = load_jurisdictions(&config);

                let birthdate = storage
                    .load_birthdate()
                    .await?
                    .ok_or(error::Error::NoBirthdate)?;

                let jurisdiction_name = if jurisdiction.is_empty() {
                    storage
                        .load_default_jurisdiction()
                        .await?
                        .ok_or(error::Error::NoDefaultJurisdiction)?
                } else {
                    jurisdiction
                };

                let user_age = age::calculate_age(birthdate);
                let bracket = jurisdictions.lookup_bracket(&jurisdiction_name, user_age)?;
                println!("{bracket}");
            }
        }

        Command::ListJurisdictions => {
            if let Some(proxy) = try_dbus_client().await {
                let names = proxy.list_jurisdictions().await?;
                for name in names {
                    println!("{name}");
                }
            } else {
                let config = Config::load(&config_path())?;
                let jurisdictions = load_jurisdictions(&config);
                for name in jurisdictions.list_names() {
                    println!("{name}");
                }
            }
        }

        Command::SetDefaultJurisdiction { name } => {
            if let Some(proxy) = try_dbus_client().await {
                proxy.set_default_jurisdiction(&name).await?;
            } else {
                let config = Config::load(&config_path())?;
                let jurisdictions = load_jurisdictions(&config);
                if jurisdictions.get(&name).is_none() {
                    anyhow::bail!("unknown jurisdiction: {name}");
                }
                let storage = storage::create_storage(&config).await;
                storage.store_default_jurisdiction(&name).await?;
            }
            println!("Default jurisdiction set to: {name}");
        }

        Command::DetectJurisdiction => {
            if let Some(proxy) = try_dbus_client().await {
                let name = proxy.detect_jurisdiction().await?;
                println!("{name}");
            } else {
                #[cfg(feature = "geoclue")]
                {
                    let config = Config::load(&config_path())?;
                    let jurisdictions = load_jurisdictions(&config);
                    let name = geoclue::detect_jurisdiction(&jurisdictions).await?;
                    println!("{name}");
                }
                #[cfg(not(feature = "geoclue"))]
                {
                    anyhow::bail!("geoclue feature not enabled");
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Daemon => {
            let config = Config::load(&config_path())?;
            run_daemon(config).await
        }
        cmd => run_cli(cmd).await,
    }
}

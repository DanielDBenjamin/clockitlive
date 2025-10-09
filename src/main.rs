#[cfg(feature = "ssr")]
use leptos::logging::log;

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PhoneHintOrigin {
    Autodetected,
    Override,
    Fallback,
}

#[cfg(feature = "ssr")]
fn detect_lan_ipv4() -> Option<std::net::Ipv4Addr> {
    use std::net::{IpAddr, UdpSocket};

    const PROBES: [&str; 2] = ["1.1.1.1:80", "8.8.8.8:80"];

    for probe in PROBES {
        let Ok(socket) = UdpSocket::bind("0.0.0.0:0") else {
            continue;
        };

        if socket.connect(probe).is_ok() {
            if let Ok(addr) = socket.local_addr() {
                match addr.ip() {
                    IpAddr::V4(ip) if !ip.is_loopback() && !ip.is_unspecified() => return Some(ip),
                    _ => continue,
                }
            }
        }
    }

    None
}

#[cfg(feature = "ssr")]
fn sanitize_host(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let without_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed);

    let host = without_scheme.trim_matches('/');
    if host.is_empty() {
        None
    } else {
        Some(host.to_string())
    }
}

#[cfg(feature = "ssr")]
fn phone_access_hint(port: u16, scheme: &str) -> (String, PhoneHintOrigin) {
    use std::env;

    if let Ok(raw) = env::var("CLOCK_IT_LAN_IP") {
        if let Some(host) = sanitize_host(&raw) {
            return (
                format!("{}://{}:{}", scheme, host, port),
                PhoneHintOrigin::Override,
            );
        }
    }

    if let Some(ip) = detect_lan_ipv4() {
        return (
            format!("{}://{}:{}", scheme, ip, port),
            PhoneHintOrigin::Autodetected,
        );
    }

    (
        format!("{}://<your-lan-ip>:{}", scheme, port),
        PhoneHintOrigin::Fallback,
    )
}

#[cfg(feature = "ssr")]
fn resolve_use_tls() -> bool {
    use std::env;

    match env::var("CLOCK_IT_USE_TLS") {
        Ok(raw) => match raw.trim() {
            "" => true,
            value => match value.to_ascii_lowercase().as_str() {
                "0" | "false" | "no" | "off" => false,
                "1" | "true" | "yes" | "on" => true,
                _ => {
                    log!(
                        "⚠️  Unrecognized CLOCK_IT_USE_TLS value '{}'; defaulting to TLS enabled.",
                        raw
                    );
                    true
                }
            },
        },
        Err(_) => true,
    }
}

#[cfg(feature = "ssr")]
fn resolve_port(use_tls: bool) -> u16 {
    use std::env;

    let default_port = if use_tls { 3443 } else { 3000 };

    if let Ok(raw) = env::var("CLOCK_IT_PORT") {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return default_port;
        }

        match trimmed.parse::<u16>() {
            Ok(port) if port != 0 => port,
            _ => {
                log!(
                    "⚠️  Invalid CLOCK_IT_PORT '{}'; using default {} instead.",
                    raw,
                    default_port
                );
                default_port
            }
        }
    } else {
        default_port
    }
}

#[cfg(feature = "ssr")]
fn server_scheme(use_tls: bool) -> &'static str {
    if use_tls {
        "https"
    } else {
        "http"
    }
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use axum_server::tls_rustls::RustlsConfig;
    use clock_it::app::*;
    use clock_it::database::{
        init_db_pool, run_migrations, test_database_structure, test_db_connection,
    };
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use tokio::net::TcpListener;

     // Load environment variables from .env file
    if let Err(e) = dotenvy::dotenv() {
        println!("⚠️  Could not load .env file: {}", e);
    } else {
        println!("✅ Loaded .env file successfully");
    }

    println!("🚀 Starting Clock-It server...");

    clock_it::database::print_test_hash();

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;

    // Initialize database
    println!("🗄️ Initializing database...");
    let _pool = match init_db_pool().await {
        Ok(pool) => {
            if let Err(e) = test_db_connection(&pool).await {
                eprintln!("❌ Database connection test failed: {}", e);
                std::process::exit(1);
            }

            if let Err(e) = run_migrations(&pool).await {
                eprintln!("❌ Failed to run database migrations: {}", e);
                std::process::exit(1);
            }

            if let Err(e) = test_database_structure(&pool).await {
                eprintln!("❌ Database structure test failed: {}", e);
                std::process::exit(1);
            }

            pool
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    // Generate routes
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let use_tls = resolve_use_tls();
    let port = resolve_port(use_tls);
    let scheme = server_scheme(use_tls);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    let (phone_url, phone_hint_origin) = phone_access_hint(port, scheme);

    log!("🌐 Server listening on {}://0.0.0.0:{}", scheme, port);
    log!("📱 Access from phone: {}", phone_url);
    match phone_hint_origin {
        PhoneHintOrigin::Fallback => {
            log!(
                "ℹ️  Unable to auto-detect your LAN IP. Set CLOCK_IT_LAN_IP or replace <your-lan-ip> above."
            );
        }
        PhoneHintOrigin::Override => {
            log!("ℹ️  Using CLOCK_IT_LAN_IP override for phone access hint.");
        }
        PhoneHintOrigin::Autodetected => {}
    }
    log!("💻 Access from laptop: {}://localhost:{}", scheme, port);

    if use_tls {
        let config = RustlsConfig::from_pem_file("certs/cert.pem", "certs/key.pem")
            .await
            .expect("Failed to load TLS certificates");

        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service())
            .await
            .unwrap();
    } else {
        log!("⚠️  TLS disabled; serving plain HTTP (development use only).");

        let listener = TcpListener::bind(addr)
            .await
            .expect("Failed to bind HTTP listener");

        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    }
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
}

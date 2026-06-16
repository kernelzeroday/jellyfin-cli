mod api;
mod config;
mod display;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "jf", about = "Jellyfin CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Authenticate via Quick Connect (or --token for direct API key)
    Login {
        /// Use an API key directly instead of Quick Connect
        #[arg(long)]
        token: Option<String>,
        /// Server URL override
        #[arg(long)]
        server: Option<String>,
    },
    /// Show current auth status and server info
    Status,
    /// List media libraries
    Libraries,
    /// Browse items in a library
    #[command(alias = "ls")]
    List {
        /// Library name or ID (omit to list all recent items)
        library: Option<String>,
        /// Item type filter: movie, series, episode, audio, album, artist
        #[arg(short = 't', long)]
        r#type: Option<String>,
        /// Max results
        #[arg(short, long, default_value = "30")]
        limit: u32,
        /// Sort by: name, date, rating, random
        #[arg(short, long, default_value = "name")]
        sort: String,
    },
    /// Search across all libraries
    Search {
        query: Vec<String>,
        /// Max results
        #[arg(short, long, default_value = "20")]
        limit: u32,
    },
    /// Show details for an item by ID or name
    Info {
        /// Item ID or name (searches if not a valid ID)
        item: Vec<String>,
    },
    /// List seasons/episodes for a series
    Episodes {
        /// Series ID or name
        series: Vec<String>,
        /// Season number (omit to list all seasons)
        #[arg(short, long)]
        season: Option<u32>,
    },
    /// List users
    Users,
    /// Show server system info
    #[command(alias = "sys")]
    System,
    /// Get a streaming/download URL for an item
    Url {
        /// Item ID or name
        item: Vec<String>,
    },
    /// Show recently added items
    Recent {
        /// Max results
        #[arg(short, long, default_value = "20")]
        limit: u32,
    },
    /// Mark an item as played/favorite
    Mark {
        /// Item ID or name
        item: Vec<String>,
        /// Mark type
        #[arg(short, long, value_parser = ["played", "unplayed", "favorite", "unfavorite"])]
        r#as: String,
    },
    /// List active client sessions (devices you can remote-control)
    Sessions,
    /// Cast an item to a device (auto-picks the only controllable device)
    Cast {
        /// Item ID or name
        item: Vec<String>,
        /// Target device name (substring match); omit to auto-pick
        #[arg(long)]
        to: Option<String>,
    },
    /// Play an item locally with mpv
    Play {
        /// Item ID or name
        item: Vec<String>,
        /// Extra arguments passed to mpv
        #[arg(last = true)]
        mpv_args: Vec<String>,
    },
    /// Remote-control playback on a device
    Remote {
        /// Action to send
        #[arg(value_parser = ["pause", "stop", "next", "prev", "seek"])]
        action: String,
        /// Target device name (substring match); omit to auto-pick
        #[arg(long)]
        to: Option<String>,
        /// Seek position in seconds (required for `seek`)
        #[arg(long)]
        position: Option<u64>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Login { token, server } => cmd_login(token, server).await,
        Command::Status => cmd_status().await,
        Command::Libraries => cmd_libraries().await,
        Command::List { library, r#type, limit, sort } => {
            cmd_list(library, r#type, limit, sort).await
        }
        Command::Search { query, limit } => cmd_search(query.join(" "), limit).await,
        Command::Info { item } => cmd_info(item.join(" ")).await,
        Command::Episodes { series, season } => cmd_episodes(series.join(" "), season).await,
        Command::Users => cmd_users().await,
        Command::System => cmd_system().await,
        Command::Url { item } => cmd_url(item.join(" ")).await,
        Command::Recent { limit } => cmd_recent(limit).await,
        Command::Mark { item, r#as } => cmd_mark(item.join(" "), r#as).await,
        Command::Sessions => cmd_sessions().await,
        Command::Cast { item, to } => cmd_cast(item.join(" "), to).await,
        Command::Play { item, mpv_args } => cmd_play(item.join(" "), mpv_args).await,
        Command::Remote { action, to, position } => cmd_remote(action, to, position).await,
    }
}

async fn cmd_login(token: Option<String>, server: Option<String>) -> Result<()> {
    let mut cfg = config::Config::load()?;

    if let Some(url) = server {
        cfg.server_url = url;
        cfg.save()?;
    }

    if let Some(api_key) = token {
        cfg.access_token = Some(api_key.clone());
        let client = api::Client::new(&cfg)?;
        match client.me().await {
            Ok(user) => {
                cfg.user_id = Some(user.id.clone());
                cfg.user_name = Some(user.name.clone());
                cfg.save()?;
                println!(
                    "Authenticated as {}",
                    colored::Colorize::green(user.name.as_str())
                );
            }
            Err(_) => {
                cfg.user_id = None;
                cfg.user_name = Some("api-key".into());
                cfg.save()?;
                println!("Token saved (could not resolve user — may be a server API key)");
            }
        }
        return Ok(());
    }

    let client = api::Client::new_unauthenticated(&cfg.server_url);
    println!("Initiating Quick Connect...");
    let qc = client.quick_connect_initiate().await?;
    println!(
        "\n  Enter code {} in Jellyfin web UI → Settings → Quick Connect\n",
        colored::Colorize::bold(colored::Colorize::cyan(qc.code.as_str()))
    );

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        let status = client.quick_connect_status(&qc.secret).await?;
        if status.authenticated {
            let auth = client.authenticate_quick_connect(&qc.secret).await?;
            cfg.access_token = Some(auth.access_token.clone());
            cfg.user_id = Some(auth.user.id.clone());
            cfg.user_name = Some(auth.user.name.clone());
            cfg.save()?;
            println!(
                "Authenticated as {}",
                colored::Colorize::green(auth.user.name.as_str())
            );
            return Ok(());
        }
        eprint!(".");
    }
}

async fn cmd_status() -> Result<()> {
    let cfg = config::Config::load()?;
    println!("Server: {}", colored::Colorize::cyan(cfg.server_url.as_str()));
    match &cfg.access_token {
        Some(_) => {
            let client = api::Client::new(&cfg)?;
            match client.system_info().await {
                Ok(info) => {
                    println!("Connected: {} v{}", info.server_name, info.version);
                    println!(
                        "User: {}",
                        colored::Colorize::green(
                            cfg.user_name.as_deref().unwrap_or("unknown")
                        )
                    );
                }
                Err(_) => {
                    println!(
                        "Status: {} (token may be expired, run `jf login`)",
                        colored::Colorize::red("auth failed")
                    );
                }
            }
        }
        None => println!(
            "Status: {} — run `jf login`",
            colored::Colorize::yellow("not authenticated")
        ),
    }
    Ok(())
}

async fn cmd_libraries() -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let libs = client.libraries().await?;
    display::print_libraries(&libs);
    Ok(())
}

async fn cmd_list(
    library: Option<String>,
    item_type: Option<String>,
    limit: u32,
    sort: String,
) -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;

    let parent_id = match library {
        Some(ref name) => Some(client.resolve_library_id(name).await?),
        None => None,
    };

    let jf_type = item_type.as_deref().map(api::normalize_item_type);
    let sort_by = match sort.to_lowercase().as_str() {
        "date" => "DateCreated",
        "rating" => "CommunityRating",
        "random" => "Random",
        _ => "SortName",
    };

    let items = client
        .items(parent_id.as_deref(), jf_type.as_deref(), limit, sort_by)
        .await?;
    display::print_items(&items.items, &cfg.server_url);
    if items.total_record_count > limit {
        println!(
            "\n  {} of {} items shown",
            colored::Colorize::dimmed(format!("{}", limit).as_str()),
            items.total_record_count
        );
    }
    Ok(())
}

async fn cmd_search(query: String, limit: u32) -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let hints = client.search(&query, limit).await?;
    display::print_search_results(&hints, &cfg.server_url);
    Ok(())
}

async fn cmd_info(item: String) -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let id = client.resolve_item_id(&item).await?;
    let item = client.item_detail(&id).await?;
    display::print_item_detail(&item, &cfg.server_url);
    Ok(())
}

async fn cmd_episodes(series: String, season: Option<u32>) -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let id = client.resolve_item_id(&series).await?;

    if let Some(season_num) = season {
        let episodes = client.episodes(&id, Some(season_num)).await?;
        display::print_items(&episodes.items, &cfg.server_url);
    } else {
        let seasons = client.seasons(&id).await?;
        display::print_items(&seasons.items, &cfg.server_url);
    }
    Ok(())
}

async fn cmd_users() -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let users = client.users().await?;
    display::print_users(&users);
    Ok(())
}

async fn cmd_system() -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let info = client.system_info().await?;
    display::print_system_info(&info);
    Ok(())
}

async fn cmd_url(item: String) -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let id = client.resolve_item_id(&item).await?;
    let detail = client.item_detail(&id).await?;

    let token = cfg.access_token.as_deref().unwrap_or("");
    let stream_url = format!(
        "{}/Items/{}/Download?api_key={}",
        cfg.server_url, id, token
    );
    println!("{}", colored::Colorize::bold("Download URL:"));
    println!("  {}", stream_url);

    if detail.item_type == "Episode" || detail.item_type == "Movie" {
        let play_url = format!(
            "{}/Videos/{}/stream?Static=true&api_key={}",
            cfg.server_url, id, token
        );
        println!("{}", colored::Colorize::bold("Stream URL:"));
        println!("  {}", play_url);
    }
    Ok(())
}

async fn cmd_recent(limit: u32) -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let items = client.latest_items(limit).await?;
    display::print_items(&items, &cfg.server_url);
    Ok(())
}

async fn cmd_mark(item: String, mark_as: String) -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let id = client.resolve_item_id(&item).await?;

    match mark_as.as_str() {
        "played" => client.mark_played(&id).await?,
        "unplayed" => client.mark_unplayed(&id).await?,
        "favorite" => client.set_favorite(&id, true).await?,
        "unfavorite" => client.set_favorite(&id, false).await?,
        _ => unreachable!(),
    }
    println!("Marked {} as {}", id, colored::Colorize::green(mark_as.as_str()));
    Ok(())
}

async fn cmd_sessions() -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let sessions = client.sessions().await?;
    display::print_sessions(&sessions);
    Ok(())
}

async fn cmd_cast(item: String, to: Option<String>) -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let id = client.resolve_item_id(&item).await?;
    let detail = client.item_detail(&id).await?;
    let session = client.resolve_session(to.as_deref()).await?;
    client.play_on_session(&session.id, &id, "PlayNow").await?;

    let device = session
        .device_name
        .as_deref()
        .or(session.client.as_deref())
        .unwrap_or("device");
    println!(
        "Casting {} to {}",
        colored::Colorize::bold(detail.name.as_str()),
        colored::Colorize::green(device)
    );
    Ok(())
}

async fn cmd_play(item: String, mpv_args: Vec<String>) -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let id = client.resolve_item_id(&item).await?;
    let detail = client.item_detail(&id).await?;

    let token = cfg.access_token.as_deref().unwrap_or("");
    let url = match detail.item_type.as_str() {
        "Episode" | "Movie" => format!(
            "{}/Videos/{}/stream?Static=true&api_key={}",
            cfg.server_url, id, token
        ),
        "Audio" => format!(
            "{}/Audio/{}/stream?Static=true&api_key={}",
            cfg.server_url, id, token
        ),
        _ => format!(
            "{}/Items/{}/Download?api_key={}",
            cfg.server_url, id, token
        ),
    };

    let title = if let Some(ref series) = detail.series_name {
        let ep = detail.index_number.map(|e| format!("E{:02}", e)).unwrap_or_default();
        let sn = detail.parent_index_number.map(|s| format!("S{:02}", s)).unwrap_or_default();
        format!("{} {}{} - {}", series, sn, ep, detail.name)
    } else {
        detail.name.clone()
    };

    println!(
        "Playing {} with mpv...",
        colored::Colorize::bold(title.as_str())
    );

    let status = std::process::Command::new("mpv")
        .arg(&url)
        .arg(format!("--force-media-title={}", title))
        .args(&mpv_args)
        .status()
        .context("failed to launch mpv — is it installed?")?;

    if !status.success() {
        bail!("mpv exited with {}", status);
    }
    Ok(())
}

async fn cmd_remote(action: String, to: Option<String>, position: Option<u64>) -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let session = client.resolve_session(to.as_deref()).await?;

    let (command, ticks) = match action.as_str() {
        "pause" => ("PlayPause", None),
        "stop" => ("Stop", None),
        "next" => ("NextTrack", None),
        "prev" => ("PreviousTrack", None),
        "seek" => {
            let secs = position.context("--position <seconds> is required for seek")?;
            ("Seek", Some(secs * 10_000_000))
        }
        _ => unreachable!(),
    };

    client.playstate_command(&session.id, command, ticks).await?;

    let device = session
        .device_name
        .as_deref()
        .or(session.client.as_deref())
        .unwrap_or("device");
    println!(
        "Sent {} to {}",
        colored::Colorize::green(action.as_str()),
        device
    );
    Ok(())
}

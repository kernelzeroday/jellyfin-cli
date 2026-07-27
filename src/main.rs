mod api;
mod config;
mod display;
mod program;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
        /// Filter by year (auto-detected from trailing year in query)
        #[arg(short, long)]
        year: Option<u32>,
        /// Filter by type: movie, series, episode, audio, album, artist
        #[arg(short = 't', long)]
        r#type: Option<String>,
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
    /// Trigger a library scan (all libraries, or one by name/ID)
    Scan {
        /// Library name or ID (omit to scan all libraries)
        library: Option<String>,
    },
    /// Refresh metadata/images for a specific item
    Refresh {
        /// Item ID or name
        item: Vec<String>,
        /// Force a full metadata re-fetch instead of only missing fields
        #[arg(long)]
        metadata: bool,
        /// Force images to be re-downloaded
        #[arg(long)]
        images: bool,
    },
    /// Search external metadata providers to (re)identify an item
    Identify {
        /// Item ID or name
        item: Vec<String>,
        /// Apply the Nth result from the search (1-based)
        #[arg(long)]
        apply: Option<usize>,
        /// Also replace existing images when applying
        #[arg(long)]
        replace_images: bool,
        /// Override the title to search for
        #[arg(long)]
        name: Option<String>,
        /// Override the year to search for
        #[arg(long)]
        year: Option<u32>,
    },
    /// List scheduled server tasks and their status
    Tasks,
    /// Run a scheduled task now (e.g. library scan, plugin task)
    RunTask {
        /// Task name (substring match)
        name: Vec<String>,
    },
    /// List missing episodes for a series
    Missing {
        /// Series ID or name
        series: Vec<String>,
    },
    /// List, inspect, create, and program playlists
    Playlist {
        #[command(subcommand)]
        command: PlaylistCommand,
    },
}

#[derive(Subcommand)]
enum PlaylistCommand {
    /// List playlists visible to the current user
    List,
    /// Show the ordered items in a playlist
    Show {
        /// Playlist name or ID
        playlist: String,
    },
    /// Create a playlist from item names or IDs
    Create {
        /// New playlist name
        name: String,
        /// Item names or IDs (quote names containing spaces)
        #[arg(required = true)]
        items: Vec<String>,
        /// Make the playlist visible to other users
        #[arg(long)]
        public: bool,
    },
    /// Add items to the end of a playlist
    Add {
        /// Playlist name or ID
        playlist: String,
        /// Item names or IDs (quote names containing spaces)
        #[arg(required = true)]
        items: Vec<String>,
    },
    /// Replace a playlist's contents, preserving the playlist itself
    Replace {
        /// Playlist name or ID
        playlist: String,
        /// Item names or IDs (quote names containing spaces)
        #[arg(required = true)]
        items: Vec<String>,
    },
    /// Build or update long-form channels from a JSON programming config
    Program {
        /// Path to the programming config
        file: PathBuf,
        /// Resolve and preview the schedule without changing Jellyfin
        #[arg(long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Login { token, server } => cmd_login(token, server).await,
        Command::Status => cmd_status().await,
        Command::Libraries => cmd_libraries().await,
        Command::List {
            library,
            r#type,
            limit,
            sort,
        } => cmd_list(library, r#type, limit, sort).await,
        Command::Search {
            query,
            limit,
            year,
            r#type,
        } => cmd_search(query.join(" "), limit, year, r#type).await,
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
        Command::Remote {
            action,
            to,
            position,
        } => cmd_remote(action, to, position).await,
        Command::Scan { library } => cmd_scan(library).await,
        Command::Refresh {
            item,
            metadata,
            images,
        } => cmd_refresh(item.join(" "), metadata, images).await,
        Command::Identify {
            item,
            apply,
            replace_images,
            name,
            year,
        } => cmd_identify(item.join(" "), apply, replace_images, name, year).await,
        Command::Tasks => cmd_tasks().await,
        Command::RunTask { name } => cmd_run_task(name.join(" ")).await,
        Command::Missing { series } => cmd_missing(series.join(" ")).await,
        Command::Playlist { command } => cmd_playlist(command).await,
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
    println!(
        "Server: {}",
        colored::Colorize::cyan(cfg.server_url.as_str())
    );
    match &cfg.access_token {
        Some(_) => {
            let client = api::Client::new(&cfg)?;
            match client.system_info().await {
                Ok(info) => {
                    println!("Connected: {} v{}", info.server_name, info.version);
                    println!(
                        "User: {}",
                        colored::Colorize::green(cfg.user_name.as_deref().unwrap_or("unknown"))
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

async fn cmd_search(
    query: String,
    limit: u32,
    year: Option<u32>,
    item_type: Option<String>,
) -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;

    let (search_term, year_filter) = match year {
        Some(y) => (query, Some(y)),
        None => api::extract_trailing_year(&query),
    };

    let jf_type = item_type.as_deref().map(api::normalize_item_type);
    let api_limit = if jf_type.is_some() {
        limit.max(100)
    } else {
        // Mixed searches often return every matching episode before their
        // parent series, so leave enough room to rank top-level items first.
        limit.max(500)
    };

    let mut hints = client
        .search(&search_term, api_limit, jf_type.as_deref())
        .await?;
    api::sort_search_hints(&mut hints, &search_term);

    let filtered: Vec<_> = hints
        .into_iter()
        .filter(|h| match year_filter {
            Some(y) => h.production_year == Some(y),
            None => true,
        })
        .take(limit as usize)
        .collect();

    display::print_search_results(&filtered, &cfg.server_url);
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
    let id = client
        .resolve_item_id_of_type(&series, Some("Series"))
        .await?;

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
    let stream_url = format!("{}/Items/{}/Download?api_key={}", cfg.server_url, id, token);
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
    println!(
        "Marked {} as {}",
        id,
        colored::Colorize::green(mark_as.as_str())
    );
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

    if detail.item_type == "Series" {
        let episodes = client.episodes(&id, None).await?;
        if episodes.items.is_empty() {
            bail!("series has no episodes");
        }

        let start_index = episodes
            .items
            .iter()
            .position(|ep| ep.user_data.as_ref().map_or(true, |ud| !ud.played))
            .unwrap_or(0);

        let mut playlist = String::from("#EXTM3U\n");
        for ep in &episodes.items {
            let ep_title = format!(
                "{} S{:02}E{:02} - {}",
                detail.name,
                ep.parent_index_number.unwrap_or(0),
                ep.index_number.unwrap_or(0),
                ep.name
            );
            let duration = ep
                .run_time_ticks
                .map(|t| (t / 10_000_000) as i64)
                .unwrap_or(-1);
            let url = format!(
                "{}/Videos/{}/stream?Static=true&api_key={}",
                cfg.server_url, ep.id, token
            );
            playlist.push_str(&format!("#EXTINF:{},{}\n{}\n", duration, ep_title, url));
        }

        let playlist_path = std::env::temp_dir().join(format!("jf-{}.m3u", id));
        std::fs::write(&playlist_path, &playlist)?;

        let start_ep = &episodes.items[start_index];
        let start_title = format!(
            "{} S{:02}E{:02}",
            detail.name,
            start_ep.parent_index_number.unwrap_or(0),
            start_ep.index_number.unwrap_or(0),
        );
        println!(
            "Playing {} ({} episodes, starting at {}) with mpv...",
            colored::Colorize::bold(detail.name.as_str()),
            episodes.items.len(),
            colored::Colorize::bold(start_title.as_str()),
        );

        let status = std::process::Command::new("mpv")
            .arg(format!("--playlist={}", playlist_path.display()))
            .arg(format!("--playlist-start={}", start_index))
            .args(&mpv_args)
            .status()
            .context("failed to launch mpv — is it installed?")?;

        let _ = std::fs::remove_file(&playlist_path);

        if !status.success() {
            bail!("mpv exited with {}", status);
        }
        return Ok(());
    }

    let url = match detail.item_type.as_str() {
        "Episode" | "Movie" => format!(
            "{}/Videos/{}/stream?Static=true&api_key={}",
            cfg.server_url, id, token
        ),
        "Audio" => format!(
            "{}/Audio/{}/stream?Static=true&api_key={}",
            cfg.server_url, id, token
        ),
        _ => format!("{}/Items/{}/Download?api_key={}", cfg.server_url, id, token),
    };

    let title = if let Some(ref series) = detail.series_name {
        let ep = detail
            .index_number
            .map(|e| format!("E{:02}", e))
            .unwrap_or_default();
        let sn = detail
            .parent_index_number
            .map(|s| format!("S{:02}", s))
            .unwrap_or_default();
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

    client
        .playstate_command(&session.id, command, ticks)
        .await?;

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

async fn cmd_scan(library: Option<String>) -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;

    match library {
        Some(name) => {
            let id = client.resolve_library_id(&name).await?;
            client
                .refresh_item(&id, "Default", "Default", false, false)
                .await?;
            println!(
                "Triggered rescan for {}",
                colored::Colorize::green(name.as_str())
            );
        }
        None => {
            client.refresh_all_libraries().await?;
            println!(
                "Triggered rescan for {}",
                colored::Colorize::green("all libraries")
            );
        }
    }
    Ok(())
}

async fn cmd_refresh(item: String, metadata: bool, images: bool) -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let id = client.resolve_item_id(&item).await?;
    let detail = client.item_detail(&id).await?;

    let metadata_mode = if metadata { "FullRefresh" } else { "Default" };
    let image_mode = if images { "FullRefresh" } else { "Default" };
    client
        .refresh_item(&id, metadata_mode, image_mode, metadata, images)
        .await?;

    println!(
        "Queued refresh for {}",
        colored::Colorize::bold(detail.name.as_str())
    );
    Ok(())
}

async fn cmd_identify(
    item: String,
    apply: Option<usize>,
    replace_images: bool,
    name_override: Option<String>,
    year_override: Option<u32>,
) -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let id = client.resolve_item_id(&item).await?;
    let detail = client.item_detail(&id).await?;

    let search_type = api::remote_search_type(&detail.item_type)
        .with_context(|| format!("cannot identify items of type {}", detail.item_type))?;

    let name = name_override.unwrap_or_else(|| detail.name.clone());
    let year = year_override.or(detail.production_year);

    let results = client.remote_search(search_type, &id, &name, year).await?;
    if results.is_empty() {
        println!("{}", colored::Colorize::yellow("No matches found."));
        return Ok(());
    }

    if let Some(index) = apply {
        let chosen = results
            .get(index - 1)
            .with_context(|| format!("no result at index {}", index))?;
        client
            .apply_remote_search(&id, chosen, replace_images)
            .await?;
        let chosen_name = chosen
            .get("Name")
            .and_then(|v| v.as_str())
            .unwrap_or("match");
        println!(
            "Applied {} to {}",
            colored::Colorize::green(chosen_name),
            colored::Colorize::bold(detail.name.as_str())
        );
        return Ok(());
    }

    println!(
        "Matches for {}:",
        colored::Colorize::bold(detail.name.as_str())
    );
    for (i, result) in results.iter().enumerate() {
        let rname = result.get("Name").and_then(|v| v.as_str()).unwrap_or("?");
        let ryear = result
            .get("ProductionYear")
            .and_then(|v| v.as_u64())
            .map(|y| format!(" ({})", y))
            .unwrap_or_default();
        let providers = result
            .get("ProviderIds")
            .and_then(|v| v.as_object())
            .map(|m| {
                m.iter()
                    .map(|(k, v)| format!("{}:{}", k, v.as_str().unwrap_or("")))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        println!(
            "  {}. {}{}",
            i + 1,
            colored::Colorize::bold(rname),
            colored::Colorize::dimmed(ryear.as_str())
        );
        if !providers.is_empty() {
            println!("     {}", colored::Colorize::dimmed(providers.as_str()));
        }
    }
    println!(
        "\n  Run again with {} to apply a match",
        colored::Colorize::cyan("--apply <n>")
    );
    Ok(())
}

async fn cmd_tasks() -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let tasks = client.scheduled_tasks().await?;
    display::print_tasks(&tasks);
    Ok(())
}

async fn cmd_run_task(name: String) -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let id = client.resolve_task_id(&name).await?;
    client.start_task(&id).await?;
    println!("Started task {}", colored::Colorize::green(name.as_str()));
    Ok(())
}

async fn cmd_missing(series: String) -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;
    let id = client
        .resolve_item_id_of_type(&series, Some("Series"))
        .await?;
    let detail = client.item_detail(&id).await?;
    let episodes = client.episodes(&id, None).await?;

    let missing: Vec<_> = episodes
        .items
        .into_iter()
        .filter(|ep| ep.location_type.as_deref() == Some("Virtual"))
        .collect();

    if missing.is_empty() {
        println!(
            "No missing episodes for {}",
            colored::Colorize::bold(detail.name.as_str())
        );
        return Ok(());
    }

    println!(
        "Missing episodes for {} ({}):",
        colored::Colorize::bold(detail.name.as_str()),
        missing.len()
    );
    display::print_items(&missing, &cfg.server_url);
    Ok(())
}

async fn cmd_playlist(command: PlaylistCommand) -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::Client::new(&cfg)?;

    match command {
        PlaylistCommand::List => {
            let playlists = client.playlists().await?;
            if playlists.items.is_empty() {
                println!("{}", colored::Colorize::yellow("No playlists found."));
            } else {
                display::print_items(&playlists.items, &cfg.server_url);
            }
        }
        PlaylistCommand::Show { playlist } => {
            let id = client.resolve_playlist_id(&playlist).await?;
            let detail = client.item_detail(&id).await?;
            let items = client.playlist_items(&id).await?;
            println!(
                "{} ({} items, {:.1} hours)",
                colored::Colorize::bold(detail.name.as_str()),
                items.items.len(),
                program::ticks_to_hours(
                    items
                        .items
                        .iter()
                        .filter_map(|item| item.run_time_ticks)
                        .sum()
                )
            );
            display::print_items(&items.items, &cfg.server_url);
        }
        PlaylistCommand::Create {
            name,
            items,
            public,
        } => {
            if client
                .playlists()
                .await?
                .items
                .iter()
                .any(|playlist| playlist.name.eq_ignore_ascii_case(&name))
            {
                bail!(
                    "playlist '{}' already exists; use `jf playlist replace`",
                    name
                );
            }
            let ids = resolve_item_ids(&client, &items).await?;
            let id = client.create_playlist(&name, &ids, public).await?;
            println!(
                "Created {} with {} items ({})",
                colored::Colorize::green(name.as_str()),
                ids.len(),
                id
            );
        }
        PlaylistCommand::Add { playlist, items } => {
            let playlist_id = client.resolve_playlist_id(&playlist).await?;
            let ids = resolve_item_ids(&client, &items).await?;
            client.add_playlist_items(&playlist_id, &ids).await?;
            println!(
                "Added {} items to {}",
                ids.len(),
                colored::Colorize::green(playlist.as_str())
            );
        }
        PlaylistCommand::Replace { playlist, items } => {
            let playlist_id = client.resolve_playlist_id(&playlist).await?;
            let ids = resolve_item_ids(&client, &items).await?;
            client.replace_playlist_items(&playlist_id, &ids).await?;
            println!(
                "Replaced {} with {} items",
                colored::Colorize::green(playlist.as_str()),
                ids.len()
            );
        }
        PlaylistCommand::Program { file, dry_run } => {
            let config = program::ProgramConfig::load(&file)?;
            let mut channels = Vec::with_capacity(config.channels.len());

            println!(
                "Building {} channel schedules from {}...",
                config.channels.len(),
                file.display()
            );
            for spec in &config.channels {
                let channel = program::build_channel(&client, spec).await?;
                let hours = program::ticks_to_hours(channel.duration_ticks);
                println!(
                    "  {}: {} items, {} series, {} movies, {:.1} hours{}",
                    colored::Colorize::bold(channel.name.as_str()),
                    channel.items.len(),
                    channel.series_count,
                    channel.movie_count,
                    hours,
                    if hours + 0.01 < spec.target_hours {
                        " (library content exhausted before target)"
                    } else {
                        ""
                    }
                );
                channels.push(channel);
            }

            if dry_run {
                println!(
                    "{}",
                    colored::Colorize::yellow("Dry run: Jellyfin was not changed.")
                );
                return Ok(());
            }

            let existing = client.playlists().await?.items;
            for channel in channels {
                let item_ids = channel
                    .items
                    .iter()
                    .map(|item| item.id.clone())
                    .collect::<Vec<_>>();
                if let Some(playlist) = existing
                    .iter()
                    .find(|playlist| playlist.name.eq_ignore_ascii_case(&channel.name))
                {
                    client
                        .replace_playlist_items(&playlist.id, &item_ids)
                        .await?;
                    println!(
                        "Updated {} in place",
                        colored::Colorize::green(channel.name.as_str())
                    );
                } else {
                    let id = client
                        .create_playlist(&channel.name, &item_ids, false)
                        .await?;
                    println!(
                        "Created {} ({})",
                        colored::Colorize::green(channel.name.as_str()),
                        id
                    );
                }
            }
        }
    }

    Ok(())
}

async fn resolve_item_ids(client: &api::Client, items: &[String]) -> Result<Vec<String>> {
    let mut ids = Vec::with_capacity(items.len());
    for item in items {
        ids.push(
            client
                .resolve_item_id(item)
                .await
                .with_context(|| format!("could not resolve playlist item '{}'", item))?,
        );
    }
    Ok(ids)
}

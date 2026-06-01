use colored::Colorize;

use crate::api::{Item, SearchHint, SystemInfo, UserDto, VirtualFolder};

pub fn print_libraries(libs: &[VirtualFolder]) {
    for lib in libs {
        let kind = lib
            .collection_type
            .as_deref()
            .unwrap_or("unknown");
        println!(
            "  {} {} {}",
            lib.name.bold().cyan(),
            format!("[{}]", kind).dimmed(),
            lib.item_id.dimmed()
        );
        for loc in &lib.locations {
            println!("    {}", loc.dimmed());
        }
    }
}

pub fn print_items(items: &[Item], server_url: &str) {
    for item in items {
        let type_tag = format_type_tag(&item.item_type);
        let year = item
            .production_year
            .map(|y| format!(" ({})", y))
            .unwrap_or_default();
        let rating = item
            .community_rating
            .map(|r| format!(" {:.1}★", r).yellow().to_string())
            .unwrap_or_default();

        let mut line = format!(
            "  {} {}{}{}",
            type_tag,
            item.name.bold(),
            year.dimmed(),
            rating
        );

        if let Some(ref series) = item.series_name {
            let ep = format_episode_number(item.parent_index_number, item.index_number);
            line = format!(
                "  {} {} — {}{}",
                type_tag,
                series.cyan(),
                item.name.bold(),
                if ep.is_empty() {
                    String::new()
                } else {
                    format!(" {}", ep.dimmed())
                }
            );
        }

        if let Some(ref ud) = item.user_data {
            if ud.played {
                line.push_str(&" ✓".green().to_string());
            }
            if ud.is_favorite {
                line.push_str(&" ♥".red().to_string());
            }
        }

        let runtime = format_runtime(item.run_time_ticks);
        if !runtime.is_empty() {
            line.push_str(&format!(" {}", runtime.dimmed()));
        }

        println!("{}", line);

        let _ = server_url;
        println!("    {}", item.id.dimmed());
    }
}

pub fn print_search_results(hints: &[SearchHint], _server_url: &str) {
    if hints.is_empty() {
        println!("  {}", "No results found.".dimmed());
        return;
    }
    for hint in hints {
        let type_tag = format_type_tag(&hint.item_type);
        let year = hint
            .production_year
            .map(|y| format!(" ({})", y))
            .unwrap_or_default();
        let series = hint
            .series
            .as_deref()
            .map(|s| format!(" — {}", s).cyan().to_string())
            .unwrap_or_default();
        let runtime = format_runtime(hint.run_time_ticks);

        println!(
            "  {} {}{}{}{}",
            type_tag,
            hint.name.bold(),
            year.dimmed(),
            series,
            if runtime.is_empty() {
                String::new()
            } else {
                format!(" {}", runtime.dimmed())
            }
        );
        println!("    {}", hint.id.dimmed());
    }
}

pub fn print_item_detail(item: &Item, server_url: &str) {
    let type_tag = format_type_tag(&item.item_type);
    println!(
        "\n{} {}",
        type_tag,
        item.name.bold()
    );

    if let Some(ref series) = item.series_name {
        let ep = format_episode_number(item.parent_index_number, item.index_number);
        println!("  Series: {}  {}", series.cyan(), ep.dimmed());
    }

    if let Some(year) = item.production_year {
        print!("  Year: {}", year);
    }
    if let Some(ref rating) = item.official_rating {
        print!("  Rated: {}", rating);
    }
    if let Some(r) = item.community_rating {
        print!("  Rating: {}", format!("{:.1}★", r).yellow());
    }
    if item.production_year.is_some()
        || item.official_rating.is_some()
        || item.community_rating.is_some()
    {
        println!();
    }

    let runtime = format_runtime(item.run_time_ticks);
    if !runtime.is_empty() {
        println!("  Runtime: {}", runtime);
    }

    if !item.genres.is_empty() {
        println!("  Genres: {}", item.genres.join(", "));
    }

    if let Some(ref status) = item.status {
        println!("  Status: {}", status);
    }

    if let Some(count) = item.child_count {
        let label = match item.item_type.as_str() {
            "Series" => "Seasons",
            "Season" => "Episodes",
            "MusicAlbum" => "Tracks",
            "MusicArtist" => "Albums",
            "BoxSet" => "Items",
            _ => "Children",
        };
        println!("  {}: {}", label, count);
    }

    if let Some(ref ud) = item.user_data {
        let mut flags = Vec::new();
        if ud.played {
            flags.push("played".green().to_string());
        }
        if ud.is_favorite {
            flags.push("favorite".red().to_string());
        }
        if ud.play_count > 0 {
            flags.push(format!("{}x", ud.play_count));
        }
        if !flags.is_empty() {
            println!("  User: {}", flags.join(", "));
        }
    }

    if let Some(ref tags) = item.tags {
        if !tags.is_empty() {
            println!("  Tags: {}", tags.join(", ").dimmed());
        }
    }

    if let Some(ref studios) = item.studios {
        if !studios.is_empty() {
            let names: Vec<&str> = studios.iter().map(|s| s.name.as_str()).collect();
            println!("  Studios: {}", names.join(", "));
        }
    }

    if let Some(ref people) = item.people {
        let directors: Vec<&str> = people
            .iter()
            .filter(|p| p.person_type.as_deref() == Some("Director"))
            .map(|p| p.name.as_str())
            .collect();
        if !directors.is_empty() {
            println!("  Director: {}", directors.join(", "));
        }

        let actors: Vec<String> = people
            .iter()
            .filter(|p| p.person_type.as_deref() == Some("Actor"))
            .take(8)
            .map(|p| {
                if let Some(ref role) = p.role {
                    format!("{} ({})", p.name, role.dimmed())
                } else {
                    p.name.clone()
                }
            })
            .collect();
        if !actors.is_empty() {
            println!("  Cast: {}", actors.join(", "));
        }
    }

    if let Some(ref sources) = item.media_sources {
        for src in sources {
            let mut parts = Vec::new();
            if let Some(ref c) = src.container {
                parts.push(c.to_uppercase());
            }
            if let Some(br) = src.bitrate {
                parts.push(format!("{:.1} Mbps", br as f64 / 1_000_000.0));
            }
            if let Some(size) = src.size {
                parts.push(format_size(size));
            }
            if !parts.is_empty() {
                println!("  Media: {}", parts.join(" | "));
            }
            for stream in &src.media_streams {
                if let Some(ref dt) = stream.display_title {
                    println!(
                        "    {}: {}",
                        stream.stream_type.dimmed(),
                        dt
                    );
                }
            }
        }
    }

    if let Some(ref overview) = item.overview {
        println!("\n  {}", textwrap(overview, 76));
    }

    println!("\n  ID: {}", item.id.dimmed());
    let token_hint = "<token>";
    println!(
        "  Stream: {}/Videos/{}/stream?Static=true&api_key={}",
        server_url, item.id, token_hint
    );
    println!();
}

pub fn print_users(users: &[UserDto]) {
    for user in users {
        let last = user
            .last_activity_date
            .as_deref()
            .unwrap_or("never");
        println!(
            "  {} {} last active: {}",
            user.name.bold(),
            user.id.dimmed(),
            last.dimmed()
        );
    }
}

pub fn print_system_info(info: &SystemInfo) {
    println!("  Server: {}", info.server_name.bold());
    println!("  Version: {}", info.version.cyan());
    if let Some(ref os) = info.operating_system {
        if !os.is_empty() {
            println!("  OS: {}", os);
        }
    }
    if let Some(ref addr) = info.local_address {
        println!("  Address: {}", addr);
    }
    println!("  ID: {}", info.id.dimmed());
    println!(
        "  Pending restart: {}",
        if info.has_pending_restart {
            "yes".yellow().to_string()
        } else {
            "no".to_string()
        }
    );
}

fn format_type_tag(t: &str) -> String {
    let (label, color) = match t {
        "Movie" => ("MOV", "blue"),
        "Series" => ("SER", "magenta"),
        "Season" => ("SEA", "magenta"),
        "Episode" => ("EP ", "cyan"),
        "Audio" => ("AUD", "green"),
        "MusicAlbum" => ("ALB", "green"),
        "MusicArtist" => ("ART", "green"),
        "Book" => ("BOK", "yellow"),
        "BoxSet" => ("SET", "red"),
        "Person" => ("PER", "white"),
        _ => ("???", "white"),
    };
    let s = format!("[{}]", label);
    match color {
        "blue" => s.blue().bold().to_string(),
        "magenta" => s.magenta().bold().to_string(),
        "cyan" => s.cyan().bold().to_string(),
        "green" => s.green().bold().to_string(),
        "yellow" => s.yellow().bold().to_string(),
        "red" => s.red().bold().to_string(),
        _ => s.white().bold().to_string(),
    }
}

fn format_episode_number(season: Option<u32>, episode: Option<u32>) -> String {
    match (season, episode) {
        (Some(s), Some(e)) => format!("S{:02}E{:02}", s, e),
        (None, Some(e)) => format!("E{:02}", e),
        (Some(s), None) => format!("S{:02}", s),
        _ => String::new(),
    }
}

fn format_runtime(ticks: Option<u64>) -> String {
    match ticks {
        Some(t) if t > 0 => {
            let minutes = t / 10_000_000 / 60;
            if minutes >= 60 {
                format!("{}h{}m", minutes / 60, minutes % 60)
            } else {
                format!("{}m", minutes)
            }
        }
        _ => String::new(),
    }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.0} MB", bytes as f64 / 1_000_000.0)
    } else {
        format!("{} KB", bytes / 1000)
    }
}

fn textwrap(text: &str, width: usize) -> String {
    let clean = text
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n");
    let clean = strip_html(&clean);
    let mut result = String::new();
    for paragraph in clean.split('\n') {
        let trimmed = paragraph.trim();
        if trimmed.is_empty() {
            result.push('\n');
            continue;
        }
        let mut line_len = 0;
        for word in trimmed.split_whitespace() {
            if line_len > 0 && line_len + word.len() + 1 > width {
                result.push_str("\n  ");
                line_len = 0;
            } else if line_len > 0 {
                result.push(' ');
                line_len += 1;
            }
            result.push_str(word);
            line_len += word.len();
        }
        result.push('\n');
    }
    result.trim_end().to_string()
}

fn strip_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            out.push(c);
        }
    }
    out
}

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::config::Config;

pub struct Client {
    http: reqwest::Client,
    base_url: String,
    token: Option<String>,
    user_id: Option<String>,
    auth_header: String,
}

// --- Quick Connect types ---

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct QuickConnectResult {
    pub secret: String,
    pub code: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct QuickConnectStatus {
    pub authenticated: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AuthResult {
    pub access_token: String,
    pub user: UserDto,
}

// --- Data types ---

#[derive(Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct UserDto {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub last_login_date: Option<String>,
    #[serde(default)]
    pub last_activity_date: Option<String>,
    #[serde(default)]
    pub has_password: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VirtualFolder {
    pub name: String,
    pub collection_type: Option<String>,
    pub item_id: String,
    #[serde(default)]
    pub locations: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ItemsResult {
    pub items: Vec<Item>,
    pub total_record_count: u32,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Item {
    pub id: String,
    pub name: String,
    #[serde(default, rename = "Type")]
    pub item_type: String,
    #[serde(default)]
    pub production_year: Option<u32>,
    #[serde(default)]
    pub community_rating: Option<f64>,
    #[serde(default)]
    pub official_rating: Option<String>,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub series_name: Option<String>,
    #[serde(default)]
    pub season_name: Option<String>,
    #[serde(default)]
    pub index_number: Option<u32>,
    #[serde(default)]
    pub parent_index_number: Option<u32>,
    #[serde(default)]
    pub run_time_ticks: Option<u64>,
    #[serde(default)]
    pub date_created: Option<String>,
    #[serde(default)]
    pub premiere_date: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub container: Option<String>,
    #[serde(default)]
    pub media_sources: Option<Vec<MediaSource>>,
    #[serde(default)]
    pub user_data: Option<UserItemData>,
    #[serde(default)]
    pub people: Option<Vec<Person>>,
    #[serde(default)]
    pub studios: Option<Vec<NameId>>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub child_count: Option<u32>,
    #[serde(default)]
    pub recursive_item_count: Option<u32>,
    #[serde(default)]
    pub location_type: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MediaSource {
    pub size: Option<u64>,
    pub container: Option<String>,
    pub bitrate: Option<u64>,
    #[serde(default)]
    pub media_streams: Vec<MediaStream>,
    pub path: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MediaStream {
    #[serde(rename = "Type")]
    pub stream_type: String,
    pub codec: Option<String>,
    pub display_title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub bit_rate: Option<u64>,
    pub channels: Option<u32>,
    pub language: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct UserItemData {
    #[serde(default)]
    pub played: bool,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub play_count: u32,
    pub last_played_date: Option<String>,
    pub played_percentage: Option<f64>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Person {
    pub name: String,
    pub role: Option<String>,
    #[serde(rename = "Type")]
    pub person_type: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct NameId {
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SearchHintResult {
    pub search_hints: Vec<SearchHint>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SearchHint {
    pub id: String,
    pub name: String,
    #[serde(default, rename = "Type")]
    pub item_type: String,
    #[serde(default)]
    pub production_year: Option<u32>,
    #[serde(default)]
    pub series: Option<String>,
    #[serde(default)]
    pub run_time_ticks: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SystemInfo {
    pub server_name: String,
    pub version: String,
    pub operating_system: Option<String>,
    pub id: String,
    pub has_pending_restart: bool,
    #[serde(default)]
    pub local_address: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Session {
    pub id: String,
    #[serde(default)]
    pub device_name: Option<String>,
    #[serde(default)]
    pub client: Option<String>,
    #[serde(default)]
    pub user_name: Option<String>,
    #[serde(default)]
    pub supports_remote_control: bool,
    #[serde(default)]
    pub now_playing_item: Option<Item>,
    #[serde(default)]
    pub play_state: Option<PlayState>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PlayState {
    #[serde(default)]
    pub is_paused: bool,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub state: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub current_progress_percentage: Option<f64>,
    #[serde(default)]
    pub last_execution_result: Option<TaskExecutionResult>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct TaskExecutionResult {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub end_time_utc: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlaylistCreationResult {
    pub id: String,
}

pub fn normalize_item_type(t: &str) -> String {
    match t.to_lowercase().as_str() {
        "movie" | "movies" => "Movie",
        "series" | "show" | "shows" => "Series",
        "episode" | "episodes" => "Episode",
        "audio" | "song" | "songs" | "track" | "tracks" => "Audio",
        "album" | "albums" => "MusicAlbum",
        "artist" | "artists" => "MusicArtist",
        "book" | "books" => "Book",
        "boxset" | "collection" => "BoxSet",
        other => other,
    }
    .to_string()
}

/// Maps an item type to its `/Items/RemoteSearch/{Type}` path segment, if identification is supported.
pub fn remote_search_type(item_type: &str) -> Option<&'static str> {
    match item_type {
        "Movie" => Some("Movie"),
        "Series" => Some("Series"),
        "Episode" => Some("Episode"),
        "MusicVideo" => Some("MusicVideo"),
        "BoxSet" => Some("BoxSet"),
        "Trailer" => Some("Trailer"),
        "Person" => Some("Person"),
        _ => None,
    }
}

/// Jellyfin 12 removed query-string token authentication (`?api_key=`), so media URLs must be
/// authenticated with this header instead.
pub fn auth_header(token: &str) -> String {
    format!("Authorization: MediaBrowser Token=\"{}\"", token)
}

/// The same header as an mpv option; mpv applies it to every network request it makes,
/// including the entries of a playlist.
pub fn mpv_auth_arg(token: &str) -> String {
    format!("--http-header-fields={}", auth_header(token))
}

impl Client {
    pub fn new(cfg: &Config) -> Result<Self> {
        let token = cfg
            .access_token
            .as_ref()
            .context("not authenticated — run `jf login`")?;
        Ok(Self {
            http: reqwest::Client::new(),
            base_url: cfg.server_url.clone(),
            token: Some(token.clone()),
            user_id: cfg.user_id.clone(),
            auth_header: Self::make_auth_header(&cfg.device_id),
        })
    }

    pub fn new_unauthenticated(server_url: &str) -> Self {
        let device_id = uuid::Uuid::new_v4().to_string();
        Self {
            http: reqwest::Client::new(),
            base_url: server_url.to_string(),
            token: None,
            user_id: None,
            auth_header: Self::make_auth_header(&device_id),
        }
    }

    fn make_auth_header(device_id: &str) -> String {
        format!(
            "MediaBrowser Client=\"Jellyfin CLI\", Device=\"CLI\", DeviceId=\"{}\", Version=\"0.1.0\"",
            device_id
        )
    }

    /// Jellyfin 12 disabled the legacy `X-Emby-*` authorization headers; the supported scheme is a
    /// single `Authorization: MediaBrowser ...` header, carrying the token when authenticated.
    fn authorization(&self) -> String {
        match self.token {
            Some(ref token) => format!("{}, Token=\"{}\"", self.auth_header, token),
            None => self.auth_header.clone(),
        }
    }

    fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.http
            .get(format!("{}{}", self.base_url, path))
            .header("Authorization", self.authorization())
    }

    fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.http
            .post(format!("{}{}", self.base_url, path))
            .header("Authorization", self.authorization())
    }

    fn delete(&self, path: &str) -> reqwest::RequestBuilder {
        self.http
            .delete(format!("{}{}", self.base_url, path))
            .header("Authorization", self.authorization())
    }

    // --- Quick Connect ---

    pub async fn quick_connect_initiate(&self) -> Result<QuickConnectResult> {
        Ok(self
            .post("/QuickConnect/Initiate")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn quick_connect_status(&self, secret: &str) -> Result<QuickConnectStatus> {
        Ok(self
            .get(&format!("/QuickConnect/Connect?Secret={}", secret))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn authenticate_quick_connect(&self, secret: &str) -> Result<AuthResult> {
        Ok(self
            .post("/Users/AuthenticateWithQuickConnect")
            .json(&serde_json::json!({ "Secret": secret }))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    // --- System ---

    pub async fn system_info(&self) -> Result<SystemInfo> {
        Ok(self
            .get("/System/Info")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    // --- Users ---

    pub async fn me(&self) -> Result<UserDto> {
        let users: Vec<UserDto> = self
            .get("/Users")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        users.into_iter().next().context("no users found")
    }

    pub async fn users(&self) -> Result<Vec<UserDto>> {
        Ok(self
            .get("/Users")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    // --- Libraries ---

    pub async fn libraries(&self) -> Result<Vec<VirtualFolder>> {
        match self
            .get("/Library/VirtualFolders")
            .send()
            .await?
            .error_for_status()
        {
            Ok(resp) => Ok(resp.json().await?),
            Err(_) => {
                let user_id = self.user_id.as_deref().unwrap_or("");
                let result: ItemsResult = self
                    .get(&format!("/Users/{}/Items?Fields=ChildCount", user_id))
                    .send()
                    .await?
                    .error_for_status()?
                    .json()
                    .await?;
                Ok(result
                    .items
                    .into_iter()
                    .map(|i| VirtualFolder {
                        name: i.name,
                        collection_type: Some(i.item_type),
                        item_id: i.id,
                        locations: vec![],
                    })
                    .collect())
            }
        }
    }

    pub async fn resolve_library_id(&self, name: &str) -> Result<String> {
        if name.len() == 32 && name.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(name.to_string());
        }
        let libs = self.libraries().await?;
        let lower = name.to_lowercase();
        for lib in &libs {
            if lib.name.to_lowercase() == lower || lib.name.to_lowercase().contains(&lower) {
                return Ok(lib.item_id.clone());
            }
        }
        bail!("library not found: {}", name)
    }

    // --- Items ---

    pub async fn items(
        &self,
        parent_id: Option<&str>,
        item_type: Option<&str>,
        limit: u32,
        sort_by: &str,
    ) -> Result<ItemsResult> {
        let user_id = self.user_id.as_deref().unwrap_or("");
        let mut url = format!(
            "/Users/{}/Items?Limit={}&SortBy={}&SortOrder=Ascending&Recursive=true&Fields=Overview,Genres,CommunityRating,OfficialRating,ProductionYear,DateCreated,RunTimeTicks,Container,UserData,ChildCount",
            user_id, limit, sort_by
        );
        if let Some(pid) = parent_id {
            url.push_str(&format!("&ParentId={}", pid));
        }
        if let Some(t) = item_type {
            url.push_str(&format!("&IncludeItemTypes={}", t));
        }
        Ok(self
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn item_detail(&self, id: &str) -> Result<Item> {
        let user_id = self.user_id.as_deref().unwrap_or("");
        Ok(self
            .get(&format!(
                "/Users/{}/Items/{}?Fields=Overview,Genres,CommunityRating,OfficialRating,People,Studios,Tags,MediaSources,ProductionYear,DateCreated,RunTimeTicks,Container,UserData,ChildCount",
                user_id, id
            ))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn resolve_item_id(&self, input: &str) -> Result<String> {
        self.resolve_item_id_of_type(input, None).await
    }

    pub async fn resolve_item_id_of_type(
        &self,
        input: &str,
        item_type: Option<&str>,
    ) -> Result<String> {
        if input.len() == 32 && input.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(input.to_string());
        }
        let (query, year) = extract_trailing_year(input);
        // Untyped hint searches can be dominated by hundreds of episodes before
        // their parent series appears. Typed callers need less headroom, but still
        // fetch enough candidates to detect same-title remakes safely.
        let limit = if item_type.is_some() { 100 } else { 500 };
        let hints = self.search(&query, limit, item_type).await?;
        select_search_hint(&hints, &query, year, item_type)
            .map(|hint| hint.id.clone())
            .with_context(|| format!("could not resolve '{}'", input))
    }

    // --- Search ---

    pub async fn search(
        &self,
        query: &str,
        limit: u32,
        include_types: Option<&str>,
    ) -> Result<Vec<SearchHint>> {
        let user_id = self.user_id.as_deref().unwrap_or("");
        let mut url = format!(
            "/Search/Hints?searchTerm={}&Limit={}&UserId={}",
            urlencoding::encode(query),
            limit,
            user_id
        );
        if let Some(types) = include_types {
            url.push_str(&format!("&IncludeItemTypes={}", types));
        }
        let result: SearchHintResult = self
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(result.search_hints)
    }

    // --- Seasons / Episodes ---

    pub async fn seasons(&self, series_id: &str) -> Result<ItemsResult> {
        let user_id = self.user_id.as_deref().unwrap_or("");
        Ok(self
            .get(&format!(
                "/Shows/{}/Seasons?UserId={}&Fields=Overview,UserData,ChildCount",
                series_id, user_id
            ))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn episodes(&self, series_id: &str, season: Option<u32>) -> Result<ItemsResult> {
        let user_id = self.user_id.as_deref().unwrap_or("");
        let mut url = format!(
            "/Shows/{}/Episodes?UserId={}&Fields=Overview,UserData,RunTimeTicks,Container,LocationType",
            series_id, user_id
        );
        if let Some(s) = season {
            url.push_str(&format!("&Season={}", s));
        }
        Ok(self
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    // --- Playlists ---

    pub async fn playlists(&self) -> Result<ItemsResult> {
        let user_id = self.user_id.as_deref().unwrap_or("");
        Ok(self
            .get(&format!(
                "/Users/{}/Items?IncludeItemTypes=Playlist&Recursive=true&Limit=1000&SortBy=SortName&SortOrder=Ascending&Fields=Overview,RunTimeTicks,ChildCount",
                user_id
            ))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn resolve_playlist_id(&self, name: &str) -> Result<String> {
        if name.len() == 32 && name.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(name.to_string());
        }

        let playlists = self.playlists().await?.items;
        let lower = name.to_lowercase();
        playlists
            .iter()
            .find(|p| p.name.to_lowercase() == lower)
            .or_else(|| {
                playlists
                    .iter()
                    .find(|p| p.name.to_lowercase().contains(&lower))
            })
            .map(|p| p.id.clone())
            .with_context(|| format!("no playlist matching: {}", name))
    }

    pub async fn playlist_items(&self, playlist_id: &str) -> Result<ItemsResult> {
        let user_id = self.user_id.as_deref().unwrap_or("");
        Ok(self
            .get(&format!(
                "/Playlists/{}/Items?UserId={}&Limit=100000&Fields=RunTimeTicks,Genres,ProductionYear",
                playlist_id, user_id
            ))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn create_playlist(
        &self,
        name: &str,
        item_ids: &[String],
        public: bool,
    ) -> Result<String> {
        let user_id = self
            .user_id
            .as_deref()
            .context("playlist creation requires a user login")?;
        let body = serde_json::json!({
            "Name": name,
            "Ids": item_ids,
            "UserId": user_id,
            "MediaType": "Video",
            "Users": [{
                "UserId": user_id,
                "CanEdit": true
            }],
            "IsPublic": public
        });
        let result: PlaylistCreationResult = self
            .post("/Playlists")
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(result.id)
    }

    pub async fn add_playlist_items(&self, playlist_id: &str, item_ids: &[String]) -> Result<()> {
        let user_id = self
            .user_id
            .as_deref()
            .context("playlist changes require a user login")?;
        for ids in item_ids.chunks(100) {
            let joined = ids.join(",");
            self.post(&format!(
                "/Playlists/{}/Items?Ids={}&UserId={}",
                playlist_id, joined, user_id
            ))
            .send()
            .await?
            .error_for_status()?;
        }
        Ok(())
    }

    pub async fn replace_playlist_items(
        &self,
        playlist_id: &str,
        item_ids: &[String],
    ) -> Result<()> {
        self.post(&format!("/Playlists/{}", playlist_id))
            .json(&serde_json::json!({ "Ids": item_ids }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    // --- Latest Items ---

    pub async fn latest_items(&self, limit: u32) -> Result<Vec<Item>> {
        let user_id = self.user_id.as_deref().unwrap_or("");
        Ok(self
            .get(&format!(
                "/Users/{}/Items/Latest?Limit={}&Fields=Overview,Genres,CommunityRating,ProductionYear,RunTimeTicks,Container,UserData",
                user_id, limit
            ))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    // --- Mark played/favorite ---

    pub async fn mark_played(&self, id: &str) -> Result<()> {
        let user_id = self.user_id.as_deref().unwrap_or("");
        self.post(&format!("/Users/{}/PlayedItems/{}", user_id, id))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn mark_unplayed(&self, id: &str) -> Result<()> {
        let user_id = self.user_id.as_deref().unwrap_or("");
        self.delete(&format!("/Users/{}/PlayedItems/{}", user_id, id))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn set_favorite(&self, id: &str, favorite: bool) -> Result<()> {
        let user_id = self.user_id.as_deref().unwrap_or("");
        let path = format!("/Users/{}/FavoriteItems/{}", user_id, id);
        if favorite {
            self.post(&path).send().await?.error_for_status()?;
        } else {
            self.delete(&path).send().await?.error_for_status()?;
        }
        Ok(())
    }

    // --- Sessions / remote control ---

    pub async fn sessions(&self) -> Result<Vec<Session>> {
        Ok(self
            .get("/Sessions")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn resolve_session(&self, to: Option<&str>) -> Result<Session> {
        let controllable: Vec<Session> = self
            .sessions()
            .await?
            .into_iter()
            .filter(|s| s.supports_remote_control)
            .collect();

        if controllable.is_empty() {
            bail!(
                "no controllable device is active — open the Jellyfin app on the device and try again"
            );
        }

        match to {
            Some(name) => {
                let lower = name.to_lowercase();
                controllable
                    .into_iter()
                    .find(|s| {
                        let dev = s.device_name.as_deref().unwrap_or("").to_lowercase();
                        let client = s.client.as_deref().unwrap_or("").to_lowercase();
                        dev.contains(&lower) || client.contains(&lower)
                    })
                    .with_context(|| format!("no controllable device matching '{}'", name))
            }
            None => {
                if controllable.len() == 1 {
                    Ok(controllable.into_iter().next().unwrap())
                } else {
                    let names: Vec<String> = controllable
                        .iter()
                        .map(|s| {
                            s.device_name
                                .clone()
                                .or_else(|| s.client.clone())
                                .unwrap_or_else(|| s.id.clone())
                        })
                        .collect();
                    bail!(
                        "multiple controllable devices — specify --to <name>: {}",
                        names.join(", ")
                    )
                }
            }
        }
    }

    pub async fn play_on_session(
        &self,
        session_id: &str,
        item_id: &str,
        play_command: &str,
    ) -> Result<()> {
        self.post(&format!(
            "/Sessions/{}/Playing?itemIds={}&playCommand={}",
            session_id, item_id, play_command
        ))
        .send()
        .await?
        .error_for_status()?;
        Ok(())
    }

    pub async fn playstate_command(
        &self,
        session_id: &str,
        command: &str,
        position_ticks: Option<u64>,
    ) -> Result<()> {
        let mut path = format!("/Sessions/{}/Playing/{}", session_id, command);
        if let Some(ticks) = position_ticks {
            path.push_str(&format!("?seekPositionTicks={}", ticks));
        }
        self.post(&path).send().await?.error_for_status()?;
        Ok(())
    }

    // --- Library scan / metadata refresh ---

    pub async fn refresh_all_libraries(&self) -> Result<()> {
        self.post("/Library/Refresh")
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn refresh_item(
        &self,
        id: &str,
        metadata_refresh_mode: &str,
        image_refresh_mode: &str,
        replace_metadata: bool,
        replace_images: bool,
    ) -> Result<()> {
        let path = format!(
            "/Items/{}/Refresh?Recursive=true&MetadataRefreshMode={}&ImageRefreshMode={}&ReplaceAllMetadata={}&ReplaceAllImages={}",
            id, metadata_refresh_mode, image_refresh_mode, replace_metadata, replace_images
        );
        self.post(&path).send().await?.error_for_status()?;
        Ok(())
    }

    // --- Remote metadata search / identify ---

    pub async fn remote_search(
        &self,
        search_type: &str,
        item_id: &str,
        name: &str,
        year: Option<u32>,
    ) -> Result<Vec<serde_json::Value>> {
        let mut search_info = serde_json::json!({ "Name": name });
        if let Some(y) = year {
            search_info["Year"] = serde_json::json!(y);
        }
        let body = serde_json::json!({
            "ItemId": item_id,
            "SearchInfo": search_info,
        });
        Ok(self
            .post(&format!("/Items/RemoteSearch/{}", search_type))
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn apply_remote_search(
        &self,
        item_id: &str,
        result: &serde_json::Value,
        replace_images: bool,
    ) -> Result<()> {
        self.post(&format!(
            "/Items/RemoteSearch/Apply/{}?replaceAllImages={}",
            item_id, replace_images
        ))
        .json(result)
        .send()
        .await?
        .error_for_status()?;
        Ok(())
    }

    // --- Scheduled tasks ---

    pub async fn scheduled_tasks(&self) -> Result<Vec<TaskInfo>> {
        Ok(self
            .get("/ScheduledTasks")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn resolve_task_id(&self, name: &str) -> Result<String> {
        let tasks = self.scheduled_tasks().await?;
        let lower = name.to_lowercase();
        tasks
            .iter()
            .find(|t| t.name.to_lowercase() == lower)
            .or_else(|| {
                tasks
                    .iter()
                    .find(|t| t.name.to_lowercase().contains(&lower))
            })
            .map(|t| t.id.clone())
            .with_context(|| format!("no scheduled task matching: {}", name))
    }

    pub async fn start_task(&self, id: &str) -> Result<()> {
        self.post(&format!("/ScheduledTasks/Running/{}", id))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

pub fn extract_trailing_year(query: &str) -> (String, Option<u32>) {
    let words: Vec<&str> = query.split_whitespace().collect();
    if words.len() >= 2 {
        if let Ok(y) = words.last().unwrap().parse::<u32>() {
            if (1900..=2099).contains(&y) {
                let trimmed = words[..words.len() - 1].join(" ");
                return (trimmed, Some(y));
            }
        }
    }
    (query.to_string(), None)
}

fn normalized_search_name(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn search_type_rank(item_type: &str) -> u8 {
    match item_type {
        "Series" | "Movie" | "MusicAlbum" | "MusicArtist" | "BoxSet" | "Playlist" => 0,
        "Episode" | "Audio" | "Video" | "Book" => 1,
        _ => 2,
    }
}

pub fn sort_search_hints(hints: &mut [SearchHint], query: &str) {
    let normalized_query = normalized_search_name(query);
    hints.sort_by_key(|hint| {
        (
            normalized_search_name(&hint.name) != normalized_query,
            search_type_rank(&hint.item_type),
        )
    });
}

fn select_search_hint<'a>(
    hints: &'a [SearchHint],
    query: &str,
    year: Option<u32>,
    item_type: Option<&str>,
) -> Result<&'a SearchHint> {
    let mut candidates = hints
        .iter()
        .filter(|hint| {
            item_type.is_none_or(|wanted| hint.item_type.eq_ignore_ascii_case(wanted))
                && year.is_none_or(|wanted| hint.production_year == Some(wanted))
        })
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        let type_description = item_type
            .map(|value| format!(" {}", value))
            .unwrap_or_default();
        let year_description = year
            .map(|value| format!(" from {}", value))
            .unwrap_or_default();
        bail!(
            "no{} item named '{}'{}",
            type_description,
            query,
            year_description
        );
    }

    let normalized_query = normalized_search_name(query);
    let exact = candidates
        .iter()
        .copied()
        .filter(|hint| normalized_search_name(&hint.name) == normalized_query)
        .collect::<Vec<_>>();
    if !exact.is_empty() {
        candidates = exact;
    }

    let best_type_rank = candidates
        .iter()
        .map(|hint| search_type_rank(&hint.item_type))
        .min()
        .unwrap_or(u8::MAX);
    candidates.retain(|hint| search_type_rank(&hint.item_type) == best_type_rank);

    let mut seen_ids = std::collections::HashSet::new();
    candidates.retain(|hint| seen_ids.insert(hint.id.as_str()));
    if candidates.len() == 1 {
        return Ok(candidates[0]);
    }

    // Jellyfin may expose multiple physical copies of the same logical work.
    // If every remaining candidate has the same type, normalized title, and
    // year, choose a stable ID instead of treating equivalent encodes as a
    // reboot/remake ambiguity.
    let first = candidates[0];
    if candidates.iter().all(|hint| {
        hint.item_type == first.item_type
            && hint.production_year == first.production_year
            && normalized_search_name(&hint.name) == normalized_search_name(&first.name)
    }) {
        candidates.sort_by_key(|hint| hint.id.as_str());
        return Ok(candidates[0]);
    }

    let mut descriptions = candidates
        .iter()
        .map(|hint| {
            format!(
                "{} '{}'{} ({})",
                hint.item_type,
                hint.name,
                hint.production_year
                    .map(|value| format!(" {}", value))
                    .unwrap_or_default(),
                hint.id
            )
        })
        .collect::<Vec<_>>();
    descriptions.sort();
    descriptions.truncate(8);
    bail!(
        "ambiguous match for '{}'; use a trailing year, --type where supported, or an item ID. Candidates: {}",
        query,
        descriptions.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::{SearchHint, extract_trailing_year, select_search_hint, sort_search_hints};

    fn hint(id: &str, name: &str, year: u32, item_type: &str) -> SearchHint {
        SearchHint {
            id: id.to_string(),
            name: name.to_string(),
            item_type: item_type.to_string(),
            production_year: Some(year),
            series: None,
            run_time_ticks: None,
        }
    }

    #[test]
    fn extracts_only_a_plausible_trailing_year() {
        assert_eq!(
            extract_trailing_year("Urusei Yatsura 2022"),
            ("Urusei Yatsura".to_string(), Some(2022))
        );
        assert_eq!(
            extract_trailing_year("2001: A Space Odyssey"),
            ("2001: A Space Odyssey".to_string(), None)
        );
    }

    #[test]
    fn typed_year_resolution_selects_the_requested_reboot() {
        let hints = vec![
            hint("new", "Urusei Yatsura", 2022, "Series"),
            hint("old", "Urusei Yatsura", 1981, "Series"),
        ];
        let selected =
            select_search_hint(&hints, "Urusei Yatsura", Some(1981), Some("Series")).unwrap();
        assert_eq!(selected.id, "old");
    }

    #[test]
    fn explicit_missing_year_does_not_fall_back() {
        let hints = vec![hint("old", "Urusei Yatsura", 1981, "Series")];
        let error =
            select_search_hint(&hints, "Urusei Yatsura", Some(2022), Some("Series")).unwrap_err();
        assert!(error.to_string().contains("no Series item"));
    }

    #[test]
    fn same_title_reboots_are_ambiguous_without_a_year() {
        let hints = vec![
            hint("new", "Urusei Yatsura", 2022, "Series"),
            hint("old", "Urusei Yatsura", 1981, "Series"),
        ];
        let error = select_search_hint(&hints, "Urusei Yatsura", None, Some("Series")).unwrap_err();
        assert!(error.to_string().contains("ambiguous match"));
    }

    #[test]
    fn duplicate_encodes_of_the_same_work_resolve_stably() {
        let hints = vec![
            hint("z-copy", "Ghost in the Shell", 1995, "Movie"),
            hint("a-copy", "Ghost in the Shell", 1995, "Movie"),
        ];
        let selected =
            select_search_hint(&hints, "Ghost in the Shell", Some(1995), Some("Movie")).unwrap();
        assert_eq!(selected.id, "a-copy");
    }

    #[test]
    fn untyped_resolution_prefers_a_series_over_same_named_episodes() {
        let hints = vec![
            hint("episode", "Urusei Yatsura", 2022, "Episode"),
            hint("series", "Urusei Yatsura", 2022, "Series"),
        ];
        let selected = select_search_hint(&hints, "Urusei Yatsura", Some(2022), None).unwrap();
        assert_eq!(selected.id, "series");
    }

    #[test]
    fn search_ranking_puts_exact_top_level_items_first() {
        let mut hints = vec![
            hint("episode", "Urusei Yatsura", 1981, "Episode"),
            hint("partial", "Urusei Yatsura Movie", 1983, "Movie"),
            hint("series", "Urusei Yatsura", 2022, "Series"),
        ];
        sort_search_hints(&mut hints, "Urusei Yatsura");
        assert_eq!(
            hints
                .iter()
                .map(|hint| hint.id.as_str())
                .collect::<Vec<_>>(),
            vec!["series", "episode", "partial"]
        );
    }
}

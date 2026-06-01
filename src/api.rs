use anyhow::{bail, Context, Result};
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

#[derive(Deserialize, Clone)]
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

impl Client {
    pub fn new(cfg: &Config) -> Result<Self> {
        let token = cfg.access_token.as_ref().context("not authenticated — run `jf login`")?;
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

    fn get(&self, path: &str) -> reqwest::RequestBuilder {
        let mut req = self
            .http
            .get(format!("{}{}", self.base_url, path))
            .header("X-Emby-Authorization", &self.auth_header);
        if let Some(ref t) = self.token {
            req = req.header("X-Emby-Token", t);
        }
        req
    }

    fn post(&self, path: &str) -> reqwest::RequestBuilder {
        let mut req = self
            .http
            .post(format!("{}{}", self.base_url, path))
            .header("X-Emby-Authorization", &self.auth_header);
        if let Some(ref t) = self.token {
            req = req.header("X-Emby-Token", t);
        }
        req
    }

    fn delete(&self, path: &str) -> reqwest::RequestBuilder {
        let mut req = self
            .http
            .delete(format!("{}{}", self.base_url, path))
            .header("X-Emby-Authorization", &self.auth_header);
        if let Some(ref t) = self.token {
            req = req.header("X-Emby-Token", t);
        }
        req
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
                    .get(&format!(
                        "/Users/{}/Items?Fields=ChildCount",
                        user_id
                    ))
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
            if lib.name.to_lowercase() == lower
                || lib.name.to_lowercase().contains(&lower)
            {
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
        if input.len() == 32 && input.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(input.to_string());
        }
        let hints = self.search(input, 1).await?;
        match hints.first() {
            Some(h) => Ok(h.id.clone()),
            None => bail!("no item found matching: {}", input),
        }
    }

    // --- Search ---

    pub async fn search(&self, query: &str, limit: u32) -> Result<Vec<SearchHint>> {
        let user_id = self.user_id.as_deref().unwrap_or("");
        let result: SearchHintResult = self
            .get(&format!(
                "/Search/Hints?searchTerm={}&Limit={}&UserId={}",
                urlencoding(query),
                limit,
                user_id
            ))
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
            "/Shows/{}/Episodes?UserId={}&Fields=Overview,UserData,RunTimeTicks,Container",
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
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "%20")
        .replace('&', "%26")
        .replace('=', "%3D")
        .replace('?', "%3F")
        .replace('#', "%23")
}

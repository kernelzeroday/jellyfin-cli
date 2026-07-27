use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::{cmp::Ordering, collections::HashSet, path::Path};

use crate::api::{Client, Item};

const TICKS_PER_SECOND: u64 = 10_000_000;
const DEFAULT_EPISODE_TICKS: u64 = 24 * 60 * TICKS_PER_SECOND;
const DEFAULT_MOVIE_TICKS: u64 = 100 * 60 * TICKS_PER_SECOND;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgramConfig {
    pub channels: Vec<ChannelSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelSpec {
    pub name: String,
    pub target_hours: f64,
    #[serde(default = "default_movie_interval")]
    pub movie_every_episodes: usize,
    #[serde(default)]
    pub include_specials: bool,
    #[serde(default)]
    pub minimum_episode_minutes: u64,
    #[serde(default)]
    pub maximum_episode_minutes: Option<u64>,
    #[serde(default)]
    pub series: Vec<String>,
    #[serde(default)]
    pub movies: Vec<String>,
    #[serde(default)]
    pub movie_filters: Vec<MovieFilterSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MovieFilterSpec {
    #[serde(default)]
    pub genres_any: Vec<String>,
    #[serde(default)]
    pub genres_all: Vec<String>,
    #[serde(default)]
    pub exclude_genres: Vec<String>,
    pub year_min: Option<u32>,
    pub year_max: Option<u32>,
    pub minimum_rating: Option<f64>,
    pub minimum_runtime_minutes: Option<u64>,
    pub maximum_runtime_minutes: Option<u64>,
    #[serde(default = "default_movie_sort")]
    pub sort: String,
    pub limit: Option<usize>,
}

pub struct BuiltChannel {
    pub name: String,
    pub items: Vec<Item>,
    pub series_count: usize,
    pub movie_count: usize,
    pub duration_ticks: u64,
}

fn default_movie_interval() -> usize {
    12
}

fn default_movie_sort() -> String {
    "shuffle".to_string()
}

impl ProgramConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let data = std::fs::read_to_string(path)
            .with_context(|| format!("could not read programming config {}", path.display()))?;
        let config: Self = serde_json::from_str(&data)
            .with_context(|| format!("invalid programming config {}", path.display()))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.channels.is_empty() {
            bail!("programming config has no channels");
        }
        for channel in &self.channels {
            if channel.name.trim().is_empty() {
                bail!("channel name cannot be empty");
            }
            if !channel.target_hours.is_finite() || channel.target_hours <= 0.0 {
                bail!("channel '{}' must have target_hours > 0", channel.name);
            }
            if channel.series.is_empty()
                && channel.movies.is_empty()
                && channel.movie_filters.is_empty()
            {
                bail!("channel '{}' has no content sources", channel.name);
            }
            if let Some(maximum) = channel.maximum_episode_minutes
                && maximum < channel.minimum_episode_minutes
            {
                bail!(
                    "channel '{}' has maximum_episode_minutes below minimum_episode_minutes",
                    channel.name
                );
            }
            if !channel.series.is_empty()
                && (!channel.movies.is_empty() || !channel.movie_filters.is_empty())
                && channel.movie_every_episodes == 0
            {
                bail!(
                    "channel '{}' must have movie_every_episodes > 0",
                    channel.name
                );
            }
            for filter in &channel.movie_filters {
                if let (Some(minimum), Some(maximum)) = (filter.year_min, filter.year_max)
                    && maximum < minimum
                {
                    bail!(
                        "channel '{}' has a movie filter with year_max below year_min",
                        channel.name
                    );
                }
                if let (Some(minimum), Some(maximum)) = (
                    filter.minimum_runtime_minutes,
                    filter.maximum_runtime_minutes,
                ) && maximum < minimum
                {
                    bail!(
                        "channel '{}' has a movie filter with maximum runtime below minimum",
                        channel.name
                    );
                }
                if !matches!(
                    filter.sort.to_lowercase().as_str(),
                    "shuffle" | "name" | "year" | "rating"
                ) {
                    bail!(
                        "channel '{}' has unknown movie sort '{}'",
                        channel.name,
                        filter.sort
                    );
                }
            }
        }
        Ok(())
    }
}

pub async fn build_channel(client: &Client, spec: &ChannelSpec) -> Result<BuiltChannel> {
    let mut series_episodes = Vec::with_capacity(spec.series.len());
    for series_name in &spec.series {
        let series_id = client
            .resolve_item_id_of_type(series_name, Some("Series"))
            .await
            .with_context(|| {
                format!("channel '{}': series not found: {}", spec.name, series_name)
            })?;
        let detail = client.item_detail(&series_id).await?;
        if detail.item_type != "Series" {
            bail!(
                "channel '{}': '{}' resolved to {}, not a series",
                spec.name,
                series_name,
                detail.item_type
            );
        }

        let mut episodes = client.episodes(&series_id, None).await?.items;
        let minimum_episode_ticks = spec.minimum_episode_minutes * 60 * TICKS_PER_SECOND;
        let maximum_episode_ticks = spec
            .maximum_episode_minutes
            .map(|minutes| minutes * 60 * TICKS_PER_SECOND);
        episodes.retain(|episode| {
            episode.location_type.as_deref() != Some("Virtual")
                && (spec.include_specials
                    || (episode.parent_index_number != Some(0) && episode.index_number != Some(0)))
                && episode
                    .run_time_ticks
                    .is_none_or(|ticks| ticks >= minimum_episode_ticks)
                && maximum_episode_ticks.is_none_or(|maximum| {
                    episode.run_time_ticks.is_none_or(|ticks| ticks <= maximum)
                })
        });
        episodes.sort_by_key(|episode| {
            (
                episode.parent_index_number.unwrap_or(u32::MAX),
                episode.index_number.unwrap_or(u32::MAX),
            )
        });
        if episodes.is_empty() {
            bail!(
                "channel '{}': series '{}' has no playable episodes",
                spec.name,
                detail.name
            );
        }
        series_episodes.push(episodes);
    }

    let mut movies = Vec::with_capacity(spec.movies.len());
    let mut seen_movie_ids = HashSet::new();
    for movie_name in &spec.movies {
        let movie_id = client
            .resolve_item_id_of_type(movie_name, Some("Movie"))
            .await
            .with_context(|| format!("channel '{}': movie not found: {}", spec.name, movie_name))?;
        let movie = client.item_detail(&movie_id).await?;
        if movie.item_type != "Movie" {
            bail!(
                "channel '{}': '{}' resolved to {}, not a movie",
                spec.name,
                movie_name,
                movie.item_type
            );
        }
        if seen_movie_ids.insert(movie.id.clone()) {
            movies.push(movie);
        }
    }

    if !spec.movie_filters.is_empty() {
        let catalog = client
            .items(None, Some("Movie"), 5000, "SortName")
            .await?
            .items;
        for filter in &spec.movie_filters {
            let mut matching = catalog
                .iter()
                .filter(|movie| movie_matches_filter(movie, filter))
                .cloned()
                .collect::<Vec<_>>();
            sort_movies(&mut matching, &filter.sort, &spec.name);

            let mut added = 0_usize;
            for movie in matching {
                if filter.limit.is_some_and(|limit| added >= limit) {
                    break;
                }
                if seen_movie_ids.insert(movie.id.clone()) {
                    movies.push(movie);
                    added += 1;
                }
            }
        }
    }

    if series_episodes.is_empty() && movies.is_empty() {
        bail!("channel '{}' has no playable movies", spec.name);
    }

    let target_ticks = (spec.target_hours * 60.0 * 60.0 * TICKS_PER_SECOND as f64).round() as u64;
    let duration = |item: &Item| {
        item.run_time_ticks.unwrap_or(if item.item_type == "Movie" {
            DEFAULT_MOVIE_TICKS
        } else {
            DEFAULT_EPISODE_TICKS
        })
    };
    let items = if series_episodes.is_empty() {
        interleave(vec![movies], vec![], 0, target_ticks, duration)
    } else {
        interleave(
            series_episodes,
            movies,
            spec.movie_every_episodes,
            target_ticks,
            duration,
        )
    };
    let duration_ticks = items
        .iter()
        .map(|item| {
            item.run_time_ticks.unwrap_or(if item.item_type == "Movie" {
                DEFAULT_MOVIE_TICKS
            } else {
                DEFAULT_EPISODE_TICKS
            })
        })
        .sum();
    let movie_count = items
        .iter()
        .filter(|item| item.item_type == "Movie")
        .count();

    Ok(BuiltChannel {
        name: spec.name.clone(),
        series_count: spec.series.len(),
        movie_count,
        duration_ticks,
        items,
    })
}

fn movie_matches_filter(movie: &Item, filter: &MovieFilterSpec) -> bool {
    let has_genre = |wanted: &str| {
        movie
            .genres
            .iter()
            .any(|genre| genre.eq_ignore_ascii_case(wanted))
    };
    let year_matches = |bound: Option<u32>, comparison: fn(u32, u32) -> bool| {
        bound.is_none_or(|value| {
            movie
                .production_year
                .is_some_and(|year| comparison(year, value))
        })
    };
    let runtime_minutes = movie
        .run_time_ticks
        .map(|ticks| ticks / TICKS_PER_SECOND / 60);

    (filter.genres_any.is_empty() || filter.genres_any.iter().any(|genre| has_genre(genre)))
        && filter.genres_all.iter().all(|genre| has_genre(genre))
        && !filter.exclude_genres.iter().any(|genre| has_genre(genre))
        && year_matches(filter.year_min, |year, minimum| year >= minimum)
        && year_matches(filter.year_max, |year, maximum| year <= maximum)
        && filter.minimum_rating.is_none_or(|minimum| {
            movie
                .community_rating
                .is_some_and(|rating| rating >= minimum)
        })
        && filter
            .minimum_runtime_minutes
            .is_none_or(|minimum| runtime_minutes.is_some_and(|runtime| runtime >= minimum))
        && filter
            .maximum_runtime_minutes
            .is_none_or(|maximum| runtime_minutes.is_some_and(|runtime| runtime <= maximum))
}

fn sort_movies(movies: &mut [Item], sort: &str, channel_name: &str) {
    match sort.to_lowercase().as_str() {
        "name" => movies.sort_by_key(|movie| movie.name.to_lowercase()),
        "year" => movies.sort_by_key(|movie| {
            (
                movie.production_year.unwrap_or(u32::MAX),
                movie.name.to_lowercase(),
            )
        }),
        "rating" => movies.sort_by(|left, right| {
            right
                .community_rating
                .partial_cmp(&left.community_rating)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.name.cmp(&right.name))
        }),
        _ => movies.sort_by_key(|movie| stable_shuffle_key(channel_name, &movie.id)),
    }
}

fn stable_shuffle_key(channel_name: &str, item_id: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in channel_name.bytes().chain(item_id.bytes()) {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn interleave<T, F>(
    series: Vec<Vec<T>>,
    movies: Vec<T>,
    movie_every_episodes: usize,
    target_ticks: u64,
    duration: F,
) -> Vec<T>
where
    F: Fn(&T) -> u64,
{
    let mut series = series
        .into_iter()
        .map(|items| items.into_iter())
        .collect::<Vec<_>>();
    let mut movies = movies.into_iter();
    let mut result = Vec::new();
    let mut duration_ticks = 0_u64;
    let mut episodes_since_movie = 0_usize;

    while duration_ticks < target_ticks {
        let mut made_progress = false;
        for episodes in &mut series {
            let Some(episode) = episodes.next() else {
                continue;
            };
            made_progress = true;
            duration_ticks = duration_ticks.saturating_add(duration(&episode));
            result.push(episode);
            episodes_since_movie += 1;

            if duration_ticks >= target_ticks {
                break;
            }

            if movie_every_episodes > 0 && episodes_since_movie >= movie_every_episodes {
                if let Some(movie) = movies.next() {
                    duration_ticks = duration_ticks.saturating_add(duration(&movie));
                    result.push(movie);
                }
                episodes_since_movie = 0;
            }

            if duration_ticks >= target_ticks {
                break;
            }
        }
        if !made_progress {
            break;
        }
    }

    result
}

pub fn ticks_to_hours(ticks: u64) -> f64 {
    ticks as f64 / TICKS_PER_SECOND as f64 / 60.0 / 60.0
}

#[cfg(test)]
mod tests {
    use super::{interleave, stable_shuffle_key};

    #[derive(Debug, PartialEq)]
    struct TestItem(&'static str, u64);

    #[test]
    fn round_robins_series_in_order() {
        let result = interleave(
            vec![
                vec![TestItem("a1", 1), TestItem("a2", 1)],
                vec![TestItem("b1", 1), TestItem("b2", 1)],
            ],
            vec![],
            12,
            4,
            |item| item.1,
        );
        assert_eq!(
            result,
            vec![
                TestItem("a1", 1),
                TestItem("b1", 1),
                TestItem("a2", 1),
                TestItem("b2", 1)
            ]
        );
    }

    #[test]
    fn inserts_each_movie_at_the_configured_interval() {
        let result = interleave(
            vec![vec![
                TestItem("e1", 1),
                TestItem("e2", 1),
                TestItem("e3", 1),
                TestItem("e4", 1),
            ]],
            vec![TestItem("m1", 2), TestItem("m2", 2)],
            2,
            8,
            |item| item.1,
        );
        assert_eq!(
            result,
            vec![
                TestItem("e1", 1),
                TestItem("e2", 1),
                TestItem("m1", 2),
                TestItem("e3", 1),
                TestItem("e4", 1),
                TestItem("m2", 2)
            ]
        );
    }

    #[test]
    fn sequences_movie_only_channels_until_the_target() {
        let result = interleave(
            vec![vec![
                TestItem("movie1", 2),
                TestItem("movie2", 3),
                TestItem("movie3", 4),
            ]],
            vec![],
            0,
            5,
            |item| item.1,
        );
        assert_eq!(result, vec![TestItem("movie1", 2), TestItem("movie2", 3)]);
    }

    #[test]
    fn shuffle_keys_are_stable_and_channel_specific() {
        assert_eq!(
            stable_shuffle_key("channel", "movie"),
            stable_shuffle_key("channel", "movie")
        );
        assert_ne!(
            stable_shuffle_key("channel-a", "movie"),
            stable_shuffle_key("channel-b", "movie")
        );
    }
}

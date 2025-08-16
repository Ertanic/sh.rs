use std::sync::Arc;

use tokio::select;

use crate::{
    AppState,
    repos::cache::{add_short_to_cache, cache_get},
};

pub async fn get_short_from_db(state: Arc<AppState>, long_url: &str) -> Option<String> {
    let redis_result = cache_get(state.clone(), long_url);

    let db_result =
        sqlx::query_scalar::<_, String>("SELECT short FROM public.shorts WHERE long_url = $1")
            .bind(long_url)
            .fetch_optional(&state.pg_pool);

    select! {
        Ok(Some(short)) = redis_result => {
            Some(short)
        },
        Ok(Some(short)) = db_result => {
            let (cache_short, long_url) = (short.clone(), long_url.to_string());
            tokio::spawn(async move {
                if let Err(e) = add_short_to_cache(state.clone(), &long_url, &cache_short).await {
                    tracing::error!("Failed to add short URL to cache: {}", e);
                }
            });
            Some(short)
        },
        else => {
            None
        }
    }
}

pub fn add_goto_stat(state: Arc<AppState>, short: &str) {
    let short = short.to_string();
    tokio::spawn(async move {
        match sqlx::query("INSERT INTO public.shorts_goto_stats (short_id) SELECT id FROM public.shorts WHERE short = $1")
            .bind(&short)
            .execute(&state.pg_pool)
            .await
        {
            Ok(_) => tracing::trace!("Incremented stat for UID: {}", short),
            Err(e) => tracing::error!("Failed to increment stat: {}", e),
        }
    });
}

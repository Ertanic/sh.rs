#[derive(serde::Serialize)]
pub struct GetShortsGotoStatsResponse {
    pub stats: Vec<GetShortsGotoStatsModel>
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct GetShortsGotoStatsModel {
    pub long_url: String,
    pub total: i64
}
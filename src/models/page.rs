use serde::Serialize;

#[derive(Serialize)]
pub struct Page<TData: Serialize> {
    pub title: String,
    pub data: TData
}
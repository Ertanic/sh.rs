use tera::Tera;

#[tracing::instrument]
pub fn load_templates() -> Tera {
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("navbar", include_str!("../../templates/navbar.html")),
        ("footer", include_str!("../../templates/footer.html")),
        ("base", include_str!("../../templates/base.html")),
        ("index", include_str!("../../templates/index.html")),
    ]).unwrap();
    
    tracing::trace!("Loaded templates: {:#?}", tera.get_template_names().collect::<Vec<_>>());

    tera
}
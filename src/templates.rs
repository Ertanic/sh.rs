use tera::Tera;

pub fn load_templates() -> Tera {
    let tera = Tera::new("templates/**/*").unwrap();

    tracing::trace!(
        "Loaded templates: {:#?}",
        tera.get_template_names().collect::<Vec<_>>()
    );

    tera
}

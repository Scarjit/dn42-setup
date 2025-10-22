use tera::{Context, Tera};

/// Render a template with the given context
pub fn render_template(
    template_name: &str,
    template_str: &str,
    context: &Context,
) -> Result<String, String> {
    let mut tera = Tera::default();
    tera.add_raw_template(template_name, template_str)
        .map_err(|e| format!("Failed to parse template: {}", e))?;

    tera.render(template_name, context)
        .map_err(|e| format!("Failed to render template: {}", e))
}

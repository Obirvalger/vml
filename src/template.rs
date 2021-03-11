use std::fs;

use tera::{Context, Tera};

use crate::{Error, Result};

pub fn render(context: &Context, template: &str, place: &str) -> Result<String> {
    Tera::one_off(template, &context, false).map_err(|e| Error::template(place, &e.to_string()))
}

pub fn renders(context: &Context, templates: &[&str], place: &str) -> Result<Vec<String>> {
    let mut strings = Vec::with_capacity(templates.len());

    for template in templates {
        strings.push(render(context, template, place)?);
    }

    Ok(strings)
}

pub fn render_file(
    context: &Context,
    template_file: &str,
    rendered_file: &str,
    place: &str,
) -> Result<()> {
    let template = fs::read_to_string(template_file)?;
    let rendered = render(context, &template, place)?;
    fs::write(rendered_file, &rendered.as_bytes())?;

    Ok(())
}

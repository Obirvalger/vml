use std::fs;
use std::path::Path;

use anyhow::Context as AnyhowContext;
use anyhow::Result;
use tera::Context;
use tera::Tera;

pub fn render<S: AsRef<str>>(context: &Context, template: S, place: &str) -> Result<String> {
    Tera::one_off(template.as_ref(), context, false)
        .with_context(|| format!("failed to render template in {}", place))
}

pub fn renders<S: AsRef<str>>(
    context: &Context,
    templates: &[S],
    place: &str,
) -> Result<Vec<String>> {
    let mut strings = Vec::with_capacity(templates.len());

    for template in templates {
        strings.push(render(context, template, place)?);
    }

    Ok(strings)
}

pub fn render_file<T: AsRef<Path>, R: AsRef<Path>>(
    context: &Context,
    template_file: T,
    rendered_file: R,
    place: &str,
) -> Result<()> {
    let template = fs::read_to_string(template_file)?;
    let rendered = render(context, &template, place)?;
    fs::write(rendered_file, &rendered.as_bytes())?;

    Ok(())
}

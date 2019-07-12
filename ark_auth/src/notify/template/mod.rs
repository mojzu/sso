use crate::notify::Error;
use handlebars::Handlebars;

const EMAIL_HTML: &str = "email_html";
const EMAIL_TEXT: &str = "email_text";

/// Register template strings.
pub fn register(registry: &mut Handlebars) -> Result<(), Error> {
    registry
        .register_template_string(EMAIL_HTML, include_str!("email_html.hbs"))
        .map_err(Error::HandlebarsTemplate)?;
    registry
        .register_template_string(EMAIL_TEXT, include_str!("email_text.hbs"))
        .map_err(Error::HandlebarsTemplate)?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Email {
    pub title: String,
    pub text: String,
    pub url_text: String,
    pub url: String,
    pub service_name: String,
    pub service_url: String,
}

/// Render email HTML and text templates with parameters.
pub fn email(registry: &Handlebars, parameters: &Email) -> Result<(String, String), Error> {
    let text = email_text(registry, parameters)?;
    let html = email_html(registry, parameters)?;
    Ok((text, html))
}

fn email_html(registry: &Handlebars, parameters: &Email) -> Result<String, Error> {
    registry
        .render(EMAIL_HTML, parameters)
        .map_err(Error::HandlebarsRender)
}

fn email_text(registry: &Handlebars, parameters: &Email) -> Result<String, Error> {
    registry
        .render(EMAIL_TEXT, parameters)
        .map_err(Error::HandlebarsRender)
}

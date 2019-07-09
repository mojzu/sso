use crate::server::Error;
use handlebars::Handlebars;

const EMAIL_HTML: &str = include_str!("email_html.hbs");
const EMAIL_TEXT: &str = include_str!("email_text.hbs");

const FONT_FAMILY: &str = "-apple-system,BlinkMacSystemFont,'avenir next',avenir,'helvetica neue',helvetica,ubuntu,roboto,noto,'segoe ui',arial,sans-serif;";

#[derive(Debug, Serialize, Deserialize)]
pub struct Email {
    pub font_family: String,
    pub content_title: String,
    pub content_text: String,
    pub content_url_text: String,
    pub content_url: String,
    pub service_name: String,
    pub service_url: String,
}

impl Email {
    pub fn new(
        title: &str,
        text: &str,
        url_text: &str,
        url: &str,
        service_name: &str,
        service_url: &str,
    ) -> Self {
        Email {
            font_family: FONT_FAMILY.to_owned(),
            content_title: title.to_owned(),
            content_text: text.to_owned(),
            content_url_text: url_text.to_owned(),
            content_url: url.to_owned(),
            service_name: service_name.to_owned(),
            service_url: service_url.to_owned(),
        }
    }
}

pub fn email(parameters: &Email) -> Result<(String, String), Error> {
    let text = email_text(parameters)?;
    let html = email_html(parameters)?;
    Ok((text, html))
}

fn email_html(parameters: &Email) -> Result<String, Error> {
    let reg = Handlebars::new();
    reg.render_template(EMAIL_HTML, parameters)
        .map_err(Error::HandlebarsTemplateRender)
}

fn email_text(parameters: &Email) -> Result<String, Error> {
    let reg = Handlebars::new();
    reg.render_template(EMAIL_TEXT, parameters)
        .map_err(Error::HandlebarsTemplateRender)
}

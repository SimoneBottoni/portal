use anyhow::Result;
use askama::Template;

use crate::mailer::{EmailTemplate, RenderedEmail};

#[derive(Template)]
#[template(path = "mailer/welcome/text.txt")]
struct WelcomeText<'a> {
    domain: &'a str,
    token: &'a str,
}

#[derive(Template)]
#[template(path = "mailer/welcome/html.html")]
struct WelcomeHtml<'a> {
    domain: &'a str,
    token: &'a str,
}

#[derive(Template)]
#[template(path = "mailer/welcome/subject.txt")]
struct WelcomeSubject;

pub struct WelcomeEmail<'a> {
    pub domain: &'a str,
    pub token: &'a str,
}

impl<'a> EmailTemplate for WelcomeEmail<'a> {
    fn render(&self) -> Result<RenderedEmail> {
        Ok(RenderedEmail {
            subject: WelcomeSubject.render()?,
            text_body: WelcomeText {
                domain: self.domain,
                token: self.token,
            }
            .render()?,
            html_body: WelcomeHtml {
                domain: self.domain,
                token: self.token,
            }
            .render()?,
        })
    }
}

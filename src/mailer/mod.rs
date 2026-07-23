use anyhow::Context;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, MultiPart, SinglePart, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use secrecy::ExposeSecret;

use crate::{config::MailerConfig, mailer::auth::welcome::WelcomeEmail};

pub mod auth;

pub struct RenderedEmail {
    pub subject: String,
    pub text_body: String,
    pub html_body: String,
}

pub trait EmailTemplate {
    fn render(&self) -> anyhow::Result<RenderedEmail>;
}

#[derive(Debug, Clone)]
pub struct MailerContext {
    pub mailer: AsyncSmtpTransport<Tokio1Executor>,
    pub mailbox: Mailbox, // From
    pub domain: String,
}

impl MailerContext {
    pub fn build(config: &MailerConfig) -> Self {
        let mailbox: Mailbox = config.from.parse().expect("Failed to build from Mailbox.");

        let creds = Credentials::new(
            config.username.to_owned(),
            config.api_key.expose_secret().to_string(),
        );

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
            .expect("Failed to build mailer.")
            .credentials(creds)
            .build();

        Self {
            mailer,
            mailbox,
            domain: config.domain.clone(),
        }
    }

    async fn send_email<T>(&self, to: &str, template: T) -> anyhow::Result<()>
    where
        T: EmailTemplate,
    {
        let rendered = template
            .render()
            .context("Failed to render email template.")?;

        let to = to.parse().context("Failed to convert to email.")?;

        let email = Message::builder()
            .from(self.mailbox.clone())
            .to(to)
            .subject(rendered.subject.trim())
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(rendered.text_body),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(rendered.html_body),
                    ),
            )
            .context("Failed to build email message.")?;

        self.mailer
            .send(email)
            .await
            .context("Failed to send email.")?;

        Ok(())
    }

    pub async fn send_welcome(&self, to: &str, token: &str) -> anyhow::Result<()> {
        let template = WelcomeEmail {
            domain: &self.domain,
            token,
        };
        self.send_email(to, template).await
    }
}

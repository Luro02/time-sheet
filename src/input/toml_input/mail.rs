use std::borrow::Cow;

use lettre::message::{Mailbox, MessageBuilder};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::SmtpTransport;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct MailAddress<'a> {
    name: Cow<'a, str>,
    email: Cow<'a, str>,
}

impl<'a> MailAddress<'a> {
    pub fn new(name: impl Into<Cow<'a, str>>, email: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
        }
    }
}

impl<'a> From<MailAddress<'a>> for Mailbox {
    fn from(MailAddress { name, email }: MailAddress<'a>) -> Self {
        Self::new(
            Some(name.into_owned()),
            email.parse().expect("Mail should be valid"),
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Smtp {
    url: String,
    username: String,
    password: String,
    #[serde(default)]
    use_starttls: bool,
}

impl Smtp {
    #[must_use]
    pub fn to_transport(&self) -> SmtpTransport {
        let relay = self.url.as_str();
        let transport = {
            if self.use_starttls {
                SmtpTransport::starttls_relay(relay)
            } else {
                SmtpTransport::relay(relay)
            }
        }
        .unwrap();

        transport
            .credentials(Credentials::new(
                self.username.clone(),
                self.password.clone(),
            ))
            .build()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Mail {
    from: MailAddress<'static>,
    smtp: Smtp,
}

impl Mail {
    #[must_use]
    pub fn builder(&self) -> MessageBuilder {
        MessageBuilder::new().from(self.from.clone().into())
    }

    pub fn to_transport(&self) -> SmtpTransport {
        self.smtp.to_transport()
    }
}

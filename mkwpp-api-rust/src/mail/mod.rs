use mail_send::{mail_builder::MessageBuilder, smtp::message::IntoMessage, Credentials, SmtpClientBuilder};

const MKWPP_NAME: &'static str = "Mario Kart Wii Players' Page";
const MKWPP_EMAIL: &'static str = "no-reply@mariokart64.com";

pub struct MailService;
impl MailService {
    async fn send_message(message: impl IntoMessage<'_>) -> Result<(), anyhow::Error> {
        SmtpClientBuilder::new(
            crate::ENV_VARS.smtp_hostname.as_str(),
            crate::ENV_VARS.smtp_port,
        )
        .implicit_tls(false)
        .credentials(Credentials::new(
            crate::ENV_VARS.smtp_creds_name.as_str(),
            crate::ENV_VARS.smtp_creds_secret.as_str(),
        ))
        .allow_invalid_certs()
        .connect()
        .await?
        .send(message.into_message()?)
        .await?;
        
        Ok(())
    }
    
    pub async fn account_verification(username: &str, email: &str, token: &str) -> Result<(), anyhow::Error>{
        Self::send_message(MessageBuilder::new()
            .from((MKWPP_NAME, MKWPP_EMAIL))
            .to((username, email))
            .subject("Account Verification")
            .text_body(format!(
                r#"
                Hi {username},
                
                Your Mario Kart Wii Players' Page account has been successfully created.
                
                To activate your account, please visit the following link:
                
                https://mariokart64.com/mkw/activate?tkn={token}
                
                Happy karting!
                "#
            ))).await
    }
    
    pub async fn password_reset(username: &str, email: &str, token: &str) -> Result<(), anyhow::Error>{
        Self::send_message(MessageBuilder::new()
            .from((MKWPP_NAME, MKWPP_EMAIL))
            .to((username, email))
            .subject("Account Verification")
            .text_body(format!(
                r#"
                Hi {username},
                
                Someone requested a password reset on your Mario Kart Wii Players' Page account.
                If you did not perform this action, you may safely ignore this email.
                
                To reset your password, please visit the following link:
                
                https://mariokart64.com/mkw/password/reset?tkn={token}
                
                Please note this link will expire in 15 minutes.
                
                Happy karting!
                "#
            ))).await
    }
}

use resend_rs::types::CreateEmailBaseOptions;
use resend_rs::Resend;

use crate::errors::AppError;

// sends a 6-digit otp verification email via the resend api
pub async fn send_otp_email(to: &str, otp: &str) -> Result<(), AppError> {
    let api_key = std::env::var("RESEND_API_KEY")
        .map_err(|_| AppError::InternalError("RESEND_API_KEY not set".to_string()))?;

    let from = std::env::var("RESEND_FROM_EMAIL")
        .unwrap_or_else(|_| "SUST CP Geeks <onboarding@resend.dev>".to_string());

    let resend = Resend::new(&api_key);

    let subject = "Your SUST CP Geeks verification code";

    let html_body = format!(
        r#"
        <div style="font-family: 'Segoe UI', Arial, sans-serif; max-width: 480px; margin: 0 auto; padding: 32px;">
            <h2 style="color: #1a1a2e; margin-bottom: 8px;">Verify your email</h2>
            <p style="color: #555; font-size: 15px;">
                Use the code below to verify your SUST CP Geeks account.
                It expires in <strong>10 minutes</strong>.
            </p>
            <div style="background: #f4f4f8; border-radius: 8px; padding: 24px; text-align: center; margin: 24px 0;">
                <span style="font-size: 36px; font-weight: 700; letter-spacing: 8px; color: #1a1a2e;">{}</span>
            </div>
            <p style="color: #888; font-size: 13px;">
                If you didn't request this, you can safely ignore this email.
            </p>
        </div>
        "#,
        otp
    );

    let email = CreateEmailBaseOptions::new(&from, [to], subject).with_html(&html_body);

    resend.emails.send(email).await.map_err(|e| {
        tracing::error!("failed to send email via resend: {}", e);
        AppError::InternalError(format!("Failed to send verification email: {}", e))
    })?;

    tracing::info!("otp email sent to {}", to);
    Ok(())
}

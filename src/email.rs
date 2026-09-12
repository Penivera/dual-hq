use std::sync::Arc;
use lettre::message::{Mailbox, Message, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

use crate::config::Config;

/// Core SMTP sending function with support for direct TLS (port 465) and STARTTLS (port 587).
pub async fn send_email_smtp(
    config: &Config,
    to_email: &str,
    to_name: Option<&str>,
    subject: &str,
    html_body: &str,
) -> Result<(), String> {
    if !config.smtp_enabled || config.smtp_user.trim().is_empty() || config.smtp_password.trim().is_empty() {
        tracing::info!("SMTP is disabled or credentials not provided in .env, skipping email to {}", to_email);
        return Ok(());
    }

    let from_mailbox: Mailbox = format!("{} <{}>", config.smtp_from_name, config.smtp_from_email)
        .parse()
        .map_err(|e| format!("Invalid FROM email configuration: {e}"))?;

    let to_mailbox: Mailbox = match to_name {
        Some(name) if !name.trim().is_empty() => format!("{} <{}>", name.trim(), to_email.trim())
            .parse()
            .map_err(|e| format!("Invalid TO email format: {e}"))?,
        _ => to_email
            .trim()
            .parse()
            .map_err(|e| format!("Invalid TO email format: {e}"))?,
    };

    let email = Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(SinglePart::plain(format!(
                    "{subject}\n\nPlease view this email in an HTML-capable email client."
                )))
                .singlepart(SinglePart::html(html_body.to_string())),
        )
        .map_err(|e| format!("Failed to build email message: {e}"))?;

    let creds = Credentials::new(config.smtp_user.clone(), config.smtp_password.clone());

    let mailer = if config.smtp_port == 465 {
        // Port 465: direct TLS
        AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)
            .map_err(|e| format!("Failed to initialize SMTP TLS relay: {e}"))?
            .port(config.smtp_port)
            .credentials(creds)
            .build()
    } else {
        // Port 587 or others: STARTTLS
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)
            .map_err(|e| format!("Failed to initialize SMTP STARTTLS relay: {e}"))?
            .port(config.smtp_port)
            .credentials(creds)
            .build()
    };

    mailer
        .send(email)
        .await
        .map_err(|e| format!("Failed to deliver email via SMTP: {e}"))?;

    tracing::info!("Successfully sent email to {} with subject '{}'", to_email, subject);
    Ok(())
}

/// Dispatches an email in the background without blocking the HTTP request handler.
pub fn spawn_email(
    config: Arc<Config>,
    to_email: String,
    to_name: Option<String>,
    subject: String,
    html_body: String,
) {
    tokio::spawn(async move {
        if let Err(err) = send_email_smtp(
            &config,
            &to_email,
            to_name.as_deref(),
            &subject,
            &html_body,
        )
        .await
        {
            tracing::error!("Failed to send background email to {}: {}", to_email, err);
        }
    });
}

/// Sends an account verification email to a newly registered student or user.
pub fn send_verification_email(
    config: Arc<Config>,
    to_email: String,
    to_name: String,
    verification_token: String,
) {
    let verify_url = format!(
        "{}/api/auth/verify?token={}",
        config.app_base_url.trim_end_matches('/'),
        verification_token
    );

    let subject = "Verify your email address - Internship Portal".to_string();
    let html_body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 24px; border: 1px solid #eaeaea; border-radius: 8px; }}
        .button {{ display: inline-block; padding: 12px 24px; background-color: #0070f3; color: #ffffff !important; text-decoration: none; border-radius: 5px; font-weight: 600; margin: 16px 0; }}
        .token-box {{ background-color: #f5f5f5; padding: 12px; border-radius: 4px; font-family: monospace; font-size: 16px; margin: 16px 0; word-break: break-all; }}
        .footer {{ margin-top: 24px; font-size: 12px; color: #666; border-top: 1px solid #eaeaea; padding-top: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <h2>Welcome to the Internship Application System!</h2>
        <p>Hi {to_name},</p>
        <p>Thank you for registering. Please verify your email address to activate your account and apply for internships.</p>
        <p><a href="{verify_url}" class="button">Verify Email Address</a></p>
        <p>Or click this link directly: <a href="{verify_url}">{verify_url}</a></p>
        <p>Your verification token:</p>
        <div class="token-box">{verification_token}</div>
        <div class="footer">
            <p>If you did not create an account, you can safely ignore this email.</p>
        </div>
    </div>
</body>
</html>"#
    );

    spawn_email(config, to_email, Some(to_name), subject, html_body);
}

/// Sends an application submission confirmation to the applicant.
pub fn send_application_submitted_email(
    config: Arc<Config>,
    applicant_email: String,
    applicant_name: String,
    opportunity_title: String,
    company_name: String,
) {
    let subject = format!("Application Submitted: {opportunity_title} at {company_name}");
    let html_body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 24px; border: 1px solid #eaeaea; border-radius: 8px; }}
        .badge {{ display: inline-block; padding: 4px 10px; background-color: #e6f7ff; color: #1890ff; border-radius: 4px; font-weight: 500; }}
        .footer {{ margin-top: 24px; font-size: 12px; color: #666; border-top: 1px solid #eaeaea; padding-top: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <h2>Application Received!</h2>
        <p>Hi {applicant_name},</p>
        <p>Your application for <strong>{opportunity_title}</strong> at <strong>{company_name}</strong> has been successfully received.</p>
        <p>Status: <span class="badge">Pending Review</span></p>
        <p>The hiring team will review your submission and you will be notified of any updates.</p>
        <div class="footer">
            <p>Internship Application Portal &copy; 2026</p>
        </div>
    </div>
</body>
</html>"#
    );

    spawn_email(config, applicant_email, Some(applicant_name), subject, html_body);
}

/// Sends an application status update notification to the applicant.
pub fn send_application_status_update_email(
    config: Arc<Config>,
    applicant_email: String,
    applicant_name: String,
    opportunity_title: String,
    company_name: String,
    new_status: String,
) {
    let subject = format!("Application Update: {opportunity_title} ({new_status})");
    let status_color = match new_status.to_lowercase().as_str() {
        "accepted" => "#52c41a",
        "rejected" => "#ff4d4f",
        "under_review" | "reviewed" => "#faad14",
        _ => "#1890ff",
    };

    let html_body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 24px; border: 1px solid #eaeaea; border-radius: 8px; }}
        .status-badge {{ display: inline-block; padding: 6px 14px; background-color: {status_color}; color: white; border-radius: 4px; font-weight: 600; font-size: 16px; margin: 12px 0; }}
        .footer {{ margin-top: 24px; font-size: 12px; color: #666; border-top: 1px solid #eaeaea; padding-top: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <h2>Application Status Update</h2>
        <p>Hi {applicant_name},</p>
        <p>The status of your application for <strong>{opportunity_title}</strong> at <strong>{company_name}</strong> has been updated:</p>
        <div><span class="status-badge">{new_status}</span></div>
        <p>Please log in to your dashboard to review any further instructions.</p>
        <div class="footer">
            <p>Internship Application Portal &copy; 2026</p>
        </div>
    </div>
</body>
</html>"#
    );

    spawn_email(config, applicant_email, Some(applicant_name), subject, html_body);
}

/// Sends a new internship opportunity notification to students.
pub fn send_new_opportunity_notification_email(
    config: Arc<Config>,
    student_email: String,
    student_name: String,
    title: String,
    company_name: String,
    location: String,
    opp_type: String,
    stipend: Option<String>,
    opportunity_id: i32,
) {
    let view_url = format!("{}/opportunities/{}", config.app_base_url.trim_end_matches('/'), opportunity_id);
    let subject = format!("New Opportunity: {title} at {company_name}");
    let stipend_text = stipend.unwrap_or_else(|| "Not specified".to_string());

    let html_body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 24px; border: 1px solid #eaeaea; border-radius: 8px; }}
        .card {{ background-color: #fafafa; border: 1px solid #eee; border-radius: 6px; padding: 16px; margin: 16px 0; }}
        .button {{ display: inline-block; padding: 10px 20px; background-color: #0070f3; color: #ffffff !important; text-decoration: none; border-radius: 4px; font-weight: 600; margin-top: 12px; }}
        .footer {{ margin-top: 24px; font-size: 12px; color: #666; border-top: 1px solid #eaeaea; padding-top: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <h2>New Opportunity Listed!</h2>
        <p>Hi {student_name},</p>
        <p>A new opportunity matching your profile has just been published on the portal:</p>
        <div class="card">
            <h3 style="margin-top: 0;">{title}</h3>
            <p><strong>Company:</strong> {company_name}</p>
            <p><strong>Location:</strong> {location}</p>
            <p><strong>Type:</strong> {opp_type}</p>
            <p><strong>Stipend:</strong> {stipend_text}</p>
            <a href="{view_url}" class="button">View & Apply</a>
        </div>
        <div class="footer">
            <p>You received this because you are registered as a student on the Internship Application Portal.</p>
        </div>
    </div>
</body>
</html>"#
    );

    spawn_email(config, student_email, Some(student_name), subject, html_body);
}

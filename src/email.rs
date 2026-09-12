use std::sync::Arc;
use askama::Template;
use lettre::message::{Mailbox, Message, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

use crate::config::Config;

// ============================================================================
// Askama Email Templates
// ============================================================================

#[derive(Template)]
#[template(path = "verification.html")]
pub struct VerificationTemplate<'a> {
    pub user_name: &'a str,
    pub verify_url: &'a str,
    pub verification_token: &'a str,
}

#[derive(Template)]
#[template(path = "welcome.html")]
pub struct WelcomeTemplate<'a> {
    pub user_name: &'a str,
    pub dashboard_url: &'a str,
}

#[derive(Template)]
#[template(path = "new_opportunity.html")]
pub struct NewOpportunityTemplate<'a> {
    pub student_name: &'a str,
    pub title: &'a str,
    pub company_name: &'a str,
    pub location: &'a str,
    pub opp_type: &'a str,
    pub stipend: &'a str,
    pub view_url: &'a str,
}

#[derive(Template)]
#[template(path = "application_accepted.html")]
pub struct ApplicationAcceptedTemplate<'a> {
    pub applicant_name: &'a str,
    pub opportunity_title: &'a str,
    pub company_name: &'a str,
    pub next_steps: &'a str,
    pub dashboard_url: &'a str,
}

#[derive(Template)]
#[template(path = "application_submitted.html")]
pub struct ApplicationSubmittedTemplate<'a> {
    pub applicant_name: &'a str,
    pub opportunity_title: &'a str,
    pub company_name: &'a str,
    pub dashboard_url: &'a str,
}

#[derive(Template)]
#[template(path = "application_status.html")]
pub struct ApplicationStatusTemplate<'a> {
    pub applicant_name: &'a str,
    pub opportunity_title: &'a str,
    pub company_name: &'a str,
    pub new_status: &'a str,
    pub status_bg: &'a str,
    pub status_fg: &'a str,
    pub dashboard_url: &'a str,
}

// ============================================================================
// Core SMTP Sending Logic
// ============================================================================

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

/// Spawns an asynchronous tokio task to send an email without blocking the route handler.
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

// ============================================================================
// Public Email Dispatchers (Non-blocking Tokio tasks)
// ============================================================================

/// Sends an account verification email rendered via Askama in a background task.
pub fn send_verification_email(
    config: Arc<Config>,
    to_email: String,
    to_name: String,
    verification_token: String,
) {
    tokio::spawn(async move {
        let verify_url = format!(
            "{}/api/auth/verify?token={}",
            config.app_base_url.trim_end_matches('/'),
            verification_token
        );

        let template = VerificationTemplate {
            user_name: &to_name,
            verify_url: &verify_url,
            verification_token: &verification_token,
        };

        match template.render() {
            Ok(html_body) => {
                let subject = "Verify your email address - Internship Portal".to_string();
                if let Err(err) = send_email_smtp(&config, &to_email, Some(&to_name), &subject, &html_body).await {
                    tracing::error!("Failed to send verification email to {}: {}", to_email, err);
                }
            }
            Err(err) => {
                tracing::error!("Failed to render verification email template: {}", err);
            }
        }
    });
}

/// Sends a welcome email upon successful verification rendered via Askama in a background task.
pub fn send_welcome_email(
    config: Arc<Config>,
    to_email: String,
    to_name: String,
) {
    tokio::spawn(async move {
        let dashboard_url = format!("{}/dashboard", config.app_base_url.trim_end_matches('/'));

        let template = WelcomeTemplate {
            user_name: &to_name,
            dashboard_url: &dashboard_url,
        };

        match template.render() {
            Ok(html_body) => {
                let subject = "Welcome to Internship Portal! Account Verified 🎉".to_string();
                if let Err(err) = send_email_smtp(&config, &to_email, Some(&to_name), &subject, &html_body).await {
                    tracing::error!("Failed to send welcome email to {}: {}", to_email, err);
                }
            }
            Err(err) => {
                tracing::error!("Failed to render welcome email template: {}", err);
            }
        }
    });
}

/// Sends an application accepted congratulatory email rendered via Askama in a background task.
pub fn send_application_accepted_email(
    config: Arc<Config>,
    applicant_email: String,
    applicant_name: String,
    opportunity_title: String,
    company_name: String,
    next_steps: Option<String>,
) {
    tokio::spawn(async move {
        let dashboard_url = format!("{}/dashboard/applications", config.app_base_url.trim_end_matches('/'));
        let steps = next_steps.unwrap_or_else(|| {
            "Please log in to your dashboard to review onboarding materials, schedule your orientation, or contact the hiring manager.".to_string()
        });

        let template = ApplicationAcceptedTemplate {
            applicant_name: &applicant_name,
            opportunity_title: &opportunity_title,
            company_name: &company_name,
            next_steps: &steps,
            dashboard_url: &dashboard_url,
        };

        match template.render() {
            Ok(html_body) => {
                let subject = format!("Congratulations! Application Accepted for {opportunity_title} at {company_name} 🎉");
                if let Err(err) = send_email_smtp(&config, &applicant_email, Some(&applicant_name), &subject, &html_body).await {
                    tracing::error!("Failed to send application accepted email to {}: {}", applicant_email, err);
                }
            }
            Err(err) => {
                tracing::error!("Failed to render application accepted email template: {}", err);
            }
        }
    });
}

/// Sends an application submission confirmation email rendered via Askama in a background task.
pub fn send_application_submitted_email(
    config: Arc<Config>,
    applicant_email: String,
    applicant_name: String,
    opportunity_title: String,
    company_name: String,
) {
    tokio::spawn(async move {
        let dashboard_url = format!("{}/dashboard/applications", config.app_base_url.trim_end_matches('/'));

        let template = ApplicationSubmittedTemplate {
            applicant_name: &applicant_name,
            opportunity_title: &opportunity_title,
            company_name: &company_name,
            dashboard_url: &dashboard_url,
        };

        match template.render() {
            Ok(html_body) => {
                let subject = format!("Application Submitted: {opportunity_title} at {company_name}");
                if let Err(err) = send_email_smtp(&config, &applicant_email, Some(&applicant_name), &subject, &html_body).await {
                    tracing::error!("Failed to send application submitted email to {}: {}", applicant_email, err);
                }
            }
            Err(err) => {
                tracing::error!("Failed to render application submitted email template: {}", err);
            }
        }
    });
}

/// Sends an application status update notification (delegates to accepted template if status is accepted).
pub fn send_application_status_update_email(
    config: Arc<Config>,
    applicant_email: String,
    applicant_name: String,
    opportunity_title: String,
    company_name: String,
    new_status: String,
) {
    if new_status.eq_ignore_ascii_case("accepted") {
        send_application_accepted_email(
            config,
            applicant_email,
            applicant_name,
            opportunity_title,
            company_name,
            None,
        );
        return;
    }

    tokio::spawn(async move {
        let dashboard_url = format!("{}/dashboard/applications", config.app_base_url.trim_end_matches('/'));
        let (status_bg, status_fg) = match new_status.to_lowercase().as_str() {
            "rejected" => ("#fff1f0", "#f5222d"),
            "under_review" | "reviewed" => ("#fffbe6", "#d48806"),
            _ => ("#e6f7ff", "#1890ff"),
        };

        let template = ApplicationStatusTemplate {
            applicant_name: &applicant_name,
            opportunity_title: &opportunity_title,
            company_name: &company_name,
            new_status: &new_status,
            status_bg,
            status_fg,
            dashboard_url: &dashboard_url,
        };

        match template.render() {
            Ok(html_body) => {
                let subject = format!("Application Update: {opportunity_title} ({new_status})");
                if let Err(err) = send_email_smtp(&config, &applicant_email, Some(&applicant_name), &subject, &html_body).await {
                    tracing::error!("Failed to send application status email to {}: {}", applicant_email, err);
                }
            }
            Err(err) => {
                tracing::error!("Failed to render application status email template: {}", err);
            }
        }
    });
}

/// Sends a new internship opportunity notification to students rendered via Askama in a background task.
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
    tokio::spawn(async move {
        let view_url = format!("{}/opportunities/{}", config.app_base_url.trim_end_matches('/'), opportunity_id);
        let stipend_text = stipend.unwrap_or_else(|| "Not specified".to_string());

        let template = NewOpportunityTemplate {
            student_name: &student_name,
            title: &title,
            company_name: &company_name,
            location: &location,
            opp_type: &opp_type,
            stipend: &stipend_text,
            view_url: &view_url,
        };

        match template.render() {
            Ok(html_body) => {
                let subject = format!("New Opportunity: {title} at {company_name}");
                if let Err(err) = send_email_smtp(&config, &student_email, Some(&student_name), &subject, &html_body).await {
                    tracing::error!("Failed to send new opportunity email to {}: {}", student_email, err);
                }
            }
            Err(err) => {
                tracing::error!("Failed to render new opportunity email template: {}", err);
            }
        }
    });
}

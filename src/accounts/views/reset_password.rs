use jelly::prelude::*;
use jelly::accounts::User;
use jelly::actix_web::{HttpRequest, web::{Path, Form}};
use jelly::djangohashers::{make_password};
use jelly::Result;

use crate::accounts::Account;
use crate::accounts::forms::{EmailForm, ChangePasswordForm};
use crate::accounts::jobs::{SendPasswordWasResetEmail, SendResetPasswordEmail};
use crate::accounts::views::utils::validate_token;

/// Just renders a standard "Enter Your Email" password reset page.
pub async fn form(request: HttpRequest) -> Result<HttpResponse> {
    request.render(200, "accounts/reset_password/index.html", {
        let mut context = Context::new();
        context.insert("form", &EmailForm::default());
        context.insert("sent", &false);
        context
    })
}

/// Processes the reset password request, which ultimately just passes
/// it to a background worker to execute - we do this to avoid any timing
/// attacks re: leaking user existence.
pub async fn request_reset(
    request: HttpRequest,
    form: Form<EmailForm>
) -> Result<HttpResponse> {
    let mut form = form.into_inner();
    if !form.is_valid() {
        return request.render(400, "accounts/reset_password/index.html", {
            let mut context = Context::new();
            context.insert("form", &form);
            context.insert("sent", &false);
            context
        });
    }

    request.queue(SendResetPasswordEmail {
        to: form.email.value
    })?;

    request.render(200, "accounts/reset_password/requested.html", {
        let mut context = Context::new();
        context.insert("sent", &true);
        context
    })
}

/// Given a link (of form {uidb64}-{ts}-{token}), verifies the
/// token and user, and presents them a change password form.
///
/// In general, we do not want to leak information, so any errors here
/// should simply report as "invalid or expired". It's a bit verbose, but
/// such is Rust for this type of thing. Write it once and move on. ;P
pub async fn with_token(
    request: HttpRequest,
    Path((uidb64, ts, token)): Path<(String, String, String)>
) -> Result<HttpResponse> {
    if let Ok(_account) = validate_token(&request, &uidb64, &ts, &token).await {
        return request.render(200, "accounts/reset_password/change_password.html", {
            let mut context = Context::new();
            context.insert("form", &ChangePasswordForm::default());
            context.insert("uidb64", &uidb64);
            context.insert("ts", &ts);
            context.insert("token", &token);
            context
        });
    }

    return request.render(200, "accounts/invalid_token.html", Context::new());
}

/// Verifies the password is fine, and if so, signs the user in and redirects
/// them to the dashboard with a flash message.
pub async fn reset(
    request: HttpRequest,
    Path((uidb64, ts, token)): Path<(String, String, String)>,
    form: Form<ChangePasswordForm>
) -> Result<HttpResponse> {
    let mut form = form.into_inner();
    
    if let Ok(account) = validate_token(&request, &uidb64, &ts, &token).await {       
        // Note! This is a case where we need to fetch the user ahead of form validation.
        // While it would be nice to avoid the DB hit, validating that their password is secure
        // requires pulling some account values...
        form.name = Some(account.name.clone());
        form.email = Some(account.email.clone());

        if !form.is_valid() {
            return request.render(200, "accounts/reset_password/change_password.html", {
                let mut context = Context::new();
                context.insert("form", &form);
                context
            });
        }

        let pool = request.db_pool()?;
        let password = make_password(&form.password);
        Account::update_password_and_last_login(account.id, &password, pool).await?;

        request.queue(SendPasswordWasResetEmail {
            to: account.email.clone()
        })?;

        request.set_user(User {
            id: account.id,
            name: account.name,
            is_admin: account.is_admin,
            is_anonymous: false
        })?;

        request.flash("Password Reset", "Your password was successfully reset.")?;
        return request.redirect("/dashboard/");
    }
    
    request.render(200, "accounts/reset_password/change_password.html", {
        let mut context = Context::new();
        context.insert("form", &form);
        context
    })
}

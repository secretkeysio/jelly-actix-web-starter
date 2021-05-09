use jelly::prelude::*;
use jelly::actix_web::{HttpRequest, web::Form};
use jelly::request::{Authentication, DatabasePool};
use jelly::Result;

use crate::accounts::Account;
use crate::accounts::jobs::{send_verify_account_email, send_odd_registration_attempt_email};
use crate::accounts::forms::NewAccountForm;

pub async fn form(request: HttpRequest) -> Result<HttpResponse> {
    if request.is_authenticated()? {
        return request.redirect("/dashboard/");
    }

    request.render(200, "accounts/register.html", {
        let mut ctx = Context::new();
        ctx.insert("form", &NewAccountForm::default());
        ctx
    })
}

pub async fn create_account(
    request: HttpRequest,
    form: Form<NewAccountForm>
) -> Result<HttpResponse> {
    if request.is_authenticated()? {
        return request.redirect("/dashboard/");
    }

    let mut form = form.into_inner();
    if !form.is_valid() {
        return request.render(400, "accounts/register.html", {
            let mut ctx = Context::new();
            ctx.insert("form", &form);
            ctx
        });
    }

    // Catch this error
    // if duplicate:
    //  - send email to existing user asking if they were trying to sign in
    //  - pass requesting user through normal "fake" flow to avoid leaking if
    //      an account exists?
    let db = request.db_pool()?;
    match Account::register(&form, db).await {
        Ok(uid) => {
            send_verify_account_email.builder()
                .set_json(&uid)?
                .spawn(db)
                .await?;
        },

        Err(e) => {
            error!("Error with registering: {:?}", e);

            send_odd_registration_attempt_email.builder()
                .set_json(&form.email.value)?
                .spawn(db)
                .await?;
        }
    }

    // No matter what, just appear as if it worked.
    request.redirect("/accounts/verify/")
}

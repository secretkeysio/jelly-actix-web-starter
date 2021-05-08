# Jelly
A.K.A, the actix-web starter you probably wish you had. This is provided as-is, and 
anyone is free to extend it or rework it as they desire - just give some credit if
you base a web framework off of it. :)

A disclaimer: this is used internally, and while it's very usable, it might not be
perfect. You might need to tweak a thing or two. Don't be shocked if you need to
alter `jelly` for your own needs. Pull requests for things that should be "standard"
are welcome.

## Notice Re: Licensing
The current crate used in this repository for background jobs has a potential licensing issue, depending on what you're looking to do with the code here. The _refresh_ branch of this repository will change this, but the merge is pending some things calming down in actix-web 4.0. [Learn more here](https://github.com/secretkeysio/jelly-actix-web-starter/issues/9)

In the meantime, you have two options:

- You can see if what you're doing gels with the license on background-jobs
- If not, you could just rip out background jobs and your HTTP responses on endpoints that use a background job (dispatching emails) will be slightly longer until the refresh branch is merged.

I've personally done the latter, and isn't too bad - hopefully, this passes soon. If the actix-web 4.0 beta's hit a stable state I'm also open to merging the refresh branch temporarily pinned to working betas.

## What's This?
If you've ever written a web service in Rust that's needed some of
the following:

- User accounts and authentication
- Form handling
- Background jobs
- Transactional emailing
- Flash messages
- Async Postgres database (via SQLx)

Then Jelly may be of interest to you. It's explicitly _not_ a framework; 
it's modeled after Python's Django but tries to not hide the underlying
actix-web framework too much. This is done because web frameworks 
traditionally fall into two categories:

- **The kitchen sink**: it has literally everything, and once you need to
    scale it, you start ripping things out and getting slightly annoyed.
- **The micro framework**: Works great for an API service. Absolutely sucks
    when you start reimplementing the kitchen sink.

Jelly tries to sit in-between those two ideas; think of it as a meta-framework
for actix-web. It helps you structure the app and spend less up-front time 
configuring and tweaking things, and brings important ("table stakes") pieces
to the Rust web dev experience.

**tl;dr**: CRUD web applications are boring, so consider using this to get to
the interesting parts.

## Getting Started
- Clone this repository.
- Edit `Cargo.toml` to configure your project name, as well as any other settings you need.
- Ensure you have Postgresql installed.
- Install `sqlx-cli`, with: 

```
cargo install sqlx-cli --no-default-features --features postgres
```

- Edit `.env.example` to use your settings.
- Run the account migrations with `sqlx migrate run`.
- Run the server:

```
cargo run

# Optionally, if you use cargo-watch:
cargo-watch -i templates -i static -i migrations -x run
```

If you're ready to push a release build, you probably want to run:

```
cargo build --release --no-default-features --features production
```

For configuring email dispatch, see the README in `email_templates`.

## Accounts
Accounts is modeled to provide the most common features you would expect from a user
system. It provides the following:

- Registration
    - On signup, will dispatch an email for account verification.
- Login
    - Password reset functionality is also provided.

Both password reset and account verification implement a one-time-use URL pattern. The
flow is a mirror of what Django does; the URL is hashed based on account information, so
once the password changes, the URL becomes invalid.

Registration and Login, by default, try to not leak that an existing user account might exist.
If a user attempts to register with an already registered email address, the following will happen:

- The person attempting to register will be shown the "normal" flow, as if they successfully signed up, being told to check their email to verify.
- The already registered user is sent an email notifying that this happened, and includes a link to password reset - e.g, maybe they're a confused user who just needs to get back in.

## Templates
Templates are written in [Tera](https://github.com/Keats/tera). If you've written templates in Django or Jinja2, they should be _very_ familiar.

The provided `templates` has a top-level `layout.html`, which should be your global public layout. The `templates/dashboard` folder is what a user sees upon logging in.

In development, your templates are automatically reloaded on edit. Jelly also provides a stock "an error happened" view, similar to what Django does.

In production, both of these are disabled.

## Static
The `static` folder is where you can place any static things. In development, [actix-files]() is preconfigured to serve content from that directory, in order to make life easier for just running on your machine. This is disabled in the `production` build, mostly because we tend to shove this behind Nginx. You can swap this as needed.

## Forms
Writing the same email/password/etc verification logic is a chore, and one of the nicer things Django has is Form helpers for this type of thing. If you miss that, Jelly has a forms-ish module that you can use.

For instance, you could do:

**forms.rs**
``` rust
use serde::{Deserialize, Serialize};
use jelly::forms::{EmailField, PasswordField, Validation};

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct LoginForm {
    pub email: EmailField,
    pub password: PasswordField
}

impl Validation for LoginForm {
    fn is_valid(&mut self) -> bool {
        self.email.is_valid() && !self.password.value.is_empty()
    }
}
```

**views.rs**
``` rust
/// POST-handler for logging in.
pub async fn authenticate(
    request: HttpRequest,
    form: Form<LoginForm>
) -> Result<HttpResponse> {
    if request.is_authenticated()? {
        return request.redirect("/dashboard/");
    }

    let mut form = form.into_inner();
    if !form.is_valid() {
        return request.render(400, "accounts/login.html", {
            let mut context = Context::new();
            context.insert("error", "Invalid email or password.");
            context.insert("form", &form);
            context
        });
    }
```

In this case, `EmailField` will check that the email is a mostly-valid email address. `PasswordField` will check that it's a "secure" password. Each `Field` type has an internal `errors` stack, so you can pass it back to your view and render errors as necessary.

For more supported field types, see the `jelly/forms` module.

## Request Helpers
A personal pet peeve: the default actix-web view definitions are mind-numbingly verbose. Code is read far more than it's written, and thus Jelly includes some choices to make writing views less of a headache: namely, access to things like database pools and authentication are implemented as traits on `HttpRequest`.

This makes the necessary view imports a bit cleaner, requiring just the prelude for some traits, and makes view definitons much cleaner overall. It's important to note that if, for whatever reason, you need to use standard actix-web view definitions, you totally can - Jelly doesn't restrict this, just provides a (we think) nicer alternative.

### Checking a User
You can call `request.is_authenticated()?` to check if a User is authenticated. This does not incur a database hit, but simply checks against the signed cookie session value.

You can call `request.user()?` to get the `User` for a request. This does not incur a database hit, and just loads cached information from the signed cookie session. Users are, by default anonymous - and can be checked with `is_anonymous`.

If you want the _full_ user Account object, you can call `Account::get(user.id, &db_pool).await?`, or write your own method.

You can restrict access to only authenticated users on a URL basis by using `jelly::guards::Auth`; example usage can be found in `src/dashboard/mod.rs`.

### Rendering a Template
You can call `request.render(http_code, template_path, model)`, where:

- `http_code` is an HTTP response code.
- `template_path` is a relative path to the template you want to load.
- `model` is a `tera::Context`.

_Why is `http_code` just passing a number?`_, you might ask. It's personal preference, mostly: developers are intelligent enough to know what an HTTP response code is, and it's far less verbose to just pass the number - and simple enough to scan when you're trying to track down something related to it.

`request.render()` makes two things available to you by default:

- `user`, which is the current `User` instance from the signed cookie session.
- `flash_messages`, which are one-time messages that you can have on a view.

### Returning a JSON response
You can call `request.json(http_code, obj)`, where `objc` is an object that can be serialized to JSON.

### Returning a Redirect
You can call `request.redirect(path)`, where `path` is where you want the user to go.

### Setting a Flash Message
You can call `request.flash(title, message)` to add a Flash message to the request. This is a one-time message, typically used for, say, confirming that something worked.

### Getting a Database Pool
You can call `request.db_pool()?` to get a database pool instance. This can be passed to whatever you need to call for database work.

### Queuing a Background Job
You can use `accounts/jobs` for a basis to create your own background jobs, and register them similar to how they're done in `src/main.rs`.

You can call `request.queue(MyJob {...})?` to dispatch a job in the background.

## The End
Hopefully, this helps people get going with more web services in Rust, and provides a common base to work off of. There's three things to note here:

- This is released under a "do-whatever-you-want" license. Just give credit if you use it as the basis for a web framework of your own.
- Someone else is more than welcome to take this further and make a true web framework.
- I would argue that actix-web, Rocket, and so on should really just have this kind of thing by default.
- Thanks to every developer of a sub-crate used in this project; there are too many to list, but the Rust community is hands down one of the best out there.

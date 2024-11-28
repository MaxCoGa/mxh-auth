use askama::Template;
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use axum_messages::{Message, Messages};
use serde::Deserialize;

use crate::users::{AuthSession, Credentials};

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    messages: Vec<Message>,
    next: Option<String>,
}

#[derive(Template)]
#[template(path = "signin.html")]
pub struct SigninTemplate {
    messages: Vec<Message>,
    next: Option<String>,
}

// This allows us to extract the "next" field from the query string. We use this
// to redirect after log in.
#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}

pub fn router() -> Router<()> {
    Router::new()
        .route("/signin", post(self::post::signin))
        .route("/signin", get(self::get::signin))
        .route("/login", post(self::post::login))
        .route("/login", get(self::get::login))
        .route("/logout", get(self::get::logout))
}

mod post {
    use super::*;

    pub async fn signin(
        mut auth_session: AuthSession,
        messages: Messages,
        Form(creds): Form<Credentials>,
    ) -> impl IntoResponse {
        println!("signin {:?}", creds.clone());

        let user = match auth_session.authenticate(creds.clone()).await {
            Ok(Some(user)) => user, // User exists and authenticated
            Ok(None) => {          // User does not exist, try to add

                let backend = auth_session.backend.clone(); // Access the Backend from AuthSession

                // messages.error("Invalid credentials");
                // return redirect_to_login("/signin", creds.next).into_response();
                match backend.add_user(&creds.username, &creds.password).await {
                    Ok(_) => {
                        // User added, try to authenticate again
                        match auth_session.authenticate(creds.clone()).await {
                            Ok(Some(user)) => user,
                            Ok(None) => {
                                messages.error("Invalid credentials");
                                return redirect_to_login("/signin", creds.next).into_response();
                            }
                            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                        }
                    }
                    Err(err) => {
                        messages.error(format!("Failed to create user: {}", err));
                        return redirect_to_login("/signin", creds.next).into_response();
                    }
                }
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };       

        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        messages.success(format!("Account created! Successfully logged in as {}", user.username));

        redirect_to_next_or_root(creds.next).into_response()
    }

    pub async fn login(
        mut auth_session: AuthSession,
        messages: Messages,
        Form(creds): Form<Credentials>,
    ) -> impl IntoResponse {
        println!("login {:?}",creds.clone());
        let user = match auth_session.authenticate(creds.clone()).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                messages.error("Invalid credentials");
                return redirect_to_login("/login", creds.next).into_response();
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        messages.success(format!("Successfully logged in as {}", user.username));

        redirect_to_next_or_root(creds.next).into_response()
    }

    // Helper function to redirect to login page with optional next url
    fn redirect_to_login(path: &str, next: Option<String>) -> Redirect {
        let mut login_url = path.to_string();
        if let Some(next) = next {
            login_url = format!("{}?next={}", login_url, next);
        };
        Redirect::to(&login_url)
    }

    // Helper function to redirect to next url or root
    fn redirect_to_next_or_root(next: Option<String>) -> Redirect {
        if let Some(ref next) = next {
            Redirect::to(next)
        } else {
            Redirect::to("/")
        }
    }    
}

// use crate::web::protected::ProtectedTemplate;

mod get {
    use super::*;

    pub async fn signin(
        auth_session: AuthSession,
        messages: Messages,
        Query(NextUrl { next }): Query<NextUrl>,
    ) -> impl IntoResponse {
        if let Some(_user) = auth_session.user {
            println!("user is already login redirect to /");
            // let protected_template = ProtectedTemplate {
            //     messages: messages.into_iter().collect(),
            //     username: &user.username,
            // };
            // We're already logged in, redirect to the protected page
            return Redirect::to("/").into_response();
        } else {
            // Not logged in, show the login page
            // Continue to the rest of the function to render the login page
        };
        
        println!("signin page...");
        SigninTemplate {
            messages: messages.into_iter().collect(),
            next,
        }.into_response()
    }


    pub async fn login(
        auth_session: AuthSession,
        messages: Messages,
        Query(NextUrl { next }): Query<NextUrl>,
    ) -> impl IntoResponse {

        if let Some(_user) = auth_session.user {
            println!("user is already login redirect to /");
            // let protected_template = ProtectedTemplate {
            //     messages: messages.into_iter().collect(),
            //     username: &user.username,
            // };
            // We're already logged in, redirect to the protected page
            return Redirect::to("/").into_response();
        } else {
            // Not logged in, show the login page
            // Continue to the rest of the function to render the login page
        };
        
        println!("login page...");
        LoginTemplate {
            messages: messages.into_iter().collect(),
            next,
        }.into_response()
    }

    pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
        println!("logout page...");
        match auth_session.logout().await {
            Ok(_) => Redirect::to("/login").into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
//! frontend/rest/services/authentication.rs
//!
//! Provides mechanisms to authenticate users using JWT.

use std::collections::HashMap;
use std::sync::Arc;

use futures::{Future, Stream};

use hyper::header::{ContentLength, ContentType};

use jwt::{decode, Algorithm, Validation};

use reqwest::header::USER_AGENT;

use url::form_urlencoded;

use frontend::rest::services::Future as InternalFuture;
use frontend::rest::services::{default_future, Request, Response, WebService};

use http::{build_async_client, build_client};

use config::JWTValidation;

use logging::LoggingErrors;

#[derive(Debug, Serialize, Deserialize)]
struct Auth {
    username: String,
    token: String,
    jwt_token: Option<JWTClaims>,
}

/// claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTClaims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(rename = "releaseChannels", default)]
    pub channels: Vec<String>,
    #[serde(rename = "isPatreonAccountLinked")]
    pub is_linked: bool,
    #[serde(rename = "isPatreonSubscriptionActive")]
    pub is_subscribed: bool,
}

/// Calls the given server to obtain a JWT token and returns a Future<String> with the response
pub fn authenticate_async(
    url: String,
    username: String,
    token: String,
) -> Box<dyn futures::Future<Item = String, Error = String>> {
    // Build the HTTP client up
    let client = match build_async_client() {
        Ok(v) => v,
        Err(_) => {
            return Box::new(futures::future::err(
                "Unable to build async web client".to_string(),
            ));
        }
    };

    Box::new(client.post(&url)
        .header(USER_AGENT, "liftinstall (j-selby)")
        .header("X-USERNAME", username.clone())
        .header("X-TOKEN", token.clone())
        .send()
        .map_err(|err| {
            format!("stream error {:?}, client: {:?}, http: {:?}, redirect: {:?}, serialization: {:?}, timeout: {:?}, server: {:?}",
                    err, err.is_client_error(), err.is_http(), err.is_redirect(),
                    err.is_serialization(), err.is_timeout(), err.is_server_error())
        })
        .map(|mut response| {
            match response.status() {
                reqwest::StatusCode::OK =>
                    Ok(response.text()
                        .map_err(|e| {
                            format!("Error while converting the response to text {:?}", e)
                        })),
                _ => {
                    Err(format!("Error wrong response code from server {:?}", response.status()))
                }
            }
        })
        .and_then(|x| x)
        .flatten()
    )
}

pub fn authenticate_sync(url: String, username: String, token: String) -> Result<String, String> {
    // Build the HTTP client up
    let client = build_client()?;

    let mut response = client.post(&url)
        .header(USER_AGENT, "liftinstall (j-selby)")
        .header("X-USERNAME", username.clone())
        .header("X-TOKEN", token.clone())
        .send()
        .map_err(|err| {
            format!("stream error {:?}, client: {:?}, http: {:?}, redirect: {:?}, serialization: {:?}, timeout: {:?}, server: {:?}",
                    err, err.is_client_error(), err.is_http(), err.is_redirect(),
                    err.is_serialization(), err.is_timeout(), err.is_server_error())
        })?;

    match response.status() {
        reqwest::StatusCode::OK => Ok(response
            .text()
            .map_err(|e| format!("Error while converting the response to text {:?}", e))?),
        _ => Err(format!(
            "Error wrong response code from server {:?}",
            response.status()
        )),
    }
}

pub fn validate_token(
    body: String,
    pub_key_base64: String,
    validation: Option<JWTValidation>,
) -> Result<JWTClaims, String> {
    // Get the public key for this authentication url
    let pub_key = if pub_key_base64.is_empty() {
        vec![]
    } else {
        match base64::decode(&pub_key_base64) {
            Ok(v) => v,
            Err(err) => {
                return Err(format!(
                    "Configured public key was not empty and did not decode as base64 {:?}",
                    err
                ));
            }
        }
    };

    // Configure validation for audience and issuer if the configuration provides it
    let mut validation = match validation {
        Some(v) => {
            let mut valid = Validation::new(Algorithm::RS256);
            valid.iss = v.iss;
            if let &Some(ref v) = &v.aud {
                valid.set_audience(v);
            }
            valid
        }
        None => Validation::default(),
    };
    validation.validate_exp = false;
    validation.validate_nbf = false;
    // Verify the JWT token
    decode::<JWTClaims>(&body, pub_key.as_slice(), &validation)
        .map(|tok| tok.claims)
        .map_err(|err| {
            format!(
                "Error while decoding the JWT. error: {:?} jwt: {:?}",
                err, body
            )
        })
}

pub fn handle(service: &WebService, _req: Request) -> InternalFuture {
    let framework = service
        .framework
        .read()
        .log_expect("InstallerFramework has been dirtied");
    let credentials = framework.database.credentials.clone();
    let config = framework
        .config
        .clone()
        .log_expect("No in-memory configuration found");

    // If authentication isn't configured, just return immediately
    if config.authentication.is_none() {
        return default_future(Response::new().with_status(hyper::Ok).with_body("{}"));
    }

    // Create moveable framework references so that the lambdas can write to them later
    let write_cred_fw = Arc::clone(&service.framework);

    Box::new(
        _req.body()
            .concat2()
            .map(move |body| {
                let req = form_urlencoded::parse(body.as_ref())
                    .into_owned()
                    .collect::<HashMap<String, String>>();

                // Determine which credentials we should use
                let (username, token) = {
                    let req_username = req.get("username").log_expect("No username in request");
                    let req_token = req.get("token").log_expect("No token in request");

                    // if the user didn't provide credentials, and theres nothing stored in the
                    // database, return an early error
                    let req_cred_valid = !req_username.is_empty() && !req_token.is_empty();
                    let stored_cred_valid =
                        !credentials.username.is_empty() && !credentials.token.is_empty();

                    if !req_cred_valid && !stored_cred_valid {
                        info!("No passed in credential and no stored credentials to validate");
                        return default_future(Response::new().with_status(hyper::BadRequest));
                    }

                    if req_cred_valid {
                        (req_username.clone(), req_token.clone())
                    } else {
                        (credentials.username.clone(), credentials.token.clone())
                    }
                };

                // second copy of the credentials so we can move them into a different closure
                let (username_clone, token_clone) = (username.clone(), token.clone());

                let authentication = config
                    .authentication
                    .log_expect("No authentication configuration");
                let auth_url = authentication.auth_url.clone();
                let pub_key_base64 = authentication.pub_key_base64.clone();
                let validation = authentication.validation.clone();

                // call the authentication URL to see if we are authenticated
                Box::new(
                    authenticate_async(auth_url, username.clone(), token.clone())
                        .map(|body| validate_token(body, pub_key_base64, validation))
                        .and_then(|res| res)
                        .map(move |claims| {
                            let out = Auth {
                                username: username_clone,
                                token: token_clone,
                                jwt_token: Some(claims.clone()),
                            };
                            // Convert the json to a string and return the json token
                            match serde_json::to_string(&out) {
                                Ok(v) => Ok(v),
                                Err(e) => Err(format!(
                                    "Error while converting the claims to JSON string: {:?}",
                                    e
                                )),
                            }
                        })
                        .and_then(|res| res)
                        .map(move |json| {
                            {
                                // Store the validated username and password into the installer database
                                let mut framework = write_cred_fw
                                    .write()
                                    .log_expect("InstallerFramework has been dirtied");
                                framework.database.credentials.username = username;
                                framework.database.credentials.token = token;
                            }

                            // Finally return the JSON with the response
                            info!("successfully verified username and token");
                            Response::new()
                                .with_header(ContentLength(json.len() as u64))
                                .with_header(ContentType::json())
                                .with_status(hyper::StatusCode::Ok)
                                .with_body(json)
                        })
                        .map_err(|err| {
                            error!(
                                "Got an internal error while processing user token: {:?}",
                                err
                            );
                            Response::new().with_status(hyper::StatusCode::InternalServerError)
                        })
                        .or_else(|err| {
                            // Convert the Err value into an Ok value since the error code from
                            // this HTTP request is an Ok(response)
                            Ok(err)
                        }),
                )
            })
            // Flatten the internal future into the output response future
            .flatten(),
    )
}

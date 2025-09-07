use oauth2::basic::BasicClient;
use oauth2::{
    AccessToken, AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

#[derive(Debug, Deserialize, Serialize)]
pub struct UserInfo {
    #[serde(rename(deserialize="displayName"))]
    pub display_name: Option<String>,
    pub mail: Option<String>,
    #[serde(rename(deserialize="userPrincipalName"))]
    pub user_principal_name: Option<String>,
    pub id: String,
}

type ConfiguredClient = BasicClient<
    oauth2::EndpointSet,    // HasAuthUrl
    oauth2::EndpointNotSet, // HasDeviceAuthUrl
    oauth2::EndpointNotSet, // HasIntrospectionUrl
    oauth2::EndpointNotSet, // HasRevocationUrl
    oauth2::EndpointSet,    // HasTokenUrl
>;

pub struct AzureAuth {
    client: ConfiguredClient,
}

impl AzureAuth {
    pub fn new(client_id: &str, tenant_id: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = BasicClient::new(ClientId::new(client_id.to_string()))
            .set_auth_uri(AuthUrl::new(format!(
                "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
                tenant_id
            ))?)
            .set_token_uri(TokenUrl::new(format!(
                "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
                tenant_id
            ))?)
            .set_redirect_uri(RedirectUrl::new(
                "http://localhost:8080/callback".to_string(),
            )?);

        Ok(AzureAuth { client })
    }

    pub async fn authenticate(&self) -> Result<AccessToken, Box<dyn std::error::Error>> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (auth_url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            //.add_scope(Scope::new("openid".to_string()))
            //.add_scope(Scope::new("profile".to_string()))
            //.add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new(
                "https://graph.microsoft.com/User.Read".to_string(),
            ))
            .set_pkce_challenge(pkce_challenge)
            .url();

        

        println!("Please open this URL in your browser:");
        println!("{}", auth_url);
        // Open browser automatically (platform-specific)

        if webbrowser::open(auth_url.as_str()).is_err() {
            return Err("Unable to open browser for authentication".into())
        }

        // get auth code? Host a one-shot tcp listener?
        let received_code = await_auth_code(&csrf_token).await?;
        println!("got code!");

        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none()) // Following redirects opens the client up to SSRF vulnerabilities.
            .build()?;

        // Exchange the code for a token
        let token = self
            .client
            .exchange_code(received_code)
            .set_pkce_verifier(pkce_verifier)
            //.add_extra_param("client_id", self.client.client_id().as_str())
            .request_async(&http_client)
            .await?;

        return Ok(token.access_token().clone());
    }

    pub async fn get_user_info(
        &self,
        access_token: &AccessToken,
    ) -> Result<UserInfo, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let response = client
            .get("https://graph.microsoft.com/v1.0/me")
            .bearer_auth(access_token.secret())
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Graph API error: {}", error_text).into());
        }

        let user_info: UserInfo = response.json().await?;
        Ok(user_info)
    }
}

async fn await_auth_code(
    state: &CsrfToken,
) -> Result<AuthorizationCode, Box<dyn std::error::Error>> {
    println!("await_auth_code!");
    // Set up one-shot TCP listener on port 8080
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    println!("Waiting for OAuth callback on http://localhost:8080/callback...");

    // Accept one connection (one-shot)
    let mut stream = listener.incoming().next().unwrap()?;

    // Read the HTTP request
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;

    // Parse the request line to extract query parameters
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        let error_response = "HTTP/1.1 400 Bad Request\r\n\r\n<html><body><h1>Bad Request</h1><p>Invalid HTTP request format.</p></body></html>";
        stream.write_all(error_response.as_bytes())?;
        return Err("Invalid HTTP request".into());
    }

    let path_and_query = parts[1];
    let query_params = if let Some(query_start) = path_and_query.find('?') {
        let query = &path_and_query[query_start + 1..];
        parse_query_params(query)
    } else {
        HashMap::new()
    };

    // Check for OAuth error first
    if let Some(error) = query_params.get("error") {
        let error_description = query_params
            .get("error_description")
            .map(|desc| url_decode(desc).unwrap_or_else(|_| desc.clone()))
            .unwrap_or_else(|| "No description provided".to_string());

        let error_response = format!(
            "HTTP/1.1 400 Bad Request\r\n\r\n<html><body><h1>OAuth Error</h1><p>Error: {}</p><p>Description: {}</p></body></html>",
            error, error_description
        );
        stream.write_all(error_response.as_bytes())?;
        return Err(format!("OAuth error: {} - {}", error, error_description).into());
    }

    // Validate CSRF token (state parameter)
    if let Some(received_state) = query_params.get("state") {
        let decoded_state = url_decode(received_state)?;
        if &decoded_state != state.secret() {
            let error_response = "HTTP/1.1 400 Bad Request\r\n\r\n<html><body><h1>Security Error</h1><p>CSRF token mismatch. This may indicate a security issue.</p></body></html>";
            stream.write_all(error_response.as_bytes())?;
            return Err("CSRF token mismatch - possible security issue".into());
        }
    } else {
        let error_response = "HTTP/1.1 400 Bad Request\r\n\r\n<html><body><h1>Security Error</h1><p>Missing state parameter. This may indicate a security issue.</p></body></html>";
        stream.write_all(error_response.as_bytes())?;
        return Err("Missing state parameter".into());
    }

    // Get the authorization code
    let code_str = query_params
        .get("code")
        .ok_or("Missing authorization code")?;
    let decoded_code = url_decode(code_str)?;
    let received_code = AuthorizationCode::new(decoded_code);

    // Send success response back to browser
    let success_response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body style='font-family: Arial, sans-serif; text-align: center; margin-top: 50px;'><h1 style='color: green;'>Authentication Successful!</h1><p>You can close this window and return to the application.</p></body></html>";
    stream.write_all(success_response.as_bytes())?;
    stream.flush()?;

    // Close the listener by dropping it
    drop(listener);
    Ok(received_code)
}

// Helper function to parse query parameters from URL
fn parse_query_params(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    for param in query.split('&') {
        if let Some(eq_pos) = param.find('=') {
            let key = param[..eq_pos].to_string();
            let value = param[eq_pos + 1..].to_string();
            params.insert(key, value);
        }
    }
    params
}

// Helper function for basic URL decoding
fn url_decode(encoded: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut result = String::new();
    let mut chars = encoded.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '%' => {
                // Get the next two characters for hex decoding
                let hex1 = chars
                    .next()
                    .ok_or("Invalid URL encoding: incomplete hex sequence")?;
                let hex2 = chars
                    .next()
                    .ok_or("Invalid URL encoding: incomplete hex sequence")?;
                let hex_str = format!("{}{}", hex1, hex2);
                let byte = u8::from_str_radix(&hex_str, 16)
                    .map_err(|_| "Invalid URL encoding: invalid hex sequence")?;
                result.push(byte as char);
            }
            '+' => result.push(' '),
            _ => result.push(ch),
        }
    }

    Ok(result)
}

use std::{
    fs::File,
    io::{ErrorKind, Read, Seek},
    path::Path,
    time::Duration,
};

use reqwest::{
    Method, StatusCode, Url,
    blocking::{
        Client, RequestBuilder,
        multipart::{Form, Part},
    },
    redirect::Policy,
    retry,
};
use serde_json::{Value, json};

use super::input_file;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub struct Api {
    base: Url,
    client: Client,
}

impl Api {
    pub fn new(base_url: &str) -> Result<Self, String> {
        let base = Url::parse(base_url).map_err(|_| "invalid API base URL")?;
        if !matches!(base.scheme(), "http" | "https") {
            return Err("invalid API base URL".into());
        }
        let client = Client::builder()
            .redirect(Policy::none())
            .retry(retry::never())
            .no_proxy()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|_| "could not initialize the HTTP client")?;
        Ok(Self { base, client })
    }

    pub fn request(&self, method: Method, segments: &[&str]) -> RequestBuilder {
        let mut url = self.base.clone();
        url.path_segments_mut()
            .expect("the configured API base URL supports path segments")
            .pop_if_empty()
            .extend(segments);
        self.client.request(method, url)
    }

    pub fn send_authenticated(
        &self,
        request: RequestBuilder,
        key: &str,
    ) -> Result<Option<Value>, String> {
        let auth = exchange_key(self.request(Method::POST, &["agent", "refresh"]), key)?;
        let token = auth
            .as_ref()
            .and_then(access_token)
            .ok_or("Prove2me authentication did not return an access token")?;
        send_json(request.bearer_auth(token), "request")
    }
}

pub fn exchange_key(request: RequestBuilder, key: &str) -> Result<Option<Value>, String> {
    send_json(request.json(&json!({ "api_key": key })), "authentication")
}

pub fn access_token(value: &Value) -> Option<&str> {
    value
        .get("access_token")
        .and_then(Value::as_str)
        .filter(|token| !token.trim().is_empty())
}

pub fn proof_form(
    theorem_id: &str,
    path: &Path,
    proof_type: &str,
    explanation: Option<&str>,
) -> Result<Form, &'static str> {
    let (file, length, name) = open_upload(path).map_err(|error| match error.kind() {
        ErrorKind::InvalidInput => "the Lean file must be a regular file",
        _ => "could not open the Lean file",
    })?;
    let mut form = Form::new()
        .text("theorem_id", theorem_id.to_owned())
        .text("proof_type", proof_type.to_owned())
        .part(
            "file",
            Part::reader_with_length(file, length).file_name(name),
        );
    if let Some(explanation) = explanation {
        form = form.text("explanation", explanation.to_owned());
    }
    Ok(form)
}

pub fn avatar_form(path: &Path) -> Result<Form, &'static str> {
    let (mut file, length, name) = open_upload(path).map_err(|error| match error.kind() {
        ErrorKind::InvalidInput => "the avatar must be a regular file",
        _ => "could not open the avatar file",
    })?;
    let mut prefix = [0; 12];
    let prefix = &mut prefix[..length.min(12) as usize];
    file.read_exact(prefix)
        .map_err(|_| "could not read the avatar file")?;
    let mime = if prefix.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if prefix.starts_with(b"\xff\xd8\xff") {
        "image/jpeg"
    } else if prefix.starts_with(b"RIFF") && prefix.get(8..12) == Some(b"WEBP") {
        "image/webp"
    } else {
        return Err("the avatar must be a PNG, JPEG, or WebP image");
    };
    file.rewind()
        .map_err(|_| "could not read the avatar file")?;
    let part = Part::reader_with_length(file, length)
        .file_name(name)
        .mime_str(mime)
        .expect("a static image MIME type is valid");
    Ok(Form::new().part("file", part))
}

fn open_upload(path: &Path) -> std::io::Result<(File, u64, String)> {
    let (file, metadata) = input_file::open(path)?;
    let name = path
        .file_name()
        .ok_or(ErrorKind::InvalidInput)?
        .to_string_lossy()
        .into_owned();
    Ok((file, metadata.len(), name))
}

pub fn send_json(request: RequestBuilder, operation: &str) -> Result<Option<Value>, String> {
    let response = request.send().map_err(|error| {
        if error.is_timeout() {
            format!("Prove2me {operation} timed out")
        } else {
            format!("Prove2me {operation} failed")
        }
    })?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "Prove2me {operation} failed (HTTP {})",
            status.as_u16()
        ));
    }
    if status == StatusCode::NO_CONTENT {
        return Ok(None);
    }
    response
        .json()
        .map(Some)
        .map_err(|_| format!("could not read the Prove2me {operation} response as JSON"))
}

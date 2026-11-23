use std::{fs::File, path::Path, time::Duration};

use reqwest::{
    Url,
    blocking::{
        Client, RequestBuilder,
        multipart::{Form, Part},
    },
    redirect::Policy,
    retry,
};
use serde_json::{Value, json};

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

    fn endpoint(&self, segments: &[&str]) -> Url {
        let mut url = self.base.clone();
        url.path_segments_mut()
            .expect("the configured API base URL supports path segments")
            .pop_if_empty()
            .extend(segments);
        url
    }

    pub fn search(&self, query: &str) -> RequestBuilder {
        self.client
            .get(self.endpoint(&["theorems"]))
            .query(&[("q", query)])
    }

    pub fn theorem(&self, id: &str) -> Result<RequestBuilder, &'static str> {
        if !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err("invalid theorem ID");
        }
        Ok(self.client.get(self.endpoint(&["theorems", id])))
    }

    pub fn submit(
        &self,
        theorem_id: String,
        path: &Path,
        proof_type: &'static str,
        explanation: Option<String>,
    ) -> Result<RequestBuilder, &'static str> {
        let name = path
            .file_name()
            .ok_or("missing Lean file name")?
            .to_string_lossy()
            .into_owned();
        if !path
            .metadata()
            .map_err(|_| "could not open the Lean file")?
            .is_file()
        {
            return Err("the Lean file must be a regular file");
        }
        let file = File::open(path).map_err(|_| "could not open the Lean file")?;
        let size = file
            .metadata()
            .map_err(|_| "could not read the Lean file metadata")?
            .len();
        let mut form = Form::new()
            .text("theorem_id", theorem_id)
            .text("proof_type", proof_type)
            .part("file", Part::reader_with_length(file, size).file_name(name));
        if let Some(explanation) = explanation {
            form = form.text("explanation", explanation);
        }
        Ok(self.client.post(self.endpoint(&["verify"])).multipart(form))
    }

    pub fn status(&self, id: &str) -> RequestBuilder {
        self.client
            .get(self.endpoint(&["verify"]))
            .query(&[("submission_id", id)])
    }

    pub fn send(&self, request: RequestBuilder, key: &str) -> Result<Value, String> {
        let auth = send_json(
            self.client
                .post(self.endpoint(&["agent", "refresh"]))
                .json(&json!({ "api_key": key })),
            "authentication",
        )?;
        let token = auth
            .get("access_token")
            .and_then(Value::as_str)
            .filter(|token| !token.is_empty())
            .ok_or("Prove2me authentication did not return an access token")?;
        send_json(request.bearer_auth(token), "request")
    }
}

fn send_json(request: RequestBuilder, operation: &str) -> Result<Value, String> {
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
    response
        .json()
        .map_err(|_| format!("could not read the Prove2me {operation} response as JSON"))
}

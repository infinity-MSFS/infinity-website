use infinity_ui::prelude::*;

pub fn doc() -> Doc {
    Doc::new("network", "HTTP Network", "Module · network", "Modules")
        .scope("infinity-rs")
        .tagline(
            "Make HTTP(S) requests through MSFS's networking subsystem. Bypasses the WASI sandbox and reuses the sim's certificate trust store.",
        )
        .tags(["HTTP", "GET", "POST", "Headers"])
        .section(shape())
        .section(get())
        .section(post())
}

fn shape() -> DocSection {
    DocSection::new("shape", "Shape", || {
        view! {
            <Lead>
                <InlineCode>"http_request"</InlineCode>
                " takes a method, URL, params struct, and a one-shot callback that fires with the parsed "
                <InlineCode>"HttpResponse"</InlineCode>
                ". Header strings use the wire format ("
                <InlineCode>"\"Header: value\""</InlineCode>
                ")."
            </Lead>

            <CodeBlock
                language="rust"
                title="signatures"
                code=r#"pub enum Method { Get, Post, Put }

pub struct HttpParams {
    pub headers: Vec<String>,    // "Name: value" lines
    pub post_field: Option<String>,
    pub body: Vec<u8>,
}

pub struct HttpResponse {
    pub error_code: i32,         // 0 = success
    pub status:     i32,         // HTTP status (e.g. 200)
    pub headers:    Vec<u8>,
    pub data:       Vec<u8>,
}

pub fn http_request(
    method:  Method,
    url:     &str,
    params:  HttpParams,
    on_done: impl FnOnce(HttpResponse) + 'static,
) -> NetResult<FsNetworkRequestId>;"#
            />
        }
    })
    .numbered("1.0")
    .summary("One function — `http_request` — plus the request/response types.")
}

fn get() -> DocSection {
    DocSection::new("get", "GET", || {
        view! {
            <CodeBlock
                language="rust"
                title="GET example"
                code=r#"use infinity_rs::prelude::*;

let params = HttpParams {
    headers: vec![
        "Accept: application/json".to_string(),
        "User-Agent: my-gauge/0.1".to_string(),
    ],
    post_field: None,
    body: vec![],
};

let _ = http_request(Method::Get, "https://example.com/data.json", params, move |resp| {
    if resp.error_code != 0 || resp.status >= 400 {
        eprintln!("http fail: code={} status={}", resp.error_code, resp.status);
        return;
    }
    let _ = commbus_call("demo/data", &resp.data, BroadcastFlags::JS);
});"#
            />
        }
    })
    .numbered("2.0")
    .summary("Fetch JSON, parse, fan out via comm bus.")
}

fn post() -> DocSection {
    DocSection::new("post", "POST", || {
        view! {
            <CodeBlock
                language="rust"
                title="POST"
                code=r#"let params = HttpParams {
    headers: vec!["Content-Type: application/json".to_string()],
    post_field: None,
    body: serde_json::to_vec(&payload).unwrap(),
};

let _ = http_request(Method::Post, "https://example.com/telemetry", params, move |resp| {
    let _ = resp.status; // 2xx = ok
})?;"#
            />

            <Callout tone=Tone::Accent icon=icons::INFO title="Lifetime">
                "The "
                <InlineCode>"FsNetworkRequestId"</InlineCode>
                " handle is informational; the host owns the request lifecycle. The closure is invoked exactly once when the request settles."
            </Callout>
        }
    })
    .numbered("2.1")
    .summary("Pass `body: Vec<u8>` and use `Method::Post`. `post_field` is for `application/x-www-form-urlencoded` style payloads.")
}

use base64::engine::general_purpose;
use base64::Engine;
use futures::FutureExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Response, Server};
use serde_json::{Result as SerdeResult, Value};
use std::result::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::Mutex;
use tokio::time::interval;
use unique_id::{string::StringGenerator, Generator};

use crate::entities::{Port, Session, Sessions};

pub async fn http_server(port: Port, sessions: Sessions) {
    // The address we'll bind to.
    let addr = ([0, 0, 0, 0], port.num).into();

    // This is our service handler. It receives a Request, processes it, and returns a Response.
    let make_service = make_service_fn(|_conn| {
        let sessions_ref = Arc::clone(&sessions);
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let sessions_clone = sessions_ref.clone();
                http_connection_handler(req, sessions_clone)
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_service);
    println!("‣ [HTTP] Waiting connections on {}", port.num);

    if let Err(e) = server.await {
        eprintln!("‣ [HTTP] server error: {}", e);
    }
}

async fn http_connection_handler(
    _req: hyper::Request<Body>,
    sessions: Sessions,
) -> Result<Response<Body>, hyper::Error> {
    let (session_endpoint, agent_request_path) = get_request_url(&_req);
    let uri = _req.uri().clone();
    let request_path = uri.path();
    println!("‣ [HTTP] {:?} {request_path}", _req.method());

    // avoid websocket
    if _req.headers().contains_key("upgrade") {
        println!("‣ [HTTP](ws) 403 Status on {}", request_path);
        let response = Response::builder()
            .status(403)
            .header("Content-type", "text/html")
            .body(Body::from("<h1>403 Not found</h1>"))
            .unwrap();
        return Ok(response);
    }

    if request_path.to_string().clone() == "/".to_string() {
        let response = Response::builder()
            .status(200)
            .header("Content-type", "text/html")
            .body(Body::from("<h1>Home</h1>"))
            .unwrap();
        return Ok(response);
    }

    if !sessions
        .lock()
        .await
        .contains_key(session_endpoint.as_str())
    {
        let response = Response::builder()
            .status(404)
            .header("Content-type", "text/html")
            .body(Body::from("<h1>404 Not found</h1>"))
            .unwrap();
        return Ok(response);
    }

    // Create raw http from request
    // ----------------------------
    let request_id = StringGenerator::default().next_id();
    // headers
    let http_request_info = format!(
        "{} {} {:?}\n",
        _req.method().as_str(),
        agent_request_path,
        _req.version()
    );
    let mut request = request_id.clone() + "\n" + http_request_info.as_str();

    for (key, value) in _req.headers() {
        match value.to_str() {
            Ok(value) => request += &format!("{}: {}\n", key.as_str(), value),
            Err(_) => request += &format!("{}: {:?}\n", key.as_str(), value.as_bytes()),
        }
    }

    // body
    let body = hyper::body::to_bytes(_req.into_body()).await;
    if body.is_ok() {
        let body = body.unwrap();
        if body.len() > 0 {
            request += &format!("\n{}", String::from_utf8(body.to_vec()).unwrap());
        }
    }

    let session = {
        let sessions = sessions.lock().await;
        let session = sessions.get(session_endpoint.as_str());
        match session {
            Some(session) => session.clone(),
            None => {
                let response = Response::builder()
                    .status(500)
                    .header("Content-type", "text/html")
                    .body(Body::from("<h1>500 Internal server error</h1>"))
                    .unwrap();
                return Ok(response);
            }
        }
    };

    let sent = send_raw_http(&session, request).await;
    if let Err(_) = sent {
        println!("‣ [HTTP] 500 Status on {}", session_endpoint);
        let response = Response::builder()
            .status(500)
            .header("Content-type", "text/html")
            .body(Body::from("<h1>500 Internal server error</h1>"))
            .unwrap();
        return Ok(response);
    };

    // Wait for response
    let max_time = 100_000; // 100 seconds
    let mut time = 0;
    let mut http_raw_response = String::from("");
    let mut timeout_interval = interval(Duration::from_millis(100));
    loop {
        if let Some(response) = match_response(&session, request_id.clone()).await {
            http_raw_response = response;
            break;
        }

        // Check if timeout
        if time >= max_time {
            break;
        }

        timeout_interval.tick().await;
        time += 100;
    }

    if time >= max_time {
        let response = Response::builder()
            .status(524)
            .header("Content-type", "text/html")
            .body(Body::from("<h1>524 A timeout error ocurred</h1>"))
            .unwrap();
        return Ok(response);
    }

    // Check integrity of response [Packet fragmentation error]
    if http_raw_response == "EPACKFRAG".to_string() {
        println!("‣ [HTTP] 500 Status on {}", session_endpoint);
        let response = Response::builder()
            .status(500)
            .header("Content-type", "text/html")
            .body(Body::from("<h1>500 Internal server error</h1>"))
            .unwrap();
        return Ok(response);
    }

    let http_response_result: SerdeResult<Value> = serde_json::from_str(http_raw_response.as_str());
    let http_response = http_response_result.unwrap();

    // Build response
    let status_code = match http_response["statusCode"].as_i64() {
        Some(status_code) => status_code as u16,
        None => 0,
    };
    let default_headers = serde_json::Map::new();
    let headers = match http_response["headers"].as_object() {
        Some(headers) => headers,
        None => &default_headers,
    };
    let body = http_response["body"].as_str();

    if status_code == 0 {
        println!("‣ [HTTP] 570 Status on {}", session_endpoint);
        let response = Response::builder()
            .status(570)
            .header("Content-type", "text/html")
            .body(Body::from("<h1>570 Agent bad response</h1>"))
            .unwrap();
        return Ok(response);
    }

    if headers.keys().len() < 1 {
        println!("‣ [HTTP] 570 Status on {}", session_endpoint);
        let response = Response::builder()
            .status(570)
            .header("Content-type", "text/html")
            .body(Body::from("<h1>570 Agent bad response</h1>"))
            .unwrap();
        return Ok(response);
    }

    let mut response_builder = Response::builder().status(status_code);
    for (key, value) in headers {
        if (key != "Content-Length") && (key != "content-length") {
            response_builder = response_builder.header(
                key,
                hyper::header::HeaderValue::from_str(value.as_str().unwrap()).unwrap(),
            );
        }
    }

    let response: Response<Body>;
    if body.is_some() {
        let _body = body.unwrap().to_string();
        let _body = general_purpose::STANDARD.decode(_body);
        let _body = match _body {
            Ok(_body) => match String::from_utf8(_body) {
                Ok(_body) => _body,
                Err(_) => {
                    println!("Error converting body");
                    let response = Response::builder()
                        .status(500)
                        .header("Content-type", "text/html")
                        .body(Body::from("<h1>500 Internal server error</h1>"))
                        .unwrap();
                    return Ok(response);
                }
            },
            Err(_) => {
                println!("Error decoding body");
                let response = Response::builder()
                    .status(500)
                    .header("Content-type", "text/html")
                    .body(Body::from("<h1>500 Internal server error</h1>"))
                    .unwrap();
                return Ok(response);
            }
        };
        let hyper_body = Body::from(_body);
        response = response_builder.body(hyper_body).unwrap();
    } else {
        response = response_builder.body(Body::empty()).unwrap();
    }

    println!("‣ [HTTP] 200 Status on {}", request_path);
    return Ok(response);
}

fn get_request_url(_req: &hyper::Request<Body>) -> (String, String) {
    let referer = _req.headers().get("referer");

    // [no referer]
    if !referer.is_some() {
        let mut segments = _req.uri().path().split("/").collect::<Vec<&str>>(); // /abc/paco -> ["", "abc", "paco"]
        let session_endpoint = segments[1].to_string();
        segments.drain(0..2); // request path
        return (session_endpoint, "/".to_string() + &segments.join("/"));
    }

    let referer = referer.unwrap().to_str().unwrap(); // https://localhost:8080/abc/paco
    let mut referer = referer
        .split("/")
        .map(|r| r.to_string())
        .collect::<Vec<String>>();
    referer.drain(0..3); // drop -> "https:" + "" + "localhost:8080"

    let mut url = _req
        .uri()
        .path()
        .split("/")
        .map(|r| r.to_string())
        .filter(|r| r != "")
        .collect::<Vec<String>>();

    // [different session-endpoint]
    if referer[0] != url[0] {
        return (referer[0].clone(), "/".to_string() + &url.join("/"));
    }

    // [same session-endpoint]
    url.drain(0..1);
    return (referer[0].clone(), "/".to_string() + &url.join("/"));
}

async fn send_raw_http(
    session: &Arc<Mutex<Session>>,
    http_raw_request: String,
) -> Result<(), SendError<String>> {
    let session = session.lock().await;
    let request_delimiter = "<<<EOF>>>";
    let sent = session
        .socket_tx
        .send(format!("{http_raw_request}{request_delimiter}"))
        .await; // Send raw http to tcp socket
    return match sent {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    };
}

async fn match_response(session: &Arc<Mutex<Session>>, request_id: String) -> Option<String> {
    let mut target_session = session.lock().await;

    // Check incomming response
    if let Some(agent_response) = target_session.responses_rx.recv().now_or_never() {
        let agent_response = agent_response.unwrap();

        // get response or save as unmatched
        if request_id == agent_response.0 {
            println!("----- response");
            return Some(agent_response.1);
        } else {
            target_session
                .unmatched_responses
                .insert(agent_response.0, agent_response.1);

            // get matched response or save as unmatched
            if let Some(response) = target_session.unmatched_responses.get(&request_id) {
                println!("----- response (unmatched)");
                return Some(response.clone());
            } else {
                println!("no response");
                return None;
            }
        }
    }
    return None;
}

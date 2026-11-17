//! HTTP-level tests for the baseline fetcher: the URL actually requested,
//! non-success responses, empty/garbage bodies, and the app refusing to
//! persist anything when the download fails.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use host_blocker_lib::app::App;
use host_blocker_lib::baseline::fetcher::Fetcher;
use host_blocker_lib::baseline::stevenblack::Extension;
use host_blocker_lib::error::AppError;
use host_blocker_lib::hosts_file::writer::{PrivilegedWriter, WriteError};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

struct NoopWriter;
impl PrivilegedWriter for NoopWriter {
    fn install_hosts(&self, _s: &Path, _h: &Path, _b: &Path) -> Result<(), WriteError> {
        Ok(())
    }
}

/// Run `f` with a Fetcher on a plain blocking thread: reqwest's blocking
/// client must not be created or dropped on an async-runtime thread.
async fn off_fetch<T>(base: String, f: impl FnOnce(Fetcher) -> T + Send + 'static) -> T
where
    T: Send + 'static,
{
    tokio::task::spawn_blocking(move || f(Fetcher::new(base)))
        .await
        .unwrap()
}

async fn stub(server: &MockServer, template: ResponseTemplate) {
    Mock::given(method("GET"))
        .respond_with(template)
        .mount(server)
        .await;
}

const BODY: &str = "0.0.0.0 ads.example.net\n0.0.0.0 track.example.net\n";

#[tokio::test]
async fn requests_the_variant_url_and_parses_the_body() {
    let server = MockServer::start().await;
    stub(&server, ResponseTemplate::new(200).set_body_string(BODY)).await;
    let result = off_fetch(server.uri(), move |f| {
        let url = f.url_for(&[Extension::Gambling, Extension::FakeNews]);
        (url, f.fetch(&[Extension::Gambling, Extension::FakeNews]))
    })
    .await;
    assert_eq!(
        result.0,
        format!("{}/alternates/fakenews-gambling/hosts", server.uri())
    );
    assert_eq!(
        result.1.unwrap(),
        ["0.0.0.0 ads.example.net", "0.0.0.0 track.example.net"]
    );

    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0].url.path(),
        "/alternates/fakenews-gambling/hosts"
    );
}

#[tokio::test]
async fn requests_the_unified_url_without_extensions() {
    let server = MockServer::start().await;
    stub(&server, ResponseTemplate::new(200).set_body_string(BODY)).await;
    off_fetch(server.uri(), move |f| f.fetch(&[]))
        .await
        .unwrap();
    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests[0].url.path(), "/hosts");
}

#[tokio::test]
async fn non_success_status_is_a_network_error() {
    let server = MockServer::start().await;
    stub(&server, ResponseTemplate::new(404)).await;
    let err = off_fetch(server.uri(), move |f| f.fetch(&[]))
        .await
        .unwrap_err();
    match err {
        AppError::Network(msg) => assert!(msg.contains("404"), "{msg}"),
        other => panic!("expected Network, got {other:?}"),
    }
}

#[tokio::test]
async fn body_without_entries_is_rejected() {
    let server = MockServer::start().await;
    stub(
        &server,
        ResponseTemplate::new(200).set_body_string("# just a comment\n\n"),
    )
    .await;
    let err = off_fetch(server.uri(), move |f| f.fetch(&[]))
        .await
        .unwrap_err();
    match err {
        AppError::Network(msg) => assert!(msg.contains("no usable entries"), "{msg}"),
        other => panic!("expected Network, got {other:?}"),
    }
}

/// The stub answers 200 with a body that parses to zero entries: the import
/// must fail and must not persist state, write the hosts file, or touch the
/// baseline cache.
#[tokio::test]
async fn import_with_unusable_body_changes_nothing() {
    let server = MockServer::start().await;
    stub(
        &server,
        ResponseTemplate::new(200).set_body_string("# nothing usable\n"),
    )
    .await;
    let dir = tempfile::tempdir().unwrap();
    let hosts = dir.path().join("hosts");
    let initial = "127.0.0.1 localhost\n";
    fs::write(&hosts, initial).unwrap();
    let base = server.uri();
    let state_dir = dir.path().join("state");
    let cache_dir = dir.path().join("cache");
    let outcome = {
        let hosts = hosts.clone();
        tokio::task::spawn_blocking(move || {
            let w: Arc<dyn PrivilegedWriter> = Arc::new(NoopWriter);
            let app = App::with_parts(state_dir, cache_dir, hosts, w, base);
            app.import_baseline(&[])
        })
        .await
        .unwrap()
    };
    match outcome {
        Err(AppError::Network(_)) => {}
        other => panic!("expected Network error, got {other:?}"),
    }
    assert_eq!(fs::read_to_string(&hosts).unwrap(), initial);
    let store = host_blocker_lib::store::Store::new(dir.path().join("state"));
    let persisted = store.load().unwrap();
    assert!(persisted.baseline.is_none());
    assert!(persisted.sites.is_empty());
}

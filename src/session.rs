use librespot;
use std::thread;
use tokio_core::reactor::Core;
use cpython::PyResult;
use futures;
use tokio_core::reactor::Remote;
use futures::Future;

use pyfuture::PyFuture;
use player::Player;
use metadata::{Track, Album, Artist};
use webtoken::Token;
use SpotifyId;

py_class!(pub class Session |py| {
    data session : librespot::core::session::Session;
    data pipe_path: Option<String>;
    data handle: Remote;

    @classmethod def connect(_cls, username: String, password: String, pipe_path: String) -> PyResult<PyFuture> {
        use librespot::core::config::SessionConfig;
        use librespot::core::authentication::Credentials;

        let config = SessionConfig::default();
        let credentials = Credentials::with_password(username, password);

        let (session_tx, session_rx) = futures::sync::oneshot::channel();
        let (handle_tx, handle_rx) = futures::sync::oneshot::channel();

        thread::spawn(move || {
            let mut core = Core::new().unwrap();
            let handle = core.handle();

            let _ = handle_tx.send(handle.remote().clone());

            let session = core.run(librespot::core::session::Session::connect(config, credentials, None, handle)).unwrap();

            let _ = session_tx.send(session);

            core.run(futures::future::empty::<(), ()>()).unwrap();
        });

        let handle = handle_rx.wait().unwrap();

        PyFuture::new(py, handle.clone(), session_rx, move |py, result| {
            let session = result.unwrap();
            let mut opt_pipe_path = None;
            if !pipe_path.is_empty() {
                opt_pipe_path = Some(pipe_path);
            }

            Session::create_instance(py, session, opt_pipe_path, handle)
        })
    }

    def player(&self) -> PyResult<Player> {
        let session = self.session(py).clone();
        let handle = self.handle(py).clone();
        let pipe_path = self.pipe_path(py).clone();

        Player::new(py, session, pipe_path, handle)
    }

    def get_track(&self, track: SpotifyId) -> PyResult<PyFuture> {
        let session = self.session(py).clone();
        let handle = self.handle(py).clone();
        let track = *track.id(py);

        Track::get(py, session, handle, track)
    }

    def get_album(&self, album: SpotifyId) -> PyResult<PyFuture> {
        let session = self.session(py).clone();
        let handle = self.handle(py).clone();
        let album = *album.id(py);

        Album::get(py, session, handle, album)
    }

    def get_artist(&self, artist: SpotifyId) -> PyResult<PyFuture> {
        let session = self.session(py).clone();
        let handle = self.handle(py).clone();
        let artist = *artist.id(py);

        Artist::get(py, session, handle, artist)
    }

    def web_token(&self, client_id: &str, scopes: &str) -> PyResult<PyFuture> {
        let session = self.session(py);
        let handle = self.handle(py).clone();
        Token::get(py, session, handle, client_id, scopes)
    }
});

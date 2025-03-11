use bincode::{Decode, Encode};
use std::convert::TryFrom;

pub trait IntoRequest {
    type Response;
    fn into_request(self) -> Request;
    // fn from_request(request: Request) -> Result<Self, ()>
    // where
    //     Self: Sized,
    //     Self::Response: TryFrom<Response, Error = ()> + IntoResponse;
}

pub trait IntoResponse {
    type Request;
    fn into_response(self) -> Response;
    // fn from_response(response: Response) -> Result<Self, ()>
    // where
    //     Self: Sized,
    //     Self::Request: TryFrom<Request, Error = ()> + IntoRequest;
}

#[derive(Encode, Decode, Debug)]
pub struct Ping;

#[derive(Encode, Decode, Debug)]
pub struct Pong(pub bool);

#[derive(Encode, Decode, Debug)]
pub struct SetDaemonState {
    pub enabled: bool,
}

#[derive(Encode, Decode, Debug)]
pub struct DaemonStatus {
    pub running: bool,
}

#[derive(Encode, Decode, Debug)]
pub struct SetFramerate {
    pub fps: u32,
}

#[derive(Encode, Decode, Debug)]
pub struct FramerateStatus {
    pub fps: u32,
}

#[derive(Encode, Decode, Debug)]
pub struct LoadWallpaper {
    pub path: String,
}

#[derive(Encode, Decode, Debug)]
pub struct WallpaperLoaded {
    pub name: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Encode, Decode, Debug)]
pub struct GetCurrentWallpaper;

#[derive(Encode, Decode, Debug)]
pub struct CurrentWallpaper {
    pub name: Option<String>,
    pub path: Option<String>,
}

#[derive(Encode, Decode, Debug)]
pub struct ListWallpapers;

#[derive(Encode, Decode, Debug)]
pub struct WallpaperList {
    pub wallpapers: Vec<WallpaperInfo>,
}

#[derive(Encode, Decode, Debug)]
pub struct WallpaperInfo {
    pub name: String,
    pub path: String,
}

#[derive(Encode, Decode, Debug)]
pub struct InstallWallpaper {
    pub path: String,
    pub name: Option<String>, // Optional custom name, defaults to directory name
}

#[derive(Encode, Decode, Debug)]
pub struct WallpaperInstalled {
    pub name: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Encode, Decode, Debug)]
pub struct SetCurrentWallpaper {
    pub name: String,
}

#[derive(Encode, Decode, Debug)]
pub struct WallpaperSet {
    pub name: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Encode, Decode, Debug)]
pub enum Request {
    Ping(Ping),
    SetDaemonState(SetDaemonState),
    SetFramerate(SetFramerate),
    LoadWallpaper(LoadWallpaper),
    GetCurrentWallpaper(GetCurrentWallpaper),
    ListWallpapers(ListWallpapers),
    InstallWallpaper(InstallWallpaper),
    SetCurrentWallpaper(SetCurrentWallpaper),
}

#[derive(Encode, Decode, Debug)]
pub enum Response {
    Pong(Pong),
    DaemonStatus(DaemonStatus),
    FramerateStatus(FramerateStatus),
    WallpaperLoaded(WallpaperLoaded),
    CurrentWallpaper(CurrentWallpaper),
    WallpaperList(WallpaperList),
    WallpaperInstalled(WallpaperInstalled),
    WallpaperSet(WallpaperSet),
}

impl IntoRequest for Ping {
    type Response = Pong;
    fn into_request(self) -> Request {
        Request::Ping(self)
    }
    // fn from_request(request: Request) -> Result<Self, ()> {
    //     match request {
    //         Request::Ping(ping) => Ok(ping),
    //         _ => Err(()),
    //     }
    // }
}

impl TryFrom<Request> for Ping {
    type Error = ();
    
    fn try_from(request: Request) -> Result<Self, Self::Error> {
        match request {
            Request::Ping(ping) => Ok(ping),
            _ => Err(()),
        }
    }
}

impl IntoResponse for Pong {
    type Request = Ping;
    fn into_response(self) -> Response {
        Response::Pong(self)
    }
    // fn from_response(response: Response) -> Result<Self, ()> {
    //     match response {
    //         Response::Pong(pong) => Ok(pong),
    //         _ => Err(()),
    //     }
    // }
}

impl TryFrom<Response> for Pong {
    type Error = ();
    
    fn try_from(response: Response) -> Result<Self, Self::Error> {
        match response {
            Response::Pong(pong) => Ok(pong),
            _ => Err(()),
        }
    }
}

impl IntoRequest for SetDaemonState {
    type Response = DaemonStatus;
    fn into_request(self) -> Request {
        Request::SetDaemonState(self)
    }
    // fn from_request(request: Request) -> Result<Self, ()> {
    //     match request {
    //         Request::SetDaemonState(set_state) => Ok(set_state),
    //         _ => Err(()),
    //     }
    // }
}

impl TryFrom<Request> for SetDaemonState {
    type Error = ();
    
    fn try_from(request: Request) -> Result<Self, Self::Error> {
        match request {
            Request::SetDaemonState(set_state) => Ok(set_state),
            _ => Err(()),
        }
    }
}

impl IntoResponse for DaemonStatus {
    type Request = SetDaemonState;
    fn into_response(self) -> Response {
        Response::DaemonStatus(self)
    }
    // fn from_response(response: Response) -> Result<Self, ()> {
    //     match response {
    //         Response::DaemonStatus(daemon_status) => Ok(daemon_status),
    //         _ => Err(()),
    //     }
    // }
}

impl TryFrom<Response> for DaemonStatus {
    type Error = ();
    
    fn try_from(response: Response) -> Result<Self, Self::Error> {
        match response {
            Response::DaemonStatus(daemon_status) => Ok(daemon_status),
            _ => Err(()),
        }
    }
}

impl IntoRequest for SetFramerate {
    type Response = FramerateStatus;
    fn into_request(self) -> Request {
        Request::SetFramerate(self)
    }
}

impl TryFrom<Request> for SetFramerate {
    type Error = ();
    
    fn try_from(request: Request) -> Result<Self, Self::Error> {
        match request {
            Request::SetFramerate(set_framerate) => Ok(set_framerate),
            _ => Err(()),
        }
    }
}

impl IntoResponse for FramerateStatus {
    type Request = SetFramerate;
    fn into_response(self) -> Response {
        Response::FramerateStatus(self)
    }
}

impl TryFrom<Response> for FramerateStatus {
    type Error = ();
    
    fn try_from(response: Response) -> Result<Self, Self::Error> {
        match response {
            Response::FramerateStatus(framerate_status) => Ok(framerate_status),
            _ => Err(()),
        }
    }
}

impl IntoRequest for LoadWallpaper {
    type Response = WallpaperLoaded;
    fn into_request(self) -> Request {
        Request::LoadWallpaper(self)
    }
}

impl TryFrom<Request> for LoadWallpaper {
    type Error = ();
    
    fn try_from(request: Request) -> Result<Self, Self::Error> {
        match request {
            Request::LoadWallpaper(load_wallpaper) => Ok(load_wallpaper),
            _ => Err(()),
        }
    }
}

impl IntoResponse for WallpaperLoaded {
    type Request = LoadWallpaper;
    fn into_response(self) -> Response {
        Response::WallpaperLoaded(self)
    }
}

impl TryFrom<Response> for WallpaperLoaded {
    type Error = ();
    
    fn try_from(response: Response) -> Result<Self, Self::Error> {
        match response {
            Response::WallpaperLoaded(wallpaper_loaded) => Ok(wallpaper_loaded),
            _ => Err(()),
        }
    }
}

impl IntoRequest for GetCurrentWallpaper {
    type Response = CurrentWallpaper;
    fn into_request(self) -> Request {
        Request::GetCurrentWallpaper(self)
    }
}

impl TryFrom<Request> for GetCurrentWallpaper {
    type Error = ();
    
    fn try_from(request: Request) -> Result<Self, Self::Error> {
        match request {
            Request::GetCurrentWallpaper(get_current_wallpaper) => Ok(get_current_wallpaper),
            _ => Err(()),
        }
    }
}

impl IntoResponse for CurrentWallpaper {
    type Request = GetCurrentWallpaper;
    fn into_response(self) -> Response {
        Response::CurrentWallpaper(self)
    }
}

impl TryFrom<Response> for CurrentWallpaper {
    type Error = ();
    
    fn try_from(response: Response) -> Result<Self, Self::Error> {
        match response {
            Response::CurrentWallpaper(current_wallpaper) => Ok(current_wallpaper),
            _ => Err(()),
        }
    }
}

impl IntoRequest for ListWallpapers {
    type Response = WallpaperList;
    fn into_request(self) -> Request {
        Request::ListWallpapers(self)
    }
}

impl TryFrom<Request> for ListWallpapers {
    type Error = ();
    
    fn try_from(request: Request) -> Result<Self, Self::Error> {
        match request {
            Request::ListWallpapers(list_wallpapers) => Ok(list_wallpapers),
            _ => Err(()),
        }
    }
}

impl IntoResponse for WallpaperList {
    type Request = ListWallpapers;
    fn into_response(self) -> Response {
        Response::WallpaperList(self)
    }
}

impl TryFrom<Response> for WallpaperList {
    type Error = ();
    
    fn try_from(response: Response) -> Result<Self, Self::Error> {
        match response {
            Response::WallpaperList(wallpaper_list) => Ok(wallpaper_list),
            _ => Err(()),
        }
    }
}

impl IntoRequest for InstallWallpaper {
    type Response = WallpaperInstalled;
    fn into_request(self) -> Request {
        Request::InstallWallpaper(self)
    }
}

impl TryFrom<Request> for InstallWallpaper {
    type Error = ();
    
    fn try_from(request: Request) -> Result<Self, Self::Error> {
        match request {
            Request::InstallWallpaper(install_wallpaper) => Ok(install_wallpaper),
            _ => Err(()),
        }
    }
}

impl IntoResponse for WallpaperInstalled {
    type Request = InstallWallpaper;
    fn into_response(self) -> Response {
        Response::WallpaperInstalled(self)
    }
}

impl TryFrom<Response> for WallpaperInstalled {
    type Error = ();
    
    fn try_from(response: Response) -> Result<Self, Self::Error> {
        match response {
            Response::WallpaperInstalled(wallpaper_installed) => Ok(wallpaper_installed),
            _ => Err(()),
        }
    }
}

impl IntoRequest for SetCurrentWallpaper {
    type Response = WallpaperSet;
    fn into_request(self) -> Request {
        Request::SetCurrentWallpaper(self)
    }
}

impl TryFrom<Request> for SetCurrentWallpaper {
    type Error = ();
    
    fn try_from(request: Request) -> Result<Self, Self::Error> {
        match request {
            Request::SetCurrentWallpaper(set_current_wallpaper) => Ok(set_current_wallpaper),
            _ => Err(()),
        }
    }
}

impl IntoResponse for WallpaperSet {
    type Request = SetCurrentWallpaper;
    fn into_response(self) -> Response {
        Response::WallpaperSet(self)
    }
}

impl TryFrom<Response> for WallpaperSet {
    type Error = ();
    
    fn try_from(response: Response) -> Result<Self, Self::Error> {
        match response {
            Response::WallpaperSet(wallpaper_set) => Ok(wallpaper_set),
            _ => Err(()),
        }
    }
}
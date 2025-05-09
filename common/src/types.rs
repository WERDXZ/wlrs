use bincode::{Decode, Encode};
use std::convert::TryFrom;

/// Trait for converting a type into a Request enum variant
pub trait IntoRequest {
    /// The response type that corresponds to this request
    type Response;

    /// Convert this type into a Request enum variant
    fn into_request(self) -> Request;
}

/// Trait for converting a type into a Response enum variant
pub trait IntoResponse {
    /// The request type that this response corresponds to
    type Request;

    /// Convert this type into a Response enum variant
    fn into_response(self) -> Response;
}

/// Define request-response type pairs for clearer association
pub mod type_pairs {
    use super::*;

    pub type CheckhealthRequest = Checkhealth;
    pub type CheckhealthResponse = Health;

    pub type LoadWallpaperRequest = LoadWallpaper;
    pub type LoadWallpaperResponse = WallpaperLoaded;

    pub type GetCurrentWallpaperRequest = GetCurrentWallpaper;
    pub type GetCurrentWallpaperResponse = CurrentWallpaper;

    pub type ListWallpapersRequest = ListWallpapers;
    pub type ListWallpapersResponse = WallpaperList;

    pub type InstallWallpaperRequest = InstallWallpaper;
    pub type InstallWallpaperResponse = WallpaperInstalled;

    pub type SetCurrentWallpaperRequest = SetCurrentWallpaper;
    pub type SetCurrentWallpaperResponse = WallpaperSet;

    pub type StopServerRequest = StopServer;
    pub type StopServerResponse = ServerStopping;

    pub type QueryActiveWallpapersRequest = QueryActiveWallpapers;
    pub type QueryActiveWallpapersResponse = ActiveWallpaperList;
}

/// Macro to implement request-response conversion traits
macro_rules! impl_request_response_pair {
    ($req:ty, $resp:ty, $req_variant:ident, $resp_variant:ident) => {
        impl IntoRequest for $req {
            type Response = $resp;
            fn into_request(self) -> Request {
                Request::$req_variant(self)
            }
        }

        impl TryFrom<Request> for $req {
            type Error = ();

            fn try_from(request: Request) -> Result<Self, Self::Error> {
                match request {
                    Request::$req_variant(req) => Ok(req),
                    _ => Err(()),
                }
            }
        }

        impl IntoResponse for $resp {
            type Request = $req;
            fn into_response(self) -> Response {
                Response::$resp_variant(self)
            }
        }

        impl TryFrom<Response> for $resp {
            type Error = ();

            fn try_from(response: Response) -> Result<Self, Self::Error> {
                match response {
                    Response::$resp_variant(resp) => Ok(resp),
                    _ => Err(()),
                }
            }
        }
    };
}

/// Request to check if the server is alive
#[derive(Encode, Decode, Debug)]
pub struct Checkhealth;

/// Response to a Checkhealth request
#[derive(Encode, Decode, Debug)]
pub struct Health(pub bool);

/// Request to load a wallpaper into cache by name
/// 
/// This request will load the wallpaper with the given name into memory cache
/// but will not set it as the current wallpaper.
#[derive(Encode, Decode, Debug)]
pub struct LoadWallpaper {
    /// Name of the wallpaper to load
    pub path: String,
}

/// Response indicating if a wallpaper was successfully loaded into cache
#[derive(Encode, Decode, Debug)]
pub struct WallpaperLoaded {
    /// Name of the loaded wallpaper
    pub name: String,
    /// Whether the wallpaper was loaded successfully
    pub success: bool,
    /// Error message if loading failed
    pub error: Option<String>,
}

/// Request to get information about the currently active wallpaper
#[derive(Encode, Decode, Debug)]
pub struct GetCurrentWallpaper;

/// Response containing information about the current wallpaper
#[derive(Encode, Decode, Debug)]
pub struct CurrentWallpaper {
    /// Name of the current wallpaper, if any is set
    pub name: Option<String>,
    /// Path to the current wallpaper, if any is set
    pub path: Option<String>,
}

/// Request to list all available wallpapers
#[derive(Encode, Decode, Debug)]
pub struct ListWallpapers;

/// Response containing a list of all available wallpapers
#[derive(Encode, Decode, Debug)]
pub struct WallpaperList {
    /// Vector of available wallpaper information
    pub wallpapers: Vec<WallpaperInfo>,
}

/// Information about a single wallpaper
#[derive(Encode, Decode, Debug)]
pub struct WallpaperInfo {
    /// Name of the wallpaper
    pub name: String,
    /// Path to the wallpaper directory
    pub path: String,
}

/// Request to install a new wallpaper from a directory
/// 
/// This takes a directory containing a wallpaper manifest and installs it to the data directory.
/// The wallpaper can be given a custom name or will use the directory name as default.
#[derive(Encode, Decode, Debug)]
pub struct InstallWallpaper {
    /// Path to the directory containing the wallpaper files and manifest
    pub path: String,
    /// Optional custom name, defaults to directory name if not specified
    pub name: Option<String>,
}

/// Response indicating if a wallpaper was successfully installed
#[derive(Encode, Decode, Debug)]
pub struct WallpaperInstalled {
    /// Name of the installed wallpaper
    pub name: String,
    /// Whether the wallpaper was installed successfully
    pub success: bool,
    /// Error message if installation failed
    pub error: Option<String>,
}

/// Request to set a wallpaper as the current active wallpaper
/// 
/// This will set the specified wallpaper as the current wallpaper and load it if necessary.
/// If the wallpaper is not already loaded in cache, it will be loaded first.
#[derive(Encode, Decode, Debug)]
pub struct SetCurrentWallpaper {
    /// Name of the wallpaper to set as current
    pub name: String,
    /// Optional monitor to set the wallpaper for, if not specified will set for all monitors
    pub monitor: Option<String>,
}

/// Response indicating if a wallpaper was successfully set as current
#[derive(Encode, Decode, Debug)]
pub struct WallpaperSet {
    /// Name of the wallpaper that was set
    pub name: String,
    /// Whether the wallpaper was set successfully
    pub success: bool,
    /// Error message if setting the wallpaper failed
    pub error: Option<String>,
}

/// Request to gracefully stop the server
///
/// This will initiate a clean shutdown of the server, closing connections and releasing resources.
#[derive(Encode, Decode, Debug)]
pub struct StopServer;

/// Response indicating the server is shutting down
#[derive(Encode, Decode, Debug)]
pub struct ServerStopping {
    /// Whether the shutdown was initiated successfully
    pub success: bool,
}

/// Request to query active wallpapers on all monitors
///
/// This will return a list of all currently active wallpapers across all monitors.
#[derive(Encode, Decode, Debug)]
pub struct QueryActiveWallpapers;

/// Information about a single active wallpaper
#[derive(Encode, Decode, Debug)]
pub struct ActiveWallpaperInfo {
    /// Name of the wallpaper
    pub name: String,
    /// Output/monitor name the wallpaper is displayed on
    pub output_name: String,
    /// Width of the wallpaper
    pub width: u32,
    /// Height of the wallpaper
    pub height: u32,
}

/// Response containing a list of all active wallpapers
#[derive(Encode, Decode, Debug)]
pub struct ActiveWallpaperList {
    /// Vector of active wallpaper information
    pub wallpapers: Vec<ActiveWallpaperInfo>,
    /// Whether the query was successful
    pub success: bool,
    /// Error message if query failed
    pub error: Option<String>,
}

/// All possible request types that can be sent to the server
///
/// Each variant corresponds to a specific request type and has a matching
/// response type in the Response enum.
#[derive(Encode, Decode, Debug)]
pub enum Request {
    // Variant                          // Response Type
    Checkhealth(Checkhealth),           // -> Health
    LoadWallpaper(LoadWallpaper),       // -> WallpaperLoaded
    GetCurrentWallpaper(GetCurrentWallpaper), // -> CurrentWallpaper
    ListWallpapers(ListWallpapers),     // -> WallpaperList
    InstallWallpaper(InstallWallpaper), // -> WallpaperInstalled
    SetCurrentWallpaper(SetCurrentWallpaper), // -> WallpaperSet
    StopServer(StopServer),             // -> ServerStopping
    QueryActiveWallpapers(QueryActiveWallpapers), // -> ActiveWallpaperList
}

/// All possible response types that can be received from the server
///
/// Each variant corresponds to a specific response type and matches
/// a request type in the Request enum.
#[derive(Encode, Decode, Debug)]
pub enum Response {
    // Variant                          // Request Type
    Health(Health),                      // <- Checkhealth
    WallpaperLoaded(WallpaperLoaded),    // <- LoadWallpaper
    CurrentWallpaper(CurrentWallpaper),  // <- GetCurrentWallpaper
    WallpaperList(WallpaperList),        // <- ListWallpapers
    WallpaperInstalled(WallpaperInstalled), // <- InstallWallpaper
    WallpaperSet(WallpaperSet),          // <- SetCurrentWallpaper
    ServerStopping(ServerStopping),      // <- StopServer
    ActiveWallpaperList(ActiveWallpaperList), // <- QueryActiveWallpapers
}

// Use the macro to implement all request-response pairs
impl_request_response_pair!(Checkhealth, Health, Checkhealth, Health);
impl_request_response_pair!(
    LoadWallpaper,
    WallpaperLoaded,
    LoadWallpaper,
    WallpaperLoaded
);
impl_request_response_pair!(
    GetCurrentWallpaper,
    CurrentWallpaper,
    GetCurrentWallpaper,
    CurrentWallpaper
);
impl_request_response_pair!(ListWallpapers, WallpaperList, ListWallpapers, WallpaperList);
impl_request_response_pair!(
    InstallWallpaper,
    WallpaperInstalled,
    InstallWallpaper,
    WallpaperInstalled
);
impl_request_response_pair!(
    SetCurrentWallpaper,
    WallpaperSet,
    SetCurrentWallpaper,
    WallpaperSet
);
impl_request_response_pair!(StopServer, ServerStopping, StopServer, ServerStopping);
impl_request_response_pair!(
    QueryActiveWallpapers,
    ActiveWallpaperList,
    QueryActiveWallpapers,
    ActiveWallpaperList
);


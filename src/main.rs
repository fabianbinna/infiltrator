use base64::Engine;
use std::{
    io::{prelude::*, SeekFrom}, 
    fs::File, 
    path::{Path, PathBuf}
};
use rocket::{
    fs::{FileServer, relative, NamedFile}, 
    figment::{
        Figment, 
        providers::{Serialized, Toml, Env, Format}, 
        Profile
    },  
    fairing::AdHoc, 
    serde::{Serialize, Deserialize}, 
    State, 
    response::status::NotFound, 
    Rocket, 
    Build
};

#[macro_use] extern crate rocket;

/*
In Rust, a "prelude" refers to a set of commonly used modules and traits that are 
automatically imported into every Rust program to provide a basic foundation for common 
operations.
*/

/*
TODO:
- DONE: Download files chunked in 10MB base64
- DONE: Reconstruct and save file
- DONE: config
- async requests
- concurrency of file access
- build script
- docker build infra
*/

#[get("/download/<path..>?size")]
fn download(path: PathBuf) -> Result<String, NotFound<String>> {
    // concurrency
    let file = match File::open(Path::new("data").join(path)) {
        Ok(file) => file,
        Err(_) => return Result::Err(NotFound(String::from("Could not find file.")))
    };

    // error handling?
    let file_size = file.metadata().unwrap().len();

    Result::Ok(format!("{file_size}"))
}

#[get("/download/<path..>?<part>")]
fn download_part(config: &State<Config>, path: PathBuf, part: u64) -> Result<String, NotFound<String>> {
    // concurrency
    let mut file = match File::open(Path::new("data").join(path)) {
        Ok(file) => file,
        Err(_) => return Result::Err(NotFound(String::from("Could not find file.")))
    };

    // error handling?
    let file_size = file.metadata().unwrap().len();
    let part_size = config.part_size_bytes;
   
    let start_position: u64 = part * part_size;
    let mut end_position: u64 = part * part_size + part_size;

    if start_position >= file_size {
       return Result::Ok(String::from(""));
    }

    if end_position > file_size {
        end_position = file_size;
    }

    // error handling?
    file.seek(SeekFrom::Start(start_position)).unwrap();

    let mut buffer = vec![0; (end_position - start_position) as usize];
    // error handling?
    file.read_exact(&mut buffer).unwrap();

    Result::Ok(base64::engine::general_purpose::STANDARD_NO_PAD.encode(buffer))
}

#[catch(404)]
async fn not_found() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/404.html")).await.ok()
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Config {
    part_size_bytes: u64,
}

impl Default for Config {
    fn default() -> Config {
        Config {part_size_bytes: 8388608}
    }
}

fn rocket() -> Rocket<Build> {
    let figment = Figment::from(rocket::Config::default())
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("Config.toml").nested())
        .merge(Env::prefixed("INFILTRATOR_").global())
        .select(Profile::from_env_or("INFILTRATOR_PROFILE", "release"));

    rocket::custom(figment)
        .mount("/", FileServer::from(relative!("static")))
        .mount("/", routes![download, download_part])
        .register("/", catchers![not_found])
        .attach(AdHoc::config::<Config>())
}

#[rocket::main]
async fn main() {
    if let Err(e) = rocket().launch().await {
        eprintln!("Could not launch server!");
        drop(e);
    }
}

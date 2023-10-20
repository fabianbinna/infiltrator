use base64::Engine;
use std::{
    io::{prelude::*, SeekFrom}, 
    fs::{File, OpenOptions}, 
    path::{Path, PathBuf}
};
use rocket::{
    fs::{FileServer, NamedFile}, 
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

#[get("/download/<path..>?size")]
fn download(config: &State<Config>, path: PathBuf) -> Result<String, NotFound<String>> {
    let file = match File::open(Path::new(config.data_path.as_str()).join(path)) {
        Ok(file) => file,
        Err(_) => return Result::Err(NotFound(String::from("Could not find file.")))
    };

    let file_size = file.metadata().unwrap().len();

    Result::Ok(format!("{file_size}"))
}

#[get("/download/<path..>?<part>")]
fn download_part(config: &State<Config>, path: PathBuf, part: u64) -> Result<String, NotFound<String>> {
    let mut file = match File::open(Path::new(config.data_path.as_str()).join(path)) {
        Ok(file) => file,
        Err(_) => return Result::Err(NotFound(String::from("Could not find file.")))
    };

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

    file.seek(SeekFrom::Start(start_position)).unwrap();

    let mut buffer = vec![0; (end_position - start_position) as usize];
    file.read_exact(&mut buffer).unwrap();

    Result::Ok(base64::engine::general_purpose::STANDARD_NO_PAD.encode(buffer))
}

#[post("/upload/<path..>", data = "<input>")]
fn upload(config: &State<Config>, path: PathBuf, input: String) { 
    // let mut a = File::create(Path::new(config.data_path.as_str()).join(&filepath)).unwrap();
    // a.flush();
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(Path::new(config.data_path.as_str()).join(path))
        .unwrap();

    let a = base64::engine::general_purpose::STANDARD.decode(input).unwrap();
    file.write(a.as_slice());
}

#[catch(404)]
async fn not_found() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/404.html")).await.ok()
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Config {
    part_size_bytes: u64,
    data_path: String
}

impl Default for Config {
    fn default() -> Config {
        Config {
            part_size_bytes: 8388608,
            data_path: String::from("data/")
        }
    }
}

fn rocket() -> Rocket<Build> {
    let figment = Figment::from(rocket::Config::default())
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("Config.toml").nested())
        .merge(Env::prefixed("INFILTRATOR_").global())
        .select(Profile::from_env_or("INFILTRATOR_PROFILE", "default"));

    rocket::custom(figment)
        .mount("/", FileServer::from("static"))
        .mount("/", routes![download, download_part, upload])
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

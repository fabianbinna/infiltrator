use base64::Engine;
use uuid::Uuid;
use std::{
    io::{prelude::*, SeekFrom}, 
    fs::{File, OpenOptions}, 
    path::{Path, PathBuf}, collections::HashMap, sync::{Arc, RwLock}, os::windows::prelude::AsHandle
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
    response::status::{NotFound, BadRequest, Created}, 
    Rocket, 
    Build
};

#[macro_use] extern crate rocket;

#[get("/download/<path..>?size")]
fn download_size(config: &State<Config>, path: PathBuf) -> Result<String, NotFound<String>> {
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

#[post("/upload/<filename>/reserve")]
fn upload_reserve(config: &State<Config>, currentUploads: &State<CurrentUploads>, filename: String) -> Result<Created<String>, BadRequest<String>> {
    let filepath = Path::new(config.data_path.as_str()).join(&filename);
    let mut map = currentUploads.currentUploadMap.write().unwrap();

    if filepath.exists() {
        return Result::Err(BadRequest(Some(String::from("File already exists."))));
    }

    let uuid = Uuid::new_v4().to_string();
    File::create(Path::new(config.data_path.as_str()).join(&filename)).unwrap();
    map.insert(uuid.clone(), filename);

    Result::Ok(Created::new(uuid.to_string()))
}

#[post("/upload/<uuid>", data = "<input>")]
fn upload_part(config: &State<Config>, currentUploads: &State<CurrentUploads>, uuid: String, input: String) -> Result<(), BadRequest<String>> { 
    let filename = match get_filename(currentUploads, &uuid) {
        Some(filename) => filename,
        None => return Result::Err(BadRequest(Some(String::from("Unknown UUID."))))
    };
    
    let filepath = Path::new(config.data_path.as_str()).join(filename);
    let mut file = match OpenOptions::new().write(true).append(true).open(filepath) {
        Ok(file) => file,
        Err(_) => {
            let mut map = currentUploads.currentUploadMap.write().unwrap();
            map.remove(&uuid);
            println!("deleted {}", &uuid);
            return Result::Err(BadRequest(Some(String::from("The file disappeared magically :("))));
        }
    };

    let bytes = match base64::engine::general_purpose::STANDARD.decode(input) {
        Ok(bytes) => bytes,
        Err(_) => return Result::Err(BadRequest(Some(String::from("Invalid payload."))))
    };
    
    file.write(bytes.as_slice()).unwrap();
    Result::Ok(())
}

// move to struct as member
fn get_filename(currentUploads: &State<CurrentUploads>, uuid: &String) -> Option<String> {
    let map = currentUploads.currentUploadMap.read().unwrap();
    map.get(uuid).cloned()
}

#[post("/upload/<uuid>/commit")]
fn upload_commit(currentUploads: &State<CurrentUploads>, uuid: String) {
    let mut map = currentUploads.currentUploadMap.write().unwrap();
    map.remove(&uuid);
    println!("Committed: {}", &uuid);
    println!("{:?}", map);
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

struct CurrentUploads {
    currentUploadMap: Arc<RwLock<HashMap<String, String>>>
}

fn rocket() -> Rocket<Build> {
    let figment = Figment::from(rocket::Config::default())
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("Config.toml").nested())
        .merge(Env::prefixed("INFILTRATOR_").global())
        .select(Profile::from_env_or("INFILTRATOR_PROFILE", "default"));

    rocket::custom(figment)
        .mount("/", FileServer::from("static"))
        // refactor
        .mount("/", routes![
            download_size, 
            download_part,
            upload_reserve,
            upload_part,
            upload_commit
        ])
        .register("/", catchers![not_found])
        .manage(CurrentUploads{ currentUploadMap: Arc::new(RwLock::new(HashMap::new())) })
        .attach(AdHoc::config::<Config>())
}

#[rocket::main]
async fn main() {
    if let Err(e) = rocket().launch().await {
        eprintln!("Could not launch server!");
        drop(e);
    }
}

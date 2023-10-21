use base64::Engine;
use fs_extra::dir::get_size;
use uuid::Uuid;
use std::{
    io::{prelude::*, SeekFrom}, 
    fs::{File, OpenOptions}, 
    path::{Path, PathBuf}, collections::HashMap, sync::{Arc, RwLock}, time::{Instant, Duration}
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

#[post("/upload/<filename>/reserve?<size>")]
fn upload_reserve(config: &State<Config>, current_uploads: &State<CurrentUploads>, filename: String, size: usize) -> Result<Created<String>, BadRequest<String>> {
    let mut map: std::sync::RwLockWriteGuard<'_, HashMap<String, UploadReservation>> = current_uploads.map.write().unwrap();

    let filepath = Path::new(config.data_path.as_str()).join(&filename);
    if filepath.exists() {
        return Result::Err(BadRequest(Some(String::from("File already exists."))));
    }

    remove_expired_reservations(&mut map);

    if map.len() >= config.max_simultaneous_uploads {
        return Result::Err(BadRequest(Some(String::from("Maximum simultaneous uploads reached."))));
    }

    let data_dir_size = get_size(&config.data_path).unwrap() as usize;
    let upload_size = estimated_size_total(&map);
    
    println!("dir: {}, upload: {}, file: {}, total: {}", data_dir_size, upload_size, size, (data_dir_size + upload_size + size));
    if data_dir_size + upload_size + size >= config.max_upload_directory_size {
        return Result::Err(BadRequest(Some(String::from("Maximum upload directory size reached."))));
    }
    
    let uuid = Uuid::new_v4().to_string();
    File::create(Path::new(config.data_path.as_str()).join(&filename)).unwrap();
    map.insert(uuid.clone(), UploadReservation::new(filename, size));

    Result::Ok(Created::new(uuid.to_string()))
}

#[post("/upload/<uuid>", data = "<input>")]
fn upload_part(config: &State<Config>, current_uploads: &State<CurrentUploads>, uuid: String, input: String) -> Result<(), BadRequest<String>> { 
    let filename = match current_uploads.get_filename(&uuid) {
        Some(filename) => filename,
        None => return Result::Err(BadRequest(Some(String::from("Unknown UUID."))))
    };
    
    let filepath = Path::new(config.data_path.as_str()).join(filename);
    let mut file = match OpenOptions::new().write(true).append(true).open(filepath) {
        Ok(file) => file,
        Err(_) => {
            current_uploads.delete_reservation(&uuid);
            println!("deleted {}", &uuid);
            // return Result::Err(BadRequest(Some(String::from("The file disappeared magically :("))));
            panic!("Someone deleted your file :(");
        }
    };

    let bytes = match base64::engine::general_purpose::STANDARD.decode(input) {
        Ok(bytes) => bytes,
        Err(_) => return Result::Err(BadRequest(Some(String::from("Invalid payload."))))
    };

    file.write(bytes.as_slice()).unwrap();
    Result::Ok(())
}

#[post("/upload/<uuid>/commit")]
fn upload_commit(current_uploads: &State<CurrentUploads>, uuid: String) {
    current_uploads.delete_reservation(&uuid);
    println!("Committed: {}", &uuid);
}

#[catch(404)]
async fn not_found() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/404.html")).await.ok()
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Config {
    part_size_bytes: u64,
    data_path: String,
    max_simultaneous_uploads: usize,
    max_upload_directory_size: usize
}

impl Default for Config {
    fn default() -> Config {
        Config {
            part_size_bytes: 8388608,
            data_path: String::from("data/"),
            max_simultaneous_uploads: 10,
            max_upload_directory_size: 1000000000
        }
    }
}

struct CurrentUploads {
    map: Arc<RwLock<HashMap<String, UploadReservation>>>
}

impl CurrentUploads {

    fn get_filename(&self, uuid: &String) -> Option<String> {
        let map = self.map.read().unwrap();
        match map.get(uuid) {
            Some(reservation) => Some(reservation.filename.clone()),
            None => None
        }
    }

    fn delete_reservation(&self, uuid: &String) {
        let mut map  = self.map.write().unwrap();
        map.remove(uuid);
    }
}

fn remove_expired_reservations(map: &mut std::sync::RwLockWriteGuard<'_, HashMap<String, UploadReservation>>) {
    let mut to_remove = Vec::new();
    for (uuid, reservation) in map.iter() {
        if reservation.creation.elapsed() >= Duration::from_secs(600) {
            to_remove.push(uuid.to_owned());
        }
    }
    for key in to_remove.iter() {
        map.remove(key);
    }
}

fn estimated_size_total(map: &std::sync::RwLockWriteGuard<'_, HashMap<String, UploadReservation>>) -> usize {
    map.values()
        .map(|reservation| reservation.estimated_size_bytes)
        .sum()
}

struct UploadReservation {
    filename: String,
    creation: Instant,
    estimated_size_bytes: usize
}

impl UploadReservation {
    fn new(filename: String, estimated_size_bytes: usize) -> Self {
        UploadReservation{
            filename,
            creation: Instant::now(),
            estimated_size_bytes
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
        // refactor
        .mount("/", routes![
            download_size, 
            download_part,
            upload_reserve,
            upload_part,
            upload_commit
        ])
        .register("/", catchers![not_found])
        .manage(CurrentUploads{ map: Arc::new(RwLock::new(HashMap::new())) })
        .attach(AdHoc::config::<Config>())
}

#[rocket::main]
async fn main() {
    if let Err(e) = rocket().launch().await {
        eprintln!("Could not launch server!");
        drop(e);
    }
}

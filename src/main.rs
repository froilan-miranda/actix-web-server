use actix_cors::Cors;
use actix_web::{ http::header, web, App, HttpServer, Responder, HttpResponse };
use serde:: { Deserialize, Serialize };
use reqwest::Client as HttpClient;
use async_trait::async_trait;

use std::sync::Mutex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Task {
    id: u64,
    name: String,
    completed: bool
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: u64,
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Database {
    tasks: HashMap<u64, Task>,
    users: HashMap<u64, User>,
}


impl Database {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            users: HashMap::new(),
        }
    }

    // CRUD DATA
    fn insert(&mut self, task: Task) {
        self.tasks.insert(task.id, task);
    }
    fn get(&self, id: &u64) -> Option<&Task> {
        self.tasks.get(id) 
    }
    fn get_all(&self) -> Vec<&Task> {
       self.tesks.values().collect() 
    }
    fn delete(&mut self, id: &u64) {
        self.tasks.remove(id);
    }
    fn update(&mut self, task: Task) {
        self.tasks.insert(task.id, task);
    }


    // USER DATA RELATED FUNCTION
    fn insert_user(&mut self, user: User) {
        self.users.insert(user.id, user); 
    }
    fn get_user_by_name(&self, username: &str) -> Option<&User> {
        self.users.values().find(|u:&User| u.username == username)
    }
    
    // DATABASE SAVING
    fn save_to_file(&self) -> std::io:Result<()> {
        let data: Result<String, Error> = serde_json::to_string(&self)?;
        let mut file = fs::File::create("database.json")?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    fn load_from_file() -> std::io::Result<Self> {
        let file_content = fs::read_to_string("database.json")?;
        let db: Database = serde_json::from_str(&file_content)?;
        Ok(db)
    }

}

struct AppState {
    db: Mutex<Database>,
}

async fn create_task(app_state: web::Data<AppState>, task: web::Json<Task>) -> impl Responder {
    let mut db = app_state.db.lock().unwrap();
    db.insert(task.into_inner());
    let _ =  db.save_to_file();
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io:Result<()> {
    let db: Database = match Database::load_from_file() {
        Ok(db) => db,
        Err(_) => Database::new(),
    }


    let data = web::Data::new(AppState {
        db: Mutex::new(db),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::permissive()
                    .allowed_origin_fn(|origin: &HeaderValue, _req_head: &RequestHead| {{{
                    origin.as_bytes().starts_with(b"http://127.0.0.1") || origin == "null"
                })
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600)

            )
            .app_data(data.clone())
            .route("/task", web::post().to(create_task))
    })
    .bind("127.0.0.1:8080")?
    .run()
}

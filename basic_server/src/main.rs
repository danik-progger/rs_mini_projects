use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct CreateUserResponse {
    id: u32,
    name: String,
}

type UserDB = Arc<Mutex<HashMap<u32, User>>>;

#[actix_web::get("/greet/{id}")]
async fn get_user(user_id: web::Path<u32>, db: web::Data<UserDB>) -> impl Responder {
    let user_id = user_id.into_inner();
    let db = db.lock().unwrap();
    match db.get(&user_id) {
        Some(user) => HttpResponse::Ok().json(user),
        None => HttpResponse::NotFound().finish(),
    }
}

#[actix_web::post("/greet/{id}")]
async fn create_user(user: web::Json<User>, db: web::Data<UserDB>) -> impl Responder {
    let mut db = db.lock().unwrap();
    let id = db.keys().max().unwrap_or(&0) + 1;
    let name = user.name.clone();
    db.insert(id, user.into_inner());
    HttpResponse::Created().json(CreateUserResponse { id, name })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 8085;
    print!("Starting server on port {port}");

    let user_db: UserDB = Arc::new(Mutex::new(HashMap::<u32, User>::new()));

    HttpServer::new(move || {
        let app_data = web::Data::new(user_db.clone());
        App::new()
            .app_data(app_data)
            .service(get_user)
            .service(create_user)
    })
    .bind(("127.0.0.1", port))?
    .workers(4)
    .run()
    .await
}

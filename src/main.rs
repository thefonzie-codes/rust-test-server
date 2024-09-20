use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use rusqlite::{params, Connection, Result};
use serde::{Serialize, Deserialize};
use log::{info, error};

#[derive(Serialize)]
struct User {
    id: i32,
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct AuthRequest {
    user_id: i32,
}

#[derive(Serialize)]
struct Plant {
    species: String,
    user_id: i32,
}

#[derive(Serialize)]
struct UserResponse {
    user: User,
    plants: Vec<Plant>,
}

#[get("/users")]
async fn get_users() -> HttpResponse {
    match fetch_users_from_db() {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(err) => {
            error!("Error fetching users: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

fn fetch_users_from_db() -> Result<Vec<User>> {
    let conn = Connection::open("mockdb.db")?;
    let mut stmt = conn.prepare("SELECT id, name, email FROM users")?;
    let user_iter = stmt.query_map([], |row| {
        Ok(User {
            id: row.get(0)?,
            name: row.get(1)?,
            email: row.get(2)?,
        })
    })?;

    let users: Result<Vec<User>, rusqlite::Error> = user_iter.collect();
    users
}

#[post("/auth")]
async fn login(auth_data: web::Json<AuthRequest>) -> HttpResponse {
    let user_id = auth_data.user_id;
    info!("User ID: {}", user_id);

    match get_user_with_plants_from_db(user_id) {
        Ok((user, plants)) => {
            let response = UserResponse { user, plants };
            HttpResponse::Ok().json(response)
        },
        Err(err) => {
            error!("Could not get user: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

fn get_user_with_plants_from_db(id: i32) -> Result<(User, Vec<Plant>)> {
    let conn = Connection::open("mockdb.db")?;
    let mut stmt = conn.prepare("
        SELECT users.id, users.name, users.email, plants.species 
        FROM users 
        LEFT JOIN plants ON users.id = plants.user_id 
        WHERE users.id = ?1
    ")?;

    let mut user_info: Option<User> = None;
    let mut plants = Vec::new();

    let user_iter = stmt.query_map(params![id], |row| {
        if user_info.is_none() {
            user_info = Some(User {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
            });
        }

        if let Ok(Some(species)) = row.get::<_, Option<String>>(3) {
            plants.push(Plant { species, user_id: id });
        }

        Ok::<(), rusqlite::Error>(())
    })?;

    user_iter.collect::<Result<(), rusqlite::Error>>()?; // Collect results for error handling

    let user_info = user_info.ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?;

    Ok((user_info, plants))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init(); // Initialize the logger
    HttpServer::new(|| {
        App::new()
            .service(get_users)
            .service(login)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

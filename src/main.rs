use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use rusqlite::{params, Connection,Result};
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
struct User {
    id: i32,
    name: String,
}

#[derive(Deserialize)]
struct AuthRequest {
    user_id: i32,
}

#[get("/users")]
async fn get_users() -> HttpResponse {
    match fetch_users_from_db() {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(err) => {
            eprintln!("Error fetching users: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

fn fetch_users_from_db() -> Result<Vec<User>> {
    let conn = Connection::open("mockdb.db")?;
    let mut stmt = conn.prepare("SELECT id, name FROM users")?;
    let user_iter = stmt.query_map([], |row| {
        Ok(User {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;

    let mut users = Vec::new();
    for user in user_iter {
        users.push(user?);
    }
    Ok(users)
}

#[post("/auth")]
async fn login(auth_data: web::Json<AuthRequest>) -> HttpResponse {
    let user_id = auth_data.user_id;
    println!("user id: {user_id}");

    match get_user_from_db(user_id) {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(err) => {
            eprintln!("Could not get user: {}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

fn get_user_from_db( id:i32 ) -> Result<User> {
    let conn = Connection::open("mockdb.db")?;
    let mut stmt = conn.prepare("SELECT id, name FROM users WHERE id =?1")?;
    let user = stmt.query_row(params![id], |row| {
        Ok(User {
            id: row.get(0)?,
            name: row.get(1)?
        })
    })?;

    Ok(user)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(get_users)
            .service(login)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

use std::time::Duration;

use actix_web::{
    get,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, FromRow, Pool, Postgres};

#[derive(Clone)]
struct AppState {
    pool: Pool<Postgres>,
}
#[derive(FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(FromRow)]
pub struct Employment {
    pub id: i32,
    pub employmentnumber: i32,
    pub user_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct UserDto {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct EmploymentDto {
    pub id: i32,
    pub employmentnumber: i32,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        UserDto {
            id: user.id,
            name: user.name,
        }
    }
}

impl From<Employment> for EmploymentDto {
    fn from(employment: Employment) -> Self {
        EmploymentDto {
            id: employment.id,
            employmentnumber: employment.employmentnumber,
        }
    }
}

impl From<(User, Vec<Employment>)> for UserWithEmploymentsDto {
    fn from((user, employments): (User, Vec<Employment>)) -> Self {
        UserWithEmploymentsDto {
            user: user.into(),
            employments: employments.into_iter().map(|e| e.into()).collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserWithEmploymentsDto {
    pub user: UserDto,
    pub employments: Vec<EmploymentDto>,
}

#[get("/api/users")]
async fn get_users(app: web::Data<AppState>) -> impl Responder {
    let users: Vec<User> = sqlx::query_as("SELECT * FROM users")
        .fetch_all(&app.pool)
        .await
        .unwrap();

    let mut dtos: Vec<UserWithEmploymentsDto> = Vec::new();

    for user in users {
        let get_employments_result = get_employments(&app.pool, user.id).await;

        match get_employments_result {
            Ok(employments) => {
                dtos.push(UserWithEmploymentsDto::from((user, employments)));
            }
            Err(error) => {
                println!("Error: {}", error);
            }
        }
    }

    HttpResponse::Ok().json(dtos)
}

async fn get_employments(pool: &Pool<Postgres>, id: i32) -> Result<Vec<Employment>, sqlx::Error> {
    let result = sqlx::query_as("SELECT * FROM employments WHERE user_id = $1")
        .bind(id)
        .fetch_all(pool)
        .await?;

    Ok(result)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // const DATABASE_URL: &str = "postgres://test:test@127.0.0.1/postgres";
    const DATABASE_URL: &str = "postgres://postgres:root@127.0.0.1/employment";
    let pool = PgPoolOptions::new().max_connections(100).test_before_acquire(false).connect(DATABASE_URL).await.unwrap();
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool)
        .await
        .unwrap();

    // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL)

    assert_eq!(row.0, 150);
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState { pool: pool.clone() }))
            .service(get_users)
    })
    .keep_alive(Duration::from_secs(240))
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
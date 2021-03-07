
use rand::Rng;
use actix_web::{web, HttpRequest, HttpResponse};

use crate::models::{Service, CreateBoard, Board};


// TODO: add oauth exchange for token here

fn check_rate_limit (req: &HttpRequest) -> Result<bool, HttpResponse> {
    let valid_ip = match req.connection_info().remote_addr() {
        Some(ip) => ip != "0.0.0.0",
        _ => false
    };
    if valid_ip {
        Ok(true)
    } else {
        Err(HttpResponse::TooManyRequests().finish())
    }
}

pub async fn list_boards (
    _req: HttpRequest,
    service: web::Data<Service>,
) -> Result<web::Json<Vec<Board>>, HttpResponse> {
    println!("list boards");
    // check_api_key(&req, service.config.api_key_links.as_str())?;

    match service.storage.list_boards().await {
        Ok(boards) => Ok(web::Json(boards)),
        Err(why) => Err(HttpResponse::InternalServerError().body(format!("List boards failed! {}", why))),
    }
}

pub async fn add_board (
    req: HttpRequest,
    payload: web::Json<CreateBoard>,
    service: web::Data<Service>,
) -> Result<web::Json<Board>, HttpResponse> {
    println!("add board");
    // check_api_key(&req, service.config.api_key_links.as_str())?;
    check_rate_limit(&req)?;

    // TODO validate request
    if true {
        let now = service.time_provider.unix_ts_ms();
        // https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html
        let n: u64 = rand::thread_rng().gen();
        let id = format!("{:016x}{:016x}", now, n);

        let board = Board {
            id: id.clone(),
            title: payload.title.clone(),
            owner: payload.owner.clone(), // TODO get from auth token
            created_at: now,
        };

        match service.storage.add_board(&board).await {
            Ok(_) => Ok(web::Json(board)),
            Err(why) => Err(HttpResponse::InternalServerError().body(format!("Add board failed! {}", why))),
        }
    } else {
        Err(HttpResponse::BadRequest().body("Invalid request for create board!"))
    }
}

pub async fn delete_board (
    req: HttpRequest,
    service: web::Data<Service>
) -> HttpResponse {
    println!("delete board");
    if let Err(badreq) = check_rate_limit(&req) {
        return badreq
    }

    let id = req.match_info().get("id").unwrap().to_string();
    match service.storage.delete_board(&id).await {
        Ok(_) => HttpResponse::Ok().body("Board deleted"),
        Err(why) => HttpResponse::InternalServerError().body(format!("Delete board failed! {}", why)),
    }
}

pub fn not_found () -> HttpResponse {
    HttpResponse::NotFound().body("404 DNE")
}

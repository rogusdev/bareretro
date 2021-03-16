
use std::future::Future;
use rand::Rng;
use actix_web::{web, HttpRequest, HttpResponse};

use crate::models::{Service, MyError, CrudObj, CreateBoard, Board};


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

pub async fn add<T : CrudObj,U> (
    req: HttpRequest,
    payload: web::Json<U>,
    service: web::Data<Service>,
) -> Result<web::Json<T>, HttpResponse> where T :  {
    println!("add {}", T::name_single());
    // check_api_key(&req, service.config.api_key_links.as_str())?;
    check_rate_limit(&req)?;

    // TODO validate request
    if true {
        let now = service.time_provider.unix_ts_ms();
        // https://rust-lang-nursery.github.io/rust-cookbook/algorithms/randomness.html
        let n: u64 = rand::thread_rng().gen();
        let id = format!("{:016x}{:016x}", now, n);

        let item = T::new(id, payload, now);

        match service.storage.add(&item).await {
            Ok(_) => Ok(web::Json(item)),
            Err(why) => Err(HttpResponse::InternalServerError().body(format!("Add {} failed! {}", T::name_single(), why))),
        }
    } else {
        Err(HttpResponse::BadRequest().body(format!("Invalid request for create {}!", T::name_single())))
    }
}

pub async fn list<T : CrudObj> (
    _req: HttpRequest,
    service: web::Data<Service>,
) -> Result<web::Json<Vec<T>>, HttpResponse> {
    println!("list {}", T::name_single());
    // check_api_key(&req, service.config.api_key_links.as_str())?;

    match service.storage.list().await {
        Ok(items) => Ok(web::Json(items)),
        Err(why) => Err(HttpResponse::InternalServerError().body(format!("List {} failed! {}", T::name_plural(), why))),
    }
}

pub async fn get<T : CrudObj> (
    req: HttpRequest,
    service: web::Data<Service>
) -> Result<web::Json<T>, HttpResponse> {
    println!("get {}", T::name_single());
    if let Err(badreq) = check_rate_limit(&req) {
        return Err(badreq)
    }

    let id = req.match_info().get("id").unwrap().to_string();
    return match service.storage.get(&id).await {
        Ok(item) => Ok(web::Json(item)),
        Err(why) => Err(HttpResponse::NotFound().body(
            format!("Could not find {} for id {}: {}", T::name_single(), id, why)
        ))
    };
}

pub async fn delete<T : CrudObj> (
    req: HttpRequest,
    service: web::Data<Service>,
    action: fn(&String) -> dyn Future<Output = Result<bool, MyError>>
) -> HttpResponse {
    println!("delete {}", T::name_single());
    if let Err(badreq) = check_rate_limit(&req) {
        return badreq
    }

    let id = req.match_info().get("id").unwrap().to_string();
    match action(&id).await {
        Ok(_) => HttpResponse::Ok().body(format!("Deleted {}", T::name_single())),
        Err(why) => HttpResponse::InternalServerError().body(format!("Delete {} failed! {}", T::name_single(), why)),
    }
}

pub async fn delete_board (
    req: HttpRequest,
    service: web::Data<Service>
) -> HttpResponse {
    delete::<Board>(req, service, service.storage.delete_board)
}

pub fn not_found () -> HttpResponse {
    HttpResponse::NotFound().body("404 DNE")
}

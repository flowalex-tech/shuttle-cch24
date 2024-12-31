use std::net::{Ipv4Addr, Ipv6Addr};

use actix_web::{get, web};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_dest)
        .service(get_key)
        .service(get_v6_dest)
        .service(get_v6_key);
}

#[derive(serde::Deserialize)]
struct DestInfo {
    from: Ipv4Addr,
    key: Ipv4Addr,
}

#[get("/2/dest")]
async fn get_dest(info: web::Query<DestInfo>) -> String {
    let from: Ipv4Addr = info.from;
    let key: Ipv4Addr = info.key;

    let mut octets = [0; 4];

    for (i, o) in octets.iter_mut().enumerate() {
        *o = from.octets()[i].overflowing_add(key.octets()[i]).0;
    }

    let dest = Ipv4Addr::from(octets);

    dest.to_string()
}

#[derive(serde::Deserialize)]
struct KeyInfo {
    from: Ipv4Addr,
    to: Ipv4Addr,
}

#[get("/2/key")]
async fn get_key(info: web::Query<KeyInfo>) -> String {
    let from: Ipv4Addr = info.from;
    let to: Ipv4Addr = info.to;

    let mut octets = [0; 4];

    for (i, o) in octets.iter_mut().enumerate() {
        *o = to.octets()[i].overflowing_sub(from.octets()[i]).0;
    }

    let key = Ipv4Addr::from(octets);

    key.to_string()
}

#[derive(serde::Deserialize)]
struct DestInfoV6 {
    from: Ipv6Addr,
    key: Ipv6Addr,
}

#[get("/2/v6/dest")]
async fn get_v6_dest(info: web::Query<DestInfoV6>) -> String {
    let from: Ipv6Addr = info.from;
    let key: Ipv6Addr = info.key;

    let mut segments = [0; 8];

    for (i, o) in segments.iter_mut().enumerate() {
        *o = from.segments()[i] ^ key.segments()[i];
    }

    let dest = Ipv6Addr::from(segments);

    dest.to_string()
}

#[derive(serde::Deserialize)]
struct KeyInfoV6 {
    from: Ipv6Addr,
    to: Ipv6Addr,
}

#[get("/2/v6/key")]
async fn get_v6_key(info: web::Query<KeyInfoV6>) -> String {
    let from: Ipv6Addr = info.from;
    let to: Ipv6Addr = info.to;

    let mut segments = [0; 8];

    for (i, o) in segments.iter_mut().enumerate() {
        *o = to.segments()[i] ^ from.segments()[i];
    }

    let dest = Ipv6Addr::from(segments);

    dest.to_string()
}
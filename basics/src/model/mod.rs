use actix_web::{HttpRequest, HttpResponse, web};
use chrono::NaiveDate;
use mysql::prelude::*;
use mysql::*;

pub struct User {
  id: i32,
  firstName: String,
  lastName: String,
  age: i32
}

pub fn connect_mysql() -> PooledConn {
  let url = "mysql://root:schacker@localhost:3306/bucky";
  let pool = Pool::new(url).unwrap();
  let mut conn = pool.get_conn().unwrap();
  return conn
}

pub fn query_map(mut conn: PooledConn) -> Vec<User> {
  let res = conn.query_map("select * from user", |(id, firstName, lastName, age)| User {
    id: id,
    lastName: lastName,
    firstName: firstName,
    age: age
  }).expect("Query failed!");
  return res;
}
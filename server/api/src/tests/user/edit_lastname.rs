/*
*  CodeMountain is a free and open source online judge open for everyone
*  Copyright (C) 2021 Uthsob Chakra Borty and contributors
*
*  This program is free software: you can redistribute it and/or modify
*  it under the terms of the GNU General Public License as published by
*  the Free Software Foundation, either version 3 of the License, or
*  (at your option) any later version.
*
*  This program is distributed in the hope that it will be useful,
*  but WITHOUT ANY WARRANTY; without even the implied warranty of
*  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
*  GNU General Public License for more details.
*
*  You should have received a copy of the GNU General Public License
*  along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
use crate::db::create_pool;
use crate::db::user::mutation::edit_lastname;
use crate::endpoints::user::data_update::edit_lastname_handler;
use crate::endpoints::user::payload::LastNamePayload;
use actix_web::{test, web, App};
use std::env::set_var;

static AUTHTOKEN: &'static str = "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1aWQiOjI1LCJleHAiOjk5OTk5OTk5OTksInRva2VuX3R5cGUiOiJBY2Nlc3NUb2tlbiJ9.iIBHQu2ZT4rsdTR_wCTITcCERhOgzGswSt5wWB3sWio";

#[actix_rt::test]
async fn test_edit_lastname_successful() {
    set_var(
        "DATABASE_URL",
        "postgres://postgres:a3b2c100@127.0.0.1/codemountain_test",
    );
    set_var("JWT_SECRET_KEY", "beryberysecret");

    let pool = create_pool();
    let mut app = test::init_service(
        App::new()
            .data(pool.clone())
            .route("/", web::post().to(edit_lastname_handler)),
    )
    .await;

    let req = test::TestRequest::post()
        .header("authorization", AUTHTOKEN)
        .set_json(&LastNamePayload {
            lastname: "Dow".to_string(),
        })
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    // revert the change because we'll need the value to be the previous one
    edit_lastname(25, &"Doe".to_string(), &pool)
        .expect("Failed to revert the username");

    dbg!(resp.response());
    assert!(resp.status().is_success(), "This should be successful");
}

/// Lastnames doesn't require to be unique, that's why we are going to test if it accepts invalid
/// lastnames or not
#[actix_rt::test]
async fn test_edit_lastname_unsuccessful() {
    set_var(
        "DATABASE_URL",
        "postgres://postgres:a3b2c100@127.0.0.1/codemountain_test",
    );
    set_var("JWT_SECRET_KEY", "beryberysecret");

    let pool = create_pool();
    let mut app = test::init_service(
        App::new()
            .data(pool.clone())
            .route("/", web::post().to(edit_lastname_handler)),
    )
    .await;

    let req = test::TestRequest::post()
        .header("authorization", AUTHTOKEN)
        .set_json(&LastNamePayload {
            lastname: "Do".to_string(),
        })
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    dbg!(resp.response());
    assert!(
        resp.status().is_client_error(),
        "This should be unsuccessful"
    );
}

/*
 *  CodeMountain is a free and open source online judge open for everyone
 *  Copyright (C) 2021 MD Gaziur Rahman Noor and contributors
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

/*
How password recovery works?

The password recovery is a two step method. First the user sends a request to reset the password.

First step:

The server then checks if the request is valid. Then, it generates a jwt token that can be used to reset the email.
After that, it sends an email containing the token to the user's email inbox.

Seconds step:
Now the user will send the token from the email to the server. The server validates the token and then updates the user's password
with the new one.
*/
use crate::db::user::mutation::update_password;
use crate::db::user::query::get_user_by_email;
use crate::db::user::query::get_user_by_uid;
use crate::db::Pool;
use crate::errors::Errors;
use crate::jwt::sign::generate_passwordresettoken;
use crate::jwt::verify::{decode_without_secret, verify_token_using_custom_secret};
use crate::mailer::mail;
use actix_web::{web::Data, web::Json as actix_json, Responder};
use actix_web_validator::Json;
use bcrypt::{hash, DEFAULT_COST};

use super::payload::ReturnStatus;
use super::payload::{ResetPasswordPayload, SendPasswordResetTokenPayload};

pub async fn send_password_reset_email(
    conn_pool: Data<Pool>,
    payload: Json<SendPasswordResetTokenPayload>,
) -> Result<impl Responder, Errors> {
    let jwt_secret_key = std::env::var("JWT_SECRET_KEY").unwrap();

    let user_email = payload.email.clone();

    // check if the user exists
    let user = get_user_by_email(&user_email, &conn_pool.as_ref())?;

    let reset_token = generate_passwordresettoken(&user.id, &(jwt_secret_key + &user.password))
        .map_err(|_| Errors::InternalServerError)?;

    let mut template = String::from_utf8(
        std::fs::read("email_templates/password_reset.html")
            .map_err(|_| Errors::InternalServerError)?,
    )
    .map_err(|_| Errors::InternalServerError)?;

    template = template.replace("{{lastname}}", &user.lastname);
    template = template.replace("{{username}}", &user.username);
    template = template.replace("{{otp}}", &reset_token);

    match mail(
        template,
        &user.email,
        "Password Recovery For CodeMountainOJ Account",
    ) {
        Ok(_) => Ok(actix_json(ReturnStatus { success: true })),
        Err(_) => Err(Errors::InternalServerError),
    }
}

pub async fn recover_password(
    conn_pool: Data<Pool>,
    payload: Json<ResetPasswordPayload>,
) -> Result<impl Responder, Errors> {
    let jwt_secret_key = std::env::var("JWT_SECRET_KEY").unwrap();

    let token = payload.reset_token.clone();
    let new_password = payload.password.clone();

    let token_data = decode_without_secret(&token).map_err(|_| Errors::InternalServerError)?;
    let user = get_user_by_uid(&token_data.uid, &conn_pool.as_ref())?;

    verify_token_using_custom_secret(&token, &(jwt_secret_key + &user.password))
        .map_err(|_| Errors::BadRequest("Invalid reset token"))?;

    match token_data.token_type {
        crate::jwt::claims::TokenType::PasswordResetToken => (),
        _ => return Err(Errors::BadRequest("Invalid reset token")),
    }

    let user = get_user_by_uid(&token_data.uid, &conn_pool.as_ref())?;

    let salted_password = hash(&new_password, DEFAULT_COST).map_err(|_| Errors::InternalServerError)?;

    match update_password(user.id, &salted_password, &conn_pool.as_ref()) {
        Ok(_) => Ok(actix_json(ReturnStatus { success: true })),
        Err(e) => Err(e),
    }
}

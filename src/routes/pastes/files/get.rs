use database::DbConn;
use database::models::pastes::Paste as DbPaste;
use database::models::files::File as DbFile;
use database::schema::pastes;
use models::paste::{PasteId, Visibility};
use models::paste::output::OutputFile;
use models::status::{Status, ErrorKind};
use routes::{RouteResult, OptionalUser};

use diesel::prelude::*;

use rocket::http::Status as HttpStatus;

#[get("/<paste_id>/files")]
fn get_files(paste_id: PasteId, user: OptionalUser, conn: DbConn) -> RouteResult<Vec<OutputFile>> {
  let paste: Option<DbPaste> = pastes::table.filter(pastes::id.eq(*paste_id)).first(&*conn).optional()?;
  let paste = match paste {
    Some(paste) => paste,
    None => return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste)),
  };

  if paste.visibility() == Visibility::Private && user.as_ref().map(|x| x.id()) != *paste.author_id() {
    return Ok(Status::show_error(HttpStatus::NotFound, ErrorKind::MissingPaste));
  }

  let db_file: Vec<DbFile> = DbFile::belonging_to(&paste).load(&*conn)?;
  let files: Vec<OutputFile> = db_file
    .into_iter()
    .map(|f| OutputFile::new(&f.id(), Some(f.name().clone()), None))
    .collect();

  Ok(Status::show_success(HttpStatus::Ok, files))
}
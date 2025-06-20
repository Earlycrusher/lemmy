use crate::{build_totp_2fa, generate_totp_2fa_secret};
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::source::{
  local_user::{LocalUser, LocalUserUpdateForm},
  site::Site,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::GenerateTotpSecretResponse;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

/// Generate a new secret for two-factor-authentication. Afterwards you need to call [toggle_totp]
/// to enable it. This can only be called if 2FA is currently disabled.
pub async fn generate_totp_secret(
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<GenerateTotpSecretResponse>> {
  let site = Site::read_local(&mut context.pool()).await?;

  if local_user_view.local_user.totp_2fa_enabled {
    return Err(LemmyErrorType::TotpAlreadyEnabled)?;
  }

  let secret = generate_totp_2fa_secret();
  let secret_url = build_totp_2fa(&site.name, &local_user_view.person.name, &secret)?.get_url();

  let local_user_form = LocalUserUpdateForm {
    totp_2fa_secret: Some(Some(secret)),
    ..Default::default()
  };
  LocalUser::update(
    &mut context.pool(),
    local_user_view.local_user.id,
    &local_user_form,
  )
  .await?;

  Ok(Json(GenerateTotpSecretResponse {
    totp_secret_url: secret_url.into(),
  }))
}

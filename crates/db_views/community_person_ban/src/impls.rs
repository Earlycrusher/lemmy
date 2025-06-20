use crate::CommunityPersonBanView;
use diesel::{
  dsl::{exists, not},
  select,
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  utils::{get_conn, DbPool},
};
use lemmy_db_schema_file::schema::community_actions;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

impl CommunityPersonBanView {
  pub async fn check(
    pool: &mut DbPool<'_>,
    from_person_id: PersonId,
    from_community_id: CommunityId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    let find_action = community_actions::table
      .find((from_person_id, from_community_id))
      .filter(community_actions::received_ban_at.is_not_null());
    select(not(exists(find_action)))
      .get_result::<bool>(conn)
      .await?
      .then_some(())
      .ok_or(LemmyErrorType::PersonIsBannedFromCommunity.into())
  }
}

use super::send_activity_from_user_or_community;
use crate::{
  activities::generate_activity_id,
  insert_received_activity,
  protocol::activities::following::{accept::AcceptFollow, follow::Follow},
};
use activitypub_federation::{
  config::Data,
  kinds::activity::AcceptType,
  protocol::verification::verify_urls_match,
  traits::{ActivityHandler, Actor},
};
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{
  source::{activity::ActivitySendTargets, community::CommunityActions},
  traits::Followable,
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

impl AcceptFollow {
  pub async fn send(follow: Follow, context: &Data<LemmyContext>) -> LemmyResult<()> {
    let user_or_community = follow.object.dereference_local(context).await?;
    let person = follow.actor.clone().dereference(context).await?;
    let accept = AcceptFollow {
      actor: user_or_community.id().into(),
      to: Some([person.id().into()]),
      object: follow,
      kind: AcceptType::Accept,
      id: generate_activity_id(
        AcceptType::Accept,
        &context.settings().get_protocol_and_hostname(),
      )?,
    };
    let inbox = ActivitySendTargets::to_inbox(person.shared_inbox_or_inbox());
    send_activity_from_user_or_community(context, accept, user_or_community, inbox).await
  }
}

/// Handle accepted follows
#[async_trait::async_trait]
impl ActivityHandler for AcceptFollow {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    verify_urls_match(self.actor.inner(), self.object.object.inner())?;
    self.object.verify(context).await?;
    if let Some(to) = &self.to {
      verify_urls_match(to[0].inner(), self.object.actor.inner())?;
    }
    Ok(())
  }

  async fn receive(self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    insert_received_activity(&self.id, context).await?;
    let community = self.actor.dereference(context).await?;
    let person = self.object.actor.dereference(context).await?;
    // This will throw an error if no follow was requested
    let community_id = community.id;
    let person_id = person.id;
    CommunityActions::follow_accepted(&mut context.pool(), community_id, person_id).await?;

    Ok(())
  }
}

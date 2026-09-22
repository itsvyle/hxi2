use hxi2_proto::proto::auth::v2::{DBUser, SmallData};
use rand::RngExt;

use crate::{database::DbUser, jwt_signer::JWTSignerOptions};

pub struct LoginManager {
    pub signer: &'static crate::jwt_signer::JWTSigner,
    #[allow(unused)]
    pub verifier: &'static crate::jwt_verifier::JWTVerifier,
}

pub struct LoginResponse {
    pub token: String,
    pub refresh_token: String,
    pub small_data: SmallData,
}

pub enum LoginID {
    UserID(i64),
    DiscordID(String),
}

impl LoginManager {
    pub fn new(
        signer: &'static crate::jwt_signer::JWTSigner,
        verifier: &'static crate::jwt_verifier::JWTVerifier,
    ) -> Self {
        Self { signer, verifier }
    }

    fn small_data_from_user(&self, user: &DbUser) -> SmallData {
        SmallData {
            user_id: user.id,
            username: user.username.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            permissions: user.permissions,
            promotion: user.promotion,
            __buffa_unknown_fields: Default::default(),
        }
    }

    fn new_refresh_token() -> String {
        use rand::rngs::ThreadRng;

        let mut rng = ThreadRng::default();
        let jti: [u8; 16] = rng.random();
        hex::encode(jti)
    }

    // This creates a token for the user
    // Authorization to login as such a user must have been checked PRIOR
    pub async fn login_as(&self, user_id: &LoginID) -> anyhow::Result<LoginResponse> {
        let cfg = crate::app_config::AppConfiguration::INSTANCE();
        let user = match user_id {
            LoginID::UserID(uid) => cfg.db().await.get_db_user_by_id(uid).await?,
            LoginID::DiscordID(did) => cfg.db().await.get_db_user_by_discord_id(did).await?,
        };

        let small_data = self.small_data_from_user(&user);
        let (token, claims) = self.signer.new_token(
            &format!("{}", user.id),
            &small_data,
            &JWTSignerOptions::default(),
        )?;

        let refresh_token = Self::new_refresh_token();

        Ok(LoginResponse {
            token,
            refresh_token,
            small_data: claims.data.expect("claims.data should be present"),
        })
    }
}

use phlow_sdk::prelude::*;
use aws_credential_types::Credentials;
use std::convert::TryFrom;

#[derive(Debug, Clone, Default)]
pub struct Setup {
    pub region: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub session_token: Option<String>,
    pub profile: Option<String>,
    pub endpoint_url: Option<String>,
    pub s3_force_path_style: bool,
    // Assume role fields reserved for future use
    pub assume_role_arn: Option<String>,
    pub assume_role_session_name: Option<String>,
    pub assume_role_external_id: Option<String>,
}

impl TryFrom<Value> for Setup {
    type Error = String;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let region = value.get("region").map(|v| v.to_string());
        let access_key_id = value.get("access_key_id").map(|v| v.to_string());
        let secret_access_key = value.get("secret_access_key").map(|v| v.to_string());
        let session_token = value.get("session_token").map(|v| v.to_string());
        let profile = value.get("profile").map(|v| v.to_string());
        let endpoint_url = value.get("endpoint_url").map(|v| v.to_string());
        let s3_force_path_style = value
            .get("s3_force_path_style")
            .and_then(|v| v.as_bool().cloned())
            .unwrap_or(false);

        let assume_role_arn = value.get("assume_role_arn").map(|v| v.to_string());
        let assume_role_session_name = value
            .get("assume_role_session_name")
            .map(|v| v.to_string());
        let assume_role_external_id = value
            .get("assume_role_external_id")
            .map(|v| v.to_string());

        Ok(Setup {
            region,
            access_key_id,
            secret_access_key,
            session_token,
            profile,
            endpoint_url,
            s3_force_path_style,
            assume_role_arn,
            assume_role_session_name,
            assume_role_external_id,
        })
    }
}

impl Setup {
    pub async fn build_s3_client(
        &self,
    ) -> Result<aws_sdk_s3::Client, Box<dyn std::error::Error + Send + Sync>> {
        use aws_config::BehaviorVersion;
        use aws_config::Region;

        // Base loader with latest behavior
        let mut loader = aws_config::defaults(BehaviorVersion::latest());

        if let Some(profile) = &self.profile {
            loader = loader.profile_name(profile);
        }
        if let Some(region) = &self.region {
            loader = loader.region(Region::new(region.clone()));
        }

        // Explicit static credentials if provided
        if let (Some(akid), Some(sak)) = (&self.access_key_id, &self.secret_access_key) {
            let creds = Credentials::new(
                akid.clone(),
                sak.clone(),
                self.session_token.clone(),
                None,
                "phlow-aws-static",
            );
            loader = loader.credentials_provider(creds);
        }

        let shared = loader.load().await;

        // Build service config with endpoint override and path-style if needed
        let mut s3_cfg_builder = aws_sdk_s3::config::Builder::from(&shared);

        if let Some(url) = &self.endpoint_url {
            s3_cfg_builder = s3_cfg_builder.endpoint_url(url);
        }
        if self.s3_force_path_style {
            s3_cfg_builder = s3_cfg_builder.force_path_style(true);
        }

        let s3_conf = s3_cfg_builder.build();
        Ok(aws_sdk_s3::Client::from_conf(s3_conf))
    }
}

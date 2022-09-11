mod dbus;
mod error;

use error::ClientError;
use tailor_api::{Color, ColorProfile, FanProfilePoint, ProfileInfo};
use zbus::Connection;

type ClientResult<T> = Result<T, ClientError>;

pub struct TailorConnection<'a> {
    profiles: dbus::ProfilesProxy<'a>,
    keyboard: dbus::KeyboardProxy<'a>,
    fan: dbus::FanProxy<'a>,
}

impl<'a> TailorConnection<'a> {
    pub async fn new() -> Result<TailorConnection<'a>, zbus::Error> {
        let connection = Connection::system().await?;

        let profiles = dbus::ProfilesProxy::new(&connection).await?;
        let keyboard = dbus::KeyboardProxy::new(&connection).await?;
        let fan = dbus::FanProxy::new(&connection).await?;

        Ok(Self {
            profiles,
            keyboard,
            fan,
        })
    }
}

impl<'a> TailorConnection<'a> {
    pub async fn add_keyboard_profile(
        &self,
        name: &str,
        profile: &ColorProfile,
    ) -> ClientResult<()> {
        let value = serde_json::to_string(profile)?;
        Ok(self.keyboard.add_profile(name, &value).await?)
    }

    pub async fn get_keyboard_profile(&self, name: &str) -> ClientResult<ColorProfile> {
        let profile_data = self.keyboard.get_profile(name).await?;
        Ok(serde_json::from_str(&profile_data)?)
    }

    pub async fn list_keyboard_profiles(&self) -> ClientResult<Vec<String>> {
        Ok(self.keyboard.list_profiles().await?)
    }

    pub async fn remove_keyboard_profile(&self, name: &str) -> ClientResult<()> {
        Ok(self.keyboard.remove_profile(name).await?)
    }

    pub async fn override_keyboard_color(&self, color: &Color) -> ClientResult<()> {
        let value = serde_json::to_string(color)?;
        Ok(self.keyboard.override_color(&value).await?)
    }
}

impl<'a> TailorConnection<'a> {
    pub async fn add_fan_profile(
        &self,
        name: &str,
        profile: &Vec<FanProfilePoint>,
    ) -> ClientResult<()> {
        let value = serde_json::to_string(profile)?;
        Ok(self.fan.add_profile(name, &value).await?)
    }

    pub async fn get_fan_profile(&self, name: &str) -> ClientResult<Vec<FanProfilePoint>> {
        let profile_data = self.fan.get_profile(name).await?;
        Ok(serde_json::from_str(&profile_data)?)
    }

    pub async fn list_fan_profiles(&self) -> ClientResult<Vec<String>> {
        Ok(self.fan.list_profiles().await?)
    }

    pub async fn remove_fan_profile(&self, name: &str) -> ClientResult<()> {
        Ok(self.fan.remove_profile(name).await?)
    }

    pub async fn override_fan_speed(&self, speed: u8) -> ClientResult<()> {
        Ok(self.fan.override_speed(speed).await?)
    }
}

impl<'a> TailorConnection<'a> {
    pub async fn add_global_profile(&self, name: &str, profile: &ProfileInfo) -> ClientResult<()> {
        let value = serde_json::to_string(profile)?;
        Ok(self.profiles.add_profile(name, &value).await?)
    }

    pub async fn get_global_profile(&self, name: &str) -> ClientResult<ProfileInfo> {
        let profile_data = self.profiles.get_profile(name).await?;
        Ok(serde_json::from_str(&profile_data)?)
    }

    pub async fn list_global_profiles(&self) -> ClientResult<Vec<String>> {
        Ok(self.profiles.list_profiles().await?)
    }

    pub async fn remove_global_profile(&self, name: &str) -> ClientResult<()> {
        Ok(self.profiles.remove_profile(name).await?)
    }

    pub async fn reload(&self) -> ClientResult<()> {
        Ok(self.profiles.reload().await?)
    }
}

#[cfg(test)]
mod test {
    use tailor_api::{Color, ColorPoint, ColorProfile, ColorTransition};

    use crate::TailorConnection;

    #[tokio::test]
    async fn test_connection() {
        let connection = TailorConnection::new().await.unwrap();
        let profile = ColorProfile::Multiple(vec![
            ColorPoint {
                color: Color { r: 0, g: 255, b: 0 },
                transition: ColorTransition::Linear,
                transition_time: 3000,
            },
            ColorPoint {
                color: Color { r: 255, g: 0, b: 0 },
                transition: ColorTransition::Linear,
                transition_time: 3000,
            },
            ColorPoint {
                color: Color { r: 0, g: 0, b: 255 },
                transition: ColorTransition::Linear,
                transition_time: 3000,
            },
        ]);

        let profile_name = "__test";
        connection
            .add_keyboard_profile(profile_name, &profile)
            .await
            .unwrap();
        assert_eq!(
            connection.get_keyboard_profile(profile_name).await.unwrap(),
            profile
        );
        connection
            .list_keyboard_profiles()
            .await
            .unwrap()
            .iter()
            .find(|s| s.as_str() == profile_name)
            .unwrap();
        connection
            .remove_keyboard_profile(profile_name)
            .await
            .unwrap();
    }
}
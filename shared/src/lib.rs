use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Message {
    pub message_type: MessageType,
    pub data: Vec<u8>,
}

impl Message {
    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(self)?)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Message, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(bytes)?)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum MessageType {
    Auth,
    UserLocation,
    MyLocation,
    Spawn,
    Despawn,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct UserLocation {
    pub location: Location,
}

impl UserLocation {
    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(self)?)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<UserLocation, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(bytes)?)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Auth {
    pub id: u64,
    pub url: String,
}

impl Auth {
    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(self)?)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Auth, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(bytes)?)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct Location {
    pub id: u64,
    pub x: f32,
    pub y: f32,
}

impl Location {
    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(self)?)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Location, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(bytes)?)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Spawn {
    pub id: u64,
    pub icon: String,
}

impl Spawn {
    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(self)?)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Spawn, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(bytes)?)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct Despawn {
    pub id: u64,
}

impl Despawn {
    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(self)?)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Despawn, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_deserialize() {
        let auth = Auth {
            id: 123,
            url: String::from("https://google.com/"),
        };

        let message = Message {
            message_type: MessageType::Auth,
            data: auth.serialize().unwrap(),
        };

        let bytes: Vec<u8> = message.serialize().unwrap();

        let decoded_message: Message = Message::deserialize(&bytes).unwrap();

        assert_eq!(message, decoded_message);

        let decoded_auth: Auth = Auth::deserialize(&decoded_message.data).unwrap();

        assert_eq!(auth, decoded_auth);
    }

    #[test]
    fn my_location_deserialize() {
        let my_location = Location {
            id: 123,
            x: 69.0,
            y: 420.0,
        };

        let message = Message {
            message_type: MessageType::Auth,
            data: my_location.serialize().unwrap(),
        };

        let bytes: Vec<u8> = message.serialize().unwrap();

        let decoded_message: Message = Message::deserialize(&bytes).unwrap();

        assert_eq!(message, decoded_message);

        let decoded_my_location: Location = Location::deserialize(&decoded_message.data).unwrap();

        assert_eq!(my_location, decoded_my_location);
    }

    #[test]
    fn user_locations_deserialize() {
        let external_locations = UserLocation {
            location: Location {
                id: 123,
                x: 69.0,
                y: 420.0,
            },
        };

        let message = Message {
            message_type: MessageType::Auth,
            data: external_locations.serialize().unwrap(),
        };

        let bytes: Vec<u8> = message.serialize().unwrap();

        let decoded_message: Message = Message::deserialize(&bytes).unwrap();

        assert_eq!(message, decoded_message);

        let decoded_external_locations: UserLocation =
            UserLocation::deserialize(&decoded_message.data).unwrap();

        assert_eq!(external_locations, decoded_external_locations);
    }
}

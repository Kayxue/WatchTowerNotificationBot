use serde::{Deserialize, Serialize};

pub mod request_body {

    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct UpdateRequestBody<'a> {
        pub event: &'a str,
        pub updated_containers: Vec<Containers<'a>>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Containers<'a> {
        pub name: &'a str,
        pub image: &'a str,
        pub old_id: &'a str,
        pub new_id: &'a str,
    }
}

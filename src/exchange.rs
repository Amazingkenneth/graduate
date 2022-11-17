use ChoosingState::EntryImage;
impl EntryImage {

    async fn search() -> Result<, Error> {
        use rand::Rng;
        use serde::Deserialize;

        let fetch_entry = async {
            let url =
                format!("https://pokeapi.co/api/v2/pokemon-species/{}", id);

            reqwest::get(&url).await?.json().await
        };

        let (entry, image): (Entry, _) =
            futures::future::try_join(fetch_entry, Self::fetch_image(id))
                .await?;

        Ok(Pokemon {
            number: id,
            name: entry.name.to_uppercase(),
            description: description
                .flavor_text
                .chars()
                .map(|c| if c.is_control() { ' ' } else { c })
                .collect(),
            image,
        })
    }
    async fn fetch_image(id: u16) -> Result<image::Handle, reqwest::Error> {
        let url = format!(
            "https://raw.githubusercontent.com/PokeAPI/sprites/master/sprites/pokemon/{}.png",
            id
        );

        #[cfg(not(target_arch = "wasm32"))]
        {
            let bytes = reqwest::get(&url).await?.bytes().await?;

            Ok(image::Handle::from_memory(bytes.as_ref().to_vec()))
        }

        #[cfg(target_arch = "wasm32")]
        Ok(image::Handle::from_path(url))
    }
}
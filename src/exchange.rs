use crate::*;
impl EntryImage {
    pub fn read_index() {}
    /*async fn search() -> Result<EntryImage, Error> {
        let fetch_entry = async {
            let url =
                format!("https://pokeapi.co/api/v2/pokemon-species/{}", id);

            reqwest::get(&url).await?.json().await
        };

        let (entry, image): (Entry, _) =
            futures::future::try_join(fetch_entry, Self::fetch_image(id))
                .await?;

        Ok(EntryImage {
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

    }*/
}

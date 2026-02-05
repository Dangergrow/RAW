use serde::{Deserialize, Serialize};
use url::Url;

pub const YANDEX_SEARCH: &str = "https://yandex.ru/search/?text=";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YandexService {
    pub name: &'static str,
    pub url: &'static str,
}

pub fn yandex_tiles() -> Vec<YandexService> {
    vec![
        YandexService {
            name: "Поиск",
            url: "https://yandex.ru",
        },
        YandexService {
            name: "Почта",
            url: "https://mail.yandex.ru",
        },
        YandexService {
            name: "Диск",
            url: "https://disk.yandex.ru",
        },
        YandexService {
            name: "Музыка",
            url: "https://music.yandex.ru",
        },
        YandexService {
            name: "Карты",
            url: "https://yandex.ru/maps",
        },
        YandexService {
            name: "Маркет",
            url: "https://market.yandex.ru",
        },
        YandexService {
            name: "Новости",
            url: "https://news.yandex.ru",
        },
        YandexService {
            name: "Погода",
            url: "https://yandex.ru/pogoda",
        },
    ]
}

pub fn omnibox_to_url(input: &str) -> String {
    if let Ok(parsed) = Url::parse(input) {
        return parsed.to_string();
    }
    if input.contains('.') && !input.contains(' ') {
        return format!("https://{}", input);
    }
    format!("{}{}", YANDEX_SEARCH, urlencoding::encode(input))
}

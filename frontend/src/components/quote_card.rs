// src/components/quote_card.rs

// dependencies
use crate::domain::Quote;
use gloo_net::http::Request;
use yew::prelude::*;

// const api_url
const API_URL: &str = "https://jacintodesign.github.io/quotes-api/data/quotes.json";

#[component]
pub fn QuoteCard() -> Html {
    let quotes = use_state(|| vec![]);
    let selected_quote = use_state(|| None);

    // Fetch quotes on mount
    {
        let quotes = quotes.clone();
        use_effect_with((), move |_| {
            let quotes = quotes.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let fetched_quotes: Vec<Quote> = Request::get(API_URL)
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                quotes.set(fetched_quotes);
            });
            || ()
        });
    }

    // Pick a random quote form the loaded list on button click
    let on_new_quote = {
        let quotes = quotes.clone();
        let selected_quote = selected_quote.clone();
        Callback::from(move |_| {
            let quotes = quotes.clone();
            if quotes.is_empty() {
                return;
            }
            let index = (js_sys::Math::random() * quotes.len() as f64) as usize;
            selected_quote.set(Some(quotes[index].clone()));
        })
    };

    // Display selected quote, or a placeholder until the user clicks
    let (display_text, display_author) = match (*selected_quote).clone() {
        Some(q) => (q.text, q.author),
        None => (
            AttrValue::from("Click 'New Quote' to get started!"),
            AttrValue::from("--"),
        ),
    };

    // Click handler which tweets out the quote and author
    let on_tweet = {
        let display_text = display_text.clone();
        let display_author = display_author.clone();
        Callback::from(move |_| {
            let tweet_url = format!(
                "https://x.com/intent/tweet?text=\"{}\" - {}",
                display_text, display_author
            );
            web_sys::window()
                .unwrap()
                .open_with_url_and_target(&tweet_url, "_blank")
                .unwrap();
        })
    };

    html! {
        <main class="quote-container">
            <section>
                <article class="quote-text">
                    <i class="fas fa-quote-left"></i>
                    <span> { &display_text } </span>
                </article>
                <article class="quote-author">
                    <span> { &display_author } </span>
                </article>
                <article class="button-container">
                    <button class="twitter-button" title="Tweet This!" onclick={on_tweet}>
                        <i class="fab fa-twitter"></i>
                    </button>
                    <button class="button" onclick={on_new_quote}>
                        { "New Quote" }
                    </button>
                </article>
            </section>
        </main>
    }
}
